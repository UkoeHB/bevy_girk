//local shortcuts
use crate::LobbyData;

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GameStartRequest
{
    pub lobby_data: LobbyData,
}

impl GameStartRequest
{
    pub fn game_id(&self) -> u64
    {
        self.lobby_data.id
    }
}

//-------------------------------------------------------------------------------------------------------------------
