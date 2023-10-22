//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Player initializer: data used in startup to initialize the player app.
/// This resource is consumed during initialization.
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct ClickPlayerInitializer
{
    /// The player's context.
    pub player_context: ClickPlayerContext,
}

//-------------------------------------------------------------------------------------------------------------------
