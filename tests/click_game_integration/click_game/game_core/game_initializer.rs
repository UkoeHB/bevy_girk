//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use renet2::ClientId;

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
    pub players: HashMap<ClientId, PlayerState>,
    /// Watchers.
    pub watchers: HashSet<ClientId>,
}

//-------------------------------------------------------------------------------------------------------------------
