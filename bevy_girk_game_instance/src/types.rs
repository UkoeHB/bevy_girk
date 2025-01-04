//local shortcuts

use bevy_girk_utils::ser_msg;
//third-party shortcuts
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Contains all data needed to launch a game with a game factory.
//todo: deserializing this can be optimized by storing Bytes and just taking a clone of the originating Bytes
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameLaunchPack
{
    /// Id of the game.
    pub game_id: u64,

    /// Game launch data (serialized).
    /// - Note: Client data in here should be pre-shuffled.
    #[serde_as(as = "Bytes")]
    pub game_launch_data: Vec<u8>,
}

impl GameLaunchPack
{
    pub fn new<T: Serialize>(game_id: u64, data: T) -> Self
    {
        Self{ game_id, game_launch_data: ser_msg(&data) }
    }
}

//-------------------------------------------------------------------------------------------------------------------
