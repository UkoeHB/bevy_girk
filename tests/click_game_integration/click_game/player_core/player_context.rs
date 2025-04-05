//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use renet2::ClientId;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Static information in a player app.
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct ClickPlayerContext
{
    /// This client's id
    client_id: ClientId,
    /// Game duration config.
    duration_config: GameDurationConfig,
}

impl ClickPlayerContext
{
    /// New context
    pub fn new(
        client_id       : ClientId,
        duration_config : GameDurationConfig,
    ) -> ClickPlayerContext 
    {
        ClickPlayerContext{ client_id, duration_config }
    }

    pub fn id(&self) -> ClientId                         { self.client_id }
    pub fn duration_config(&self) -> &GameDurationConfig { &self.duration_config }
}

//-------------------------------------------------------------------------------------------------------------------
