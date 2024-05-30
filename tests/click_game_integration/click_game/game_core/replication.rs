//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon_repair::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initializes all game components that may be replicated (including game framework components).
///
/// Uses `bevy_replicon_repair` to register components.
pub struct GameReplicationPlugin;

impl Plugin for GameReplicationPlugin
{
    fn build(&self, app: &mut App)
    {
        app.replicate_repair::<PlayerId>()
            .replicate_repair::<PlayerName>()
            .replicate_repair::<PlayerScore>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
