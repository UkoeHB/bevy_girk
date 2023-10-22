//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn connected_game_hub(
    In(game_hub_id)        : In<u128>,
    mut game_hubs_cache    : ResMut<GameHubsCache>,
    mut game_hub_dc_buffer : ResMut<GameHubDisconnectBuffer>,
){
    // remove hub from disconnect buffer (if it exists there)
    let mut dc_buffered = false;
    if let Ok(_) = game_hub_dc_buffer.remove_game_hub(game_hub_id) { dc_buffered = true; }

    // add game hub (starts with initial capacity of zero)
    // - note: do this even if reconnecting in case of bugs
    let mut registered = false;
    if let Ok(_) = game_hubs_cache.insert_hub(game_hub_id) { registered = true; }

    match (dc_buffered, registered)
    {
        (true, false)  => tracing::info!(game_hub_id, "game hub reconnected"),
        (false, true)  => tracing::info!(game_hub_id, "registered game hub"),
        (true, true)   => tracing::error!(game_hub_id, "connected game hub was in dc buffer but not in hubs cache"),
        (false, false) => tracing::warn!(game_hub_id, "failed registering connected game hub"),
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn disconnected_game_hub(
    In(game_hub_id)        : In<u128>,
    mut game_hubs_cache    : ResMut<GameHubsCache>,
    mut game_hub_dc_buffer : ResMut<GameHubDisconnectBuffer>,
){
    // add game hub to game hub disconnect buffer
    if let Err(_) = game_hub_dc_buffer.add_game_hub(game_hub_id)
    { tracing::error!(game_hub_id, "game hub disconnected but it is already in game hub dc buffer"); }
    else
    { tracing::info!(game_hub_id, "game hub added to disconnect buffer"); }

    // set the hub's capacity to zero
    // - we do this so the game hub can't be assigned any new games while in the dc buffer
    if let Err(_) = game_hubs_cache.set_hub_capacity(game_hub_id, GameHubCapacity(0))
    { tracing::error!(game_hub_id, "failed to set game hub capacity to zero"); }
    else
    { tracing::trace!(game_hub_id, "set game hub capacity to zero on disconnect"); }
}

//-------------------------------------------------------------------------------------------------------------------

// Note: We only call this when a game hub in the disconnect buffer has expired.
pub(crate) fn unregister_game_hub(
    In(game_hub_id)         : In<u128>,
    mut game_hubs_cache     : ResMut<GameHubsCache>,
    mut ongoing_games_cache : ResMut<OngoingGamesCache>,
    mut users_cache         : ResMut<UsersCache>,
    game_hub_dc_buffer      : Res<GameHubDisconnectBuffer>,
    user_server             : Res<HostUserServer>,
){
    // sanity check: the game hub should not be in the game hub disconnect buffer
    if game_hub_dc_buffer.has_game_hub(game_hub_id)
    { tracing::error!(game_hub_id, "ignoring unregister game hub for game hub that is in disconnect buffer"); return; };

    // remove ongoing games associated with this game hub
    // - if a game hub is offline then there is no easy way to know if it will come back; we don't want reconnecting
    //   users to get stuck unable to do anything until the ongoing game expires (i.e. because their user state is
    //   stuck on InGame); we should assume game connection information is invalid if the game hub is disconnected,
    //   and assume reconnecting users that use that information would encounter a hopeless error
    if let Ok(lobby_ids_it) = game_hubs_cache.drain_games(game_hub_id)
    {
        for lobby_id in lobby_ids_it
        {
            // remove game from ongoing games cache
            // - warning: failure here is a critical error
            let Ok(removed_game) = ongoing_games_cache.remove_ongoing_game(lobby_id)
            else { tracing::error!(game_hub_id, lobby_id, "could not remove game while unregistering game hub"); continue; };

            // check the hub id
            if removed_game.game_hub_id != game_hub_id
            { tracing::error!(?removed_game, game_hub_id, "found game hub id mismatch while unregistering game hub"); }

            // send game aborted message to users and update their states to idle
            send_game_abort_messages_and_update_states(
                    lobby_id,
                    &removed_game.connect_infos,
                    &mut users_cache,
                    &user_server
                );
        }
    }
    else { tracing::error!(game_hub_id, "unable to drain games while unregistering game hub"); }

    // remove game hub
    if let Err(_) = game_hubs_cache.remove_hub(game_hub_id)
    { tracing::error!(game_hub_id, "unable to remove game hub while unregistering game hub"); }
    else
    { tracing::info!(game_hub_id, "unregistered game hub"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn hub_update_capacity(
    In((game_hub_id, new_capacity)) : In<(u128, GameHubCapacity)>,
    mut game_hubs_cache             : ResMut<GameHubsCache>,
    game_hub_dc_buffer              : Res<GameHubDisconnectBuffer>
){
    // sanity check: the game hub should not be in the game hub disconnect buffer
    if game_hub_dc_buffer.has_game_hub(game_hub_id)
    { tracing::error!(game_hub_id, "ignoring game hub capacity for game hub that is in disconnect buffer"); return; };

    // update the hub's capacity
    if let Err(_) = game_hubs_cache.set_hub_capacity(game_hub_id, new_capacity)
    { tracing::error!(game_hub_id, ?new_capacity, "failed to update game hub capacity"); }
    else
    { tracing::trace!(game_hub_id, ?new_capacity, "updated game hub capacity"); }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mote: The hub should send 'abort game' messages in response to 'abort game' commands from the host server, to
///       ensure the registered game is correctly removed from the hub cache (we need confirmation from the hub that it
///       doesn't have a running game before we can remove registered games from the hub cache).
pub(crate) fn hub_abort_game(In((game_hub_id, lobby_id)): In<(u128, u64)>, world: &mut World)
{
    tracing::trace!(game_hub_id, lobby_id, "received abort game from game hub");

    // try to abort pending game
    if syscall(world, (game_hub_id, lobby_id), try_abort_hub_pending_game)
    { tracing::trace!(game_hub_id, lobby_id, "aborted pending game"); return; }

    // try to abort ongoing game
    if syscall(world, (game_hub_id, lobby_id), try_abort_hub_ongoing_game)
    { tracing::trace!(game_hub_id, lobby_id, "aborted ongoing game"); return; }

    tracing::error!(game_hub_id, lobby_id, "unable to abort the hub's game");
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn hub_start_game(
    In((
        game_hub_id,
        game_id,
        game_start_request,
        game_start_report
    ))                        : In<(u128, u64, GameStartRequest, GameStartReport)>,
    mut game_hubs_cache       : ResMut<GameHubsCache>,
    mut pending_lobbies_cache : ResMut<PendingLobbiesCache>,
    mut ongoing_games_cache   : ResMut<OngoingGamesCache>,
    mut users_cache           : ResMut<UsersCache>,
    hub_server                : Res<HostHubServer>,
    user_server               : Res<HostUserServer>,
){
    // expect that the hub cache has this game pending
    if !game_hubs_cache.has_pending_game(game_hub_id, game_id)
    {
        tracing::error!(game_hub_id, game_id, "aborting game because game is not registered to hub cache as pending");
        if let Err(_) = hub_server.send(game_hub_id, HostToHubMsg::AbortGame{ id: game_id })
        { tracing::error!(game_hub_id, game_id, "failed sending game abort message to game hub"); }
        return;
    }

    // check that the lobby is fully acked and pending
    let Some(lobby_data_ref) = pending_lobbies_cache.try_get_full_acked_lobby(game_id)
    else
    {
        tracing::warn!(game_hub_id, game_id, "aborting game because lobby is unavailable");
        if let Err(_) = hub_server.send(game_hub_id, HostToHubMsg::AbortGame{ id: game_id })
        { tracing::error!(game_hub_id, game_id, "failed sending game abort message to game hub"); }
        return;
    };

    // verify that this game's lobby matches our cached lobby
    // note: lobby contents may change if the lobby times out post-ack, then e.g. changes members, then re-acks
    if game_start_request.lobby_data != *lobby_data_ref
    {
        tracing::warn!(game_hub_id, game_id, "aborting game because game's lobby data doesn't match cached lobby");
        if let Err(_) = hub_server.send(game_hub_id, HostToHubMsg::AbortGame{ id: game_id })
        { tracing::error!(game_hub_id, game_id, "failed sending game abort message to game hub"); }
        return;
    }

    // add to ongoing games
    let ongoing_game = OngoingGame{ game_id, game_hub_id, connect_infos: game_start_report.connect_infos };
    if let Err(_) = ongoing_games_cache.add_ongoing_game(ongoing_game)
    {
        tracing::error!(game_hub_id, "aborting game because registering ongoing game failed");
        if let Err(_) = hub_server.send(game_hub_id, HostToHubMsg::AbortGame{ id: game_id })
        { tracing::error!(game_hub_id, game_id, "failed sending game abort message to game hub"); }
        return;
    }

    // update game hub
    if let Err(_) = game_hubs_cache.upgrade_pending_game(game_hub_id, game_id)
    {
        tracing::error!(game_hub_id, "aborting game because updating game hub failed");
        if let Err(_) = hub_server.send(game_hub_id, HostToHubMsg::AbortGame{ id: game_id })
        { tracing::error!(game_hub_id, game_id, "failed sending game abort message to game hub"); }
        return;
    }

    // extract pending lobby
    // - we do this after registering the ongoing game so the lobby won't be erased if registration fails
    let Ok(lobby) = pending_lobbies_cache.remove_lobby(game_id)
    else { tracing::error!(game_hub_id, game_id, "failed extracting pending lobby"); return; };

    // connect lobby members to the game
    for (user_id, _) in lobby.data.members.iter()
    {
        if !try_connect_user_to_game(*user_id, &mut users_cache, &ongoing_games_cache, &user_server)
        { tracing::error!(user_id, game_id, "failed connecting user to new game"); }
    }

    tracing::trace!(game_hub_id, game_id, "started new game on game hub");
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn hub_game_over(
    In((game_hub_id, game_id, game_over_report)) : In<(u128, u64, GameOverReport)>,
    mut game_hubs_cache                          : ResMut<GameHubsCache>,
    mut ongoing_games_cache                      : ResMut<OngoingGamesCache>,
    mut users_cache                              : ResMut<UsersCache>,
    user_server                                  : Res<HostUserServer>,
){
    tracing::trace!(game_hub_id, game_id, "received game over report from game hub");

    // remove game from game hub cache: pending games
    // - we remove from the pending games in case a game was somehow not upgraded from pending (unexpected)
    if let Ok(_) = game_hubs_cache.remove_pending_game(game_hub_id, game_id)
    { tracing::warn!(game_hub_id, game_id, "received game over report for pending game in hub"); return; }

    // remove game from game hub cache: ongoing games
    // - we return on error so if game hubs lie about games they are in control of, no state will be impacted
    if let Err(_) = game_hubs_cache.remove_game(game_hub_id, game_id)
    { tracing::error!(game_hub_id, "received game over report for game not registered to hub"); return; }

    // remove game from ongoing games record
    let Ok(dead_game) = ongoing_games_cache.remove_ongoing_game(game_id)
    else { tracing::error!(game_hub_id, game_id, "could not remove ongoing game matching game over report"); return; };

    // forward game over report to users and update states to idle
    send_game_over_messages_and_update_states(
            game_id,
            game_over_report,
            &dead_game.connect_infos,
            &mut users_cache,
            &user_server
        );
}

//-------------------------------------------------------------------------------------------------------------------
