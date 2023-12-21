//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Remove expired pending games and send 'abort game' to host.
pub(crate) fn clean_pending_games(
    mut pending_games_cache : ResMut<PendingGamesCache>,
    host_client             : Res<HostHubClient>,
){
    for game_start_request in pending_games_cache.drain_expired()
    {
        let game_id = game_start_request.game_id();
        tracing::warn!(game_id, "removed expired pending game");

        // notify the host the game was aborted
        if let Err(_) = host_client.send(HubToHostMsg::Abort{ id: game_id })
        { tracing::error!(game_id, "failed sending abort game to host"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Remove running games that are expired or no longer running.
/// - This should not run very often, it is inefficient.
pub(crate) fn clean_running_games(
    mut running_games_cache : ResMut<RunningGamesCache>,
    host_client             : Res<HostHubClient>,
){
    for mut game_instance in running_games_cache.drain_invalid()
    {
        let game_id: u64 = game_instance.id();
        tracing::warn!(game_id, "removed invalid running game");

        // if game instance completed successfully, we don't need to do anything else
        if let Some(true) = game_instance.try_get() { continue; }

        // command game instance to abort (otherwise it may hang)
        if let Err(_) = game_instance.send_command(GameInstanceCommand::Abort)
        { tracing::error!(game_id, "failed sending abort game command to game instance"); }

        // notify the host the game was aborted
        if let Err(_) = host_client.send(HubToHostMsg::Abort{ id: game_id })
        { tracing::error!(game_id, "failed sending abort game to host"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------
