//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Client framework config
#[derive(Resource, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ClientFwConfig
{
    /// Tick rate of the game.
    ticks_per_sec: Ticks,

    /// This client's id
    client_id: ClientIdType,
}

impl ClientFwConfig
{
    /// New game framework config
    pub fn new(
        ticks_per_sec : Ticks,
        client_id     : ClientIdType,
    ) -> ClientFwConfig 
    {
        if ticks_per_sec == Ticks(0) { panic!("ClientFwConfig: tick rate must be > 0!"); }
        ClientFwConfig{
                ticks_per_sec,
                client_id,
            }
    }

    /// Getters
    pub fn ticks_per_sec(&self) -> Ticks { self.ticks_per_sec }
    pub fn client_id(&self) -> ClientIdType { self.client_id }
}

//-------------------------------------------------------------------------------------------------------------------
