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
pub struct GameConnectInfo
{
    /// User id.
    pub user_id: u128,
    /// Token for connecting to the game server.
    pub server_connect_token: ServerConnectToken,
    /// Data needed for a user to start a game.
    #[serde_as(as = "Bytes")]
    pub serialized_start_data: Vec<u8>,
}

impl GameConnectInfo
{
    pub fn new_from_id(user_id: u128) -> GameConnectInfo
    {
        GameConnectInfo{
            user_id,
            server_connect_token: ServerConnectToken::default(),
            serialized_start_data: Vec::default()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Report emitted by a game factory that has initialized a game.
///
/// Contains information needed by clients in order to set up their local game clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartReport
{
    pub connect_infos: Vec<GameConnectInfo>,
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
