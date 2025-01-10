//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Report emitted by a client instance when leaving [`ClientAppState::Game`].
///
/// The event can be read in `OnExit(ClientAppState::Game)`.
#[derive(Event, Debug, Copy, Clone)]
pub enum ClientInstanceReport
{
    /// The client game ended because it needs a new connect token.
    RequestConnectToken(u64),
    /// The client game ended normally.
    Ended(u64),
    /// The client game was aborted.
    Aborted(u64),
}

//-------------------------------------------------------------------------------------------------------------------
