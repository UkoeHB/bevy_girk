//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Requests that can be sent to the game.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameRequest
{
    GameModeRequest,
    ClickButton,
    None
}

//-------------------------------------------------------------------------------------------------------------------
