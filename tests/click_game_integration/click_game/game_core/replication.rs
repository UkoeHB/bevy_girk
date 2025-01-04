//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initializes all game components that may be replicated (excluding game framework components).
pub struct GameReplicationPlugin;

impl Plugin for GameReplicationPlugin
{
    fn build(&self, app: &mut App)
    {
        app.replicate::<PlayerId>()
            .replicate::<PlayerName>()
            .replicate::<PlayerScore>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
