//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

/// Information used by a client to connect to a game.
#[serde_as]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GameStartInfo
{
    /// The game id.
    pub game_id: u64,
    /// User's server id.
    pub user_id: u128,
    /// User's client id within the game.
    pub client_id: u64,
    /// Data needed for a user to start a game.
    #[serde_as(as = "Bytes")]
    pub serialized_start_data: Vec<u8>,
}

impl GameStartInfo
{
    /// Generate an empty start info from a user id.
    ///
    /// Used for testing.
    pub fn new_from_id(user_id: u128) -> GameStartInfo
    {
        GameStartInfo{
            game_id: 0u64,
            user_id,
            client_id: 0u64,
            serialized_start_data: Vec::default()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Report emitted by a game factory that has initialized a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartReport
{
    /// Metadata for generating in-memory connect tokens for the game.
    #[serde(skip)]
    pub memory_meta: Option<ConnectMetaMemory>,
    /// Metadata for generating native-target connect tokens for the game.
    pub native_meta: Option<ConnectMetaNative>,
    /// Metadata for generating wasm-target connect tokens for the game.
    pub wasm_meta: Option<ConnectMetaWasm>,
    /// Contains information needed by clients in order to set up their local game clients.
    pub start_infos: Vec<GameStartInfo>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Report emitted by a game instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameInstanceReport
{
    GameStart(u64, GameStartReport),
    GameOver(u64, GameOverReport),
    GameAborted(u64),
}

impl GameInstanceReport
{
    pub fn game_id(&self) -> u64
    {
        match self
        {
            GameInstanceReport::GameStart(id, _) => *id,
            GameInstanceReport::GameOver(id, _)  => *id,
            GameInstanceReport::GameAborted(id)  => *id,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
