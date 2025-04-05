//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use renet2::ClientId;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Client framework config.
#[derive(Resource, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ClientFwConfig
{
    /// Tick rate of the game (not the client).
    ticks_per_sec: u32,

    /// The currently-running game's id.
    game_id: u64,

    /// This client's id
    client_id: ClientId,
}

impl ClientFwConfig
{
    /// Makes a new game framework config.
    pub fn new(
        ticks_per_sec : u32,
        game_id       : u64,
        client_id     : ClientId,
    ) -> ClientFwConfig 
    {
        if ticks_per_sec == 0 { panic!("ClientFwConfig: tick rate must be > 0!"); }
        ClientFwConfig{ ticks_per_sec, game_id, client_id }
    }

    /// Gets the tick rate of the game.
    pub fn ticks_per_sec(&self) -> u32 { self.ticks_per_sec }

    /// Gets the current game's id.
    pub fn game_id(&self) -> u64 { self.game_id }

    /// Gets this client's id.
    pub fn client_id(&self) -> ClientId { self.client_id }
}

//-------------------------------------------------------------------------------------------------------------------
