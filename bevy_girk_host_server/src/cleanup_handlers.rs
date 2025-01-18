//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Move expired pending lobbies back to the lobbies cache.
pub(crate) fn clean_pending_lobbies(
    mut pending_lobbies_cache : ResMut<PendingLobbiesCache>,
    mut lobbies_cache         : ResMut<LobbiesCache>,
    mut users_cache           : ResMut<UsersCache>,
    user_server               : Res<HostUserServer>,
){
    for lobby in pending_lobbies_cache.drain_expired()
    {
        tracing::trace!(lobby.data.id, "removing expired pending lobby");

        // handle ack failure
        handle_ack_failure(lobby, &mut lobbies_cache, &mut users_cache, &user_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Remove ongoing games that are too old.
/// - this should not run very often, it is inefficient
pub(crate) fn clean_ongoing_games(
    mut game_hubs_cache     : ResMut<GameHubsCache>,
    mut ongoing_games_cache : ResMut<OngoingGamesCache>,
    mut users_cache         : ResMut<UsersCache>,
    hub_server              : Res<HostHubServer>,
    user_server             : Res<HostUserServer>,
){
    for ongoing_game in ongoing_games_cache.drain_expired()
    {
        let game_id     = ongoing_game.game_id;
        let game_hub_id = ongoing_game.game_hub_id;
        tracing::warn!(game_id, "removed expired ongoing game");

        // send game aborted message to users and update their states to idle
        send_game_abort_messages_and_update_states(
                game_id,
                &ongoing_game.start_infos,
                &mut users_cache,
                &user_server
            );

        // tell game hub to abort the game
        hub_server.send(game_hub_id, HostToHubMsg::Abort{ id: game_id });

        // remove game from game hub
        if let Err(_) = game_hubs_cache.remove_game(game_hub_id, game_id)
        { tracing::error!(game_hub_id, game_id, "failed removing expired game from hub cache"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Unregister disconnected game hubs that have expired in the dc buffer.
pub(crate) fn clean_game_hub_dc_buffer(world: &mut World)
{
    let disconnected_hubs: Vec<u128> = world.resource_mut::<GameHubDisconnectBuffer>().drain_expired().into_iter().collect();
    for disconnected_hub_id in disconnected_hubs
    {
        tracing::trace!(disconnected_hub_id, "unregistering expired disconnected game hub");

        // unregister the game hub
        world.syscall(disconnected_hub_id, unregister_game_hub);
    }
}

//-------------------------------------------------------------------------------------------------------------------
