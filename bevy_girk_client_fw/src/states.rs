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
/// Note that we can remain in [`ClientFwMode::Init`] even if in state [`ClientInitializationState::Done`] if the client
/// is done initializing but the game is not (e.g. because it is waiting for other clients).
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientFwMode
{
    /// Runs in [`GameFwState::Init`] OR runs when the client is reinitializing.
    #[default]
    Init,
    /// Runs in [`GameFwState::Game`].
    Game,
    /// Runs in [`GameFwState::End`].
    End
}

//-------------------------------------------------------------------------------------------------------------------
