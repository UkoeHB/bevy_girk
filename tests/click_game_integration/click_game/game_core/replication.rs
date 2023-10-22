//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy_fn_plugin::*;
use bevy_replicon::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initializes all game components that may be replicated (including game framework components).
/// Depends on `bevy_replicon::replication_core::ReplicationCorePlugin`.
#[bevy_plugin]
pub fn GameReplicationPlugin(app: &mut App)
{
    app.replicate::<PlayerId>()
        .replicate::<PlayerName>()
        .replicate::<PlayerScore>()
        .replicate::<GameInitProgress>();
}

//-------------------------------------------------------------------------------------------------------------------
