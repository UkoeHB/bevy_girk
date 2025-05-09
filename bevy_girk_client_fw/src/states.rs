//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Over-arching client states differentiating between the outer client shell and the inner game state.
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientAppState
{
    /// The client is loading on startup.
    ///
    /// The transition from `Self::Loading` to `Self::Client` is handled by `iyes_progress`. It will occur
    /// immediately on startup if you don't track progress with any systems.
    #[default]
    Loading,
    /// The client is displaying its outer shell, where users can set up and start games.
    Client,
    /// The client is displaying the game client, where users can play the game.
    Game
}

//-------------------------------------------------------------------------------------------------------------------

/// Client intialization state.
///
/// These states only run in [`ClientAppState::Game`].
#[derive(SubStates, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
#[source(ClientAppState = ClientAppState::Game)]
pub enum ClientInitState
{
    /// Client fw state when the client fw is initializing or reinitializing.
    #[default]
    InProgress,
    /// Client fw state when the client fw is done initializing.
    Done
}

//-------------------------------------------------------------------------------------------------------------------

/// Client framework state.
///
/// These states only run in [`ClientAppState::Game`].
///
/// The state transitions for `Connecting`, `Syncing`, and `Init` are controlled by users of the client framework.
/// This is handled automatically if you use [`prepare_girk_client_app()`].
///
/// Once you enter `Init`, the client framework will take over for the `Init`/`Game`/`End` transitions.
///
/// For the sake of clarity, our documentation here reflects the behavior added by [`prepare_girk_client_app()`].
#[derive(SubStates, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
#[source(ClientAppState = ClientAppState::Game)]
pub enum ClientFwState
{
    /// Runs when [`ClientAppState::Game`] has just been entered and before the client is fully set up to
    /// run a game. This state is especially useful for setting up local-player games where you need to wait for
    /// the game app to emit setup information (which may occur after a small delay if running the game in a child
    /// process).
    /// - The [`ClientFwConfig`] and other setup details might *not* be added to the app yet.
    /// - Always runs at least one full tick.
    /// - While this runs, the `renet` method `client_disconnected()` will return true.
    /// - Client requests sent while in this state will always be dropped.
    /// - Game messages will never be received in this tick.
    ///
    /// ### Explanation
    ///
    /// This runs at least one full tick while disconnected because we do not initialize the `renet` client in the
    /// first tick while disconnected.
    #[default]
    Setup,
    /// Runs when the client is connecting to the game.
    /// - Always runs at least one full tick.
    /// - While this runs, the `renet` method `client_connecting()` will return true.
    /// - Client requests sent while in this state will always be dropped.
    /// - Game messages will never be received in this tick.
    Connecting,
    /// Runs when the client is connected and is waiting to synchronize with the game.
    /// - Always runs at least one full tick.
    /// - The first tick this runs, the `renet` method `client_just_connected()` will return true.
    /// - Client requests sent while in this state will succeed unless the client disconnects or the game shuts down.
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
    /// The first tick of this state is the first tick after a disconnect where client messages can be sent successfully.
    /// We achieve this by relying on synchronization between the first tick of this state and replicon's
    /// `ClientSet::ResetEvents`, which will run when `client_just_connected()` is true (and drop all unsent messages).
    /// Old messages sent in the ambiguous region just after a disconnect but before the disconnect is detected will
    /// be dropped by renet when they fail to send.
    Syncing,
    /// Runs when the client is synchronized and is waiting to finish initializing.
    /// - Always runs at least one full tick.
    /// - Client requests sent while in this state will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will be consumed in this tick, including any messages buffered while `Syncing`.
    ///
    /// Initialization ends when in state [`ClientInitState::Done`] and the game is no longer in
    /// [`GameFwState::Init`].
    ///
    /// ### Explanation
    ///
    /// This runs at least one full tick after synchronizing because we ignore game framework messages that contain
    /// state changes until we are in [`ClientInitState::Done`].
    /// Meanwhile, [`ClientInitState::Done`] will not be entered until `iyes_progress` has fully initialized,
    /// and we do not fully initialize until the first tick that `Init` runs.
    /// Importantly, `iyes_progress` is configured to run in PostUpdate at the end of each tick, which means the end
    /// of the first tick in state [`ClientFwState::Init`] is the first tick where [`ClientInitState::Done`]
    /// may be entered (which means only the next client tick may enter [`ClientFwState::Game`] or [`ClientFwState::End`]).
    Init,
    /// Runs in [`GameFwState::Game`] when the client is not initializing.
    /// - Client requests sent while in this state will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will be consumed in this tick.
    ///
    /// This state will not run if the client connects to a game in [`GameFwState::End`].
    Game,
    /// Runs in [`GameFwState::End`] when the client is not initializing.
    /// - Client requests sent while in this state will succeed unless the client disconnects or the game shuts down.
    /// - Game messages will be consumed in this tick. Note that no messages will appear if the game shuts down.
    ///
    /// To fully exit the game, you should set [`ClientAppState::Client`] (e.g. with an "Exit" button).
    End
}

//-------------------------------------------------------------------------------------------------------------------
