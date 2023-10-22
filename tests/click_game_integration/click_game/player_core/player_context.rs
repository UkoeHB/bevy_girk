//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Static information in a player app.
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct ClickPlayerContext
{
    /// This client's id
    client_id: ClientIdType,
    /// Game duration config.
    duration_config: GameDurationConfig,
}

impl ClickPlayerContext
{
    /// New context
    pub fn new(
        client_id       : ClientIdType,
        duration_config : GameDurationConfig,
    ) -> ClickPlayerContext 
    {
        ClickPlayerContext{ client_id, duration_config }
    }

    pub fn id(&self) -> ClientIdType                     { self.client_id }
    pub fn duration_config(&self) -> &GameDurationConfig { &self.duration_config }
}

//-------------------------------------------------------------------------------------------------------------------
