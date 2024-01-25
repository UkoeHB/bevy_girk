//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Client framework config.
#[derive(Resource, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ClientFwConfig
{
    /// Tick rate of the game (not the client).
    ticks_per_sec: u32,

    /// This client's id
    client_id: ClientId,
}

impl ClientFwConfig
{
    /// Makes a new game framework config.
    pub fn new(
        ticks_per_sec : u32,
        client_id     : ClientId,
    ) -> ClientFwConfig 
    {
        if ticks_per_sec == 0 { panic!("ClientFwConfig: tick rate must be > 0!"); }
        ClientFwConfig{ ticks_per_sec, client_id }
    }

    /// Gets the tick rate of the game.
    pub fn ticks_per_sec(&self) -> u32 { self.ticks_per_sec }

    /// Gets this client's id.
    pub fn client_id(&self) -> ClientId { self.client_id }
}

//-------------------------------------------------------------------------------------------------------------------
