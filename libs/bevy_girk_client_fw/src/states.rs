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
/// Note that we can be in `ClientInitializationState::Done` but also in `ClientFwMode::Init` if the client
/// is done initializing but the game is not (e.g. because it is waiting for other clients).
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientFwMode
{
    /// Client fw state when the game fw state is 'initializing' OR when the client is reinitializing.
    #[default]
    Init,
    /// Client fw state when the game fw state is 'in game'.
    Game,
    /// Client fw state when the game fw state is 'game end'.
    End
}

//-------------------------------------------------------------------------------------------------------------------
