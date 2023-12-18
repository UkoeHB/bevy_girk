//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Used by a game factory to initialize a client in the game.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInitDataForGame
{
    /// The client's environment type.
    pub env: bevy_simplenet::EnvType,

    /// The client's server-side user id.
    pub user_id: u128,

    /// Client init data for use in initializing a game (serialized).
    #[serde_as(as = "Bytes")]
    pub data: Vec<u8>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Contains all data needed to launch a game with a game factory.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameLaunchPack
{
    /// Id of the game.
    pub game_id: u64,

    /// Game init data (serialized).
    #[serde_as(as = "Bytes")]
    pub game_init_data: Vec<u8>,

    /// Client init data.
    /// - Note: This should should be pre-shuffled.
    pub client_init_data: Vec<ClientInitDataForGame>
}

impl GameLaunchPack
{
    pub fn new(game_id: u64, game_init_data: Vec<u8>, client_init_data: Vec<ClientInitDataForGame>) -> Self
    {
        Self{ game_id, game_init_data, client_init_data }
    }
}

//-------------------------------------------------------------------------------------------------------------------
