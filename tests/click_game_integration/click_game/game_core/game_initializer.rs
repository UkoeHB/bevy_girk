//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::{HashMap, HashSet};

//-------------------------------------------------------------------------------------------------------------------

/// Data used on startup to initialize a game app.
/// This resource is consumed during initialization.
#[derive(Resource)]
pub struct ClickGameInitializer
{
    /// Game context.
    pub game_context: ClickGameContext,
    /// Player states.
    pub players: HashMap<ClientIdType, PlayerState>,
    /// Watchers.
    pub watchers: HashSet<ClientIdType>,
}

//-------------------------------------------------------------------------------------------------------------------
