//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Client intialization state
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientInitializationState
{
    /// Client fw state when the client fw is initializing or reinitializing.
    #[default]
    InProgress,
    /// Client fw state when the client fw is done initializing.
    Done
}

//-------------------------------------------------------------------------------------------------------------------

/// Client framework mode.
///
/// The mode transitions for `Connecting`, `Syncing`, and `Init` are controlled by users of the client framework.
/// This is handled automatically if you use [`prepare_girk_client_app()`].
///
/// Once you enter `Init`, the client framework will take over for the `Init`/`Game`/`End` transitions until the framework
/// receives a [`ClientFwCommand::ReInitialize`] which will set it back to `Connecting`.
///
/// For the sake of clarity, our documentation here reflects the behavior added by [`prepare_girk_client_app()`].
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientFwMode
{
    /// Runs when the client is connecting to the game.
    /// - Always runs at least one full tick.
    /// - The first tick this runs, the `renet` method `client_just_disconnected()` will return true.
    /// - Client requests sent while in this mode will always be dropped.
    /// - Game messages will never be received in this tick.
    ///
    /// ### Explanation
    ///
    /// This runs at least one full tick while disconnected because we do not initialize the `renet` client in the
    /// first tick while disconnected.
    #[default]
    Connecting,
    /// Runs when the client is connected and is waiting to synchronize with the game.
    /// - Always runs at least one full tick.
    /// - The first tick this runs, the `renet` method `client_just_connected()` will return true.
    /// - Client requests sent while in this mode will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will not be consumed in this tick, they will only be buffered until the client is synchronized.
    ///
    /// The client will be considered 'synchronized' when it has received its first `bevy_replicon` replication message
    /// after connecting to the game server.
    ///
    /// ### Explanation
    ///
    /// This runs at least one full tick while just connected because we disable replicon's `ClientSet::Receive` for
    /// the first tick when connected.
    ///
    /// The first tick of this mode is the first tick after a disconnect where client messages can be sent successfully.
    /// We achieve this by relying on synchronization between the first tick of this mode and replicon's
    /// `ClientSet::ResetEvents`, which will run when `client_just_connected()` is true (and drop all unsent messages).
    /// Old messages sent in the ambiguous region just after a disconnect but before the disconnect is detected will
    /// be dropped by renet when they fail to send.
    Syncing,
    /// Runs when the client is synchronized and is waiting to finish initializing.
    /// - Always runs at least one full tick.
    /// - Client requests sent while in this mode will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will be consumed in this tick, including any messages buffered while `Syncing`.
    ///
    /// Initialization ends when in state [`ClientInitializationState::Done`] and the game is no longer in
    /// [`GameFwState::Game`].
    ///
    /// ### Explanation
    ///
    /// This runs at least one full tick after synchronizing because we ignore game framework messages that contain
    /// mode changes until we are in [`ClientInitializationState::Done`].
    /// Meanwhile, [`ClientInitializationState::Done`] will not be entered until `iyes_progress` has fully initialized,
    /// and we do not fully initialize until the first tick that `Init` runs.
    /// Importantly, `iyes_progress` is configured to run in PostUpdate at the end of each tick, which means the end
    /// of the first tick in state [`ClientFwMode::Init`] is the first tick where [`ClientInitializationState::Done`]
    /// may be entered (which means only the next client tick may enter [`ClientFwMode::Game`] or [`ClientFwMode::End`]).
    Init,
    /// Runs in [`GameFwState::Game`] when the client is not initializing.
    /// - Client requests sent while in this mode will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will be consumed in this tick.
    ///
    /// This mode will not run if the client connects to a game in [`GameFwState::End`].
    Game,
    /// Runs in [`GameFwState::End`] when the client is not initializing.
    /// - Client requests sent while in this mode will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will be consumed in this tick. Note that no messages will appear if the game shuts down.
    ///
    /// Once in this state, the client will never change states.
    /// This is because eventually the game will shut down but we don't want the client to pointlessly transition
    /// to a loading screen.
    End
}

//-------------------------------------------------------------------------------------------------------------------
