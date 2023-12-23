//local shortcuts

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The location of a game relative to a game client.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameLocation
{
    /// The game is on the local machine.
    Local,
    /// The game is on a host server.
    Hosted,
}

//-------------------------------------------------------------------------------------------------------------------
