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

pub(crate) fn user_is_registered(In(user_id): In<u128>, users_cache: Res<UsersCache>) -> bool
{
    users_cache.has_user(user_id)
}

//-------------------------------------------------------------------------------------------------------------------

// - note: we consider a game hub as 'registered' even if it's in the game hub dc buffer
pub(crate) fn game_hub_is_registered(
    In(game_hub_id)    : In<u128>,
    game_hubs_cache    : Res<GameHubsCache>,
    game_hub_dc_buffer : Res<GameHubDisconnectBuffer>,
) -> bool
{
    game_hubs_cache.has_hub(game_hub_id) || game_hub_dc_buffer.has_game_hub(game_hub_id)
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_lobby_join_message_and_update_state(
    token       : bevy_simplenet::RequestToken,
    lobby       : &LobbyData,
    users_cache : &mut UsersCache,
    user_server : &HostUserServer,
){
    // send lobby join message
    let user_id = token.client_id();
    let lobby_id = lobby.id;
    if let Err(_) = user_server.respond(token, HostToUserResponse::LobbyJoin{ lobby: lobby.clone() })
    { tracing::error!(lobby_id, user_id, "failed sending lobby join notification"); }

    // update user state
    if let Err(_) = users_cache.update_user_state(user_id, UserState::InLobby(lobby_id))
    { tracing::error!(lobby_id, user_id, "failed updating user state to in-lobby"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_lobby_leave_messages_and_update_states(
    lobby_data  : &LobbyData,
    users_cache : &mut UsersCache,
    user_server : &HostUserServer,
){
    for (user_id, _) in lobby_data.members.iter()
    {
        // send leave lobby message
        if let Err(_) = user_server.send(*user_id, HostToUserMsg::LobbyLeave{ id: lobby_data.id })
        { tracing::error!(user_id, "failed sending lobby leave notification"); }

        // update user state
        if let Err(_) = users_cache.update_user_state(*user_id, UserState::Idle)
        { tracing::error!(user_id, lobby_data.id, "failed updating user state to idle"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_lobby_state_messages(
    lobby_data  : &LobbyData,
    user_server : &HostUserServer,
){
    for (user_id, _) in lobby_data.members.iter()
    {
        // send lobby state message
        if let Err(_) = user_server.send(*user_id, HostToUserMsg::LobbyState{ lobby: lobby_data.clone() })
        { tracing::error!(user_id, "failed sending lobby state notification"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_pending_lobby_ack_requests_and_update_states(
    lobby_data  : &LobbyData,
    users_cache : &mut UsersCache,
    user_server : &HostUserServer,
){
    for (user_id, _) in lobby_data.members.iter()
    {
        // send pending lobby ack request
        if let Err(_) = user_server.send(*user_id, HostToUserMsg::PendingLobbyAckRequest{ id: lobby_data.id })
        { tracing::error!(user_id, lobby_data.id, "failed sending lobby ack request"); }

        // update user state
        if let Err(_) = users_cache.update_user_state(*user_id, UserState::InPendingLobby(lobby_data.id))
        { tracing::error!(user_id, lobby_data.id, "failed updating user state to in pending lobby"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_pending_lobby_ack_fails_and_update_states(
    lobby_data  : &LobbyData,
    users_cache : &mut UsersCache,
    user_server : &HostUserServer,
){
    for (user_id, _) in lobby_data.members.iter()
    {
        // send pending lobby ack fail
        if let Err(_) = user_server.send(*user_id, HostToUserMsg::PendingLobbyAckFail{ id: lobby_data.id })
        { tracing::error!(user_id, "failed sending lobby ack fail notification"); }

        // update user states
        if let Err(_) = users_cache.update_user_state(*user_id, UserState::InLobby(lobby_data.id))
        { tracing::error!(user_id, lobby_data.id, "failed updating user state to in lobby"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_game_abort_messages_and_update_states(
    aborted_lobby_id : u64,
    user_infos       : &Vec<GameConnectInfo>,
    users_cache      : &mut UsersCache,
    user_server      : &HostUserServer,
){
    for user_info in user_infos.iter()
    {
        // send lobby state message
        if let Err(_) = user_server.send(user_info.user_id, HostToUserMsg::GameAborted{ id: aborted_lobby_id })
        { tracing::error!(user_info.user_id, aborted_lobby_id, "failed sending game aborted notification"); }

        // update user state
        if let Err(_) = users_cache.update_user_state(user_info.user_id, UserState::Idle)
        { tracing::error!(aborted_lobby_id, user_info.user_id, "failed updating user state to idle"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn send_game_over_messages_and_update_states(
    game_id          : u64,
    game_over_report : GameOverReport,
    user_infos       : &Vec<GameConnectInfo>,
    users_cache      : &mut UsersCache,
    user_server      : &HostUserServer,
){
    for user_info in user_infos.iter()
    {
        let user_id = user_info.user_id;

        // check that user is in a game
        let Some(UserState::InGame(in_game_id)) = users_cache.get_user_state(user_info.user_id)
        else { tracing::warn!(user_id, game_id, "received game over report but user is not in a game"); continue; };

        // check that user is in the game associated with this game over report
        if in_game_id != game_id
        {
            tracing::warn!(user_id, in_game_id, game_id, "received game over report but user is in a different game");
            continue;
        }

        // send game over report to user
        if let Err(_) = user_server.send(
                user_id,
                HostToUserMsg::GameOver{ id: game_id, report: game_over_report.clone() }
            )
        { tracing::error!(user_id, game_id, "failed sending game over report"); }

        // update user state
        if let Err(_) = users_cache.update_user_state(user_id, UserState::Idle)
        { tracing::error!(user_id, game_id, "failed updating user state to idle"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_ack_failure(
    lobby         : Lobby,
    lobbies_cache : &mut LobbiesCache,
    users_cache   : &mut UsersCache,
    user_server   : &HostUserServer,
){
    // send ack fails to users
    send_pending_lobby_ack_fails_and_update_states(&lobby.data, users_cache, user_server);

    // put lobby in lobbies list
    if let Err(lobby) = lobbies_cache.insert_lobby(lobby)
    {
        // this should not happen, but must be handled
        tracing::error!(lobby.data.id, "failed inserting former pending lobby");

        // kick users from lobby
        send_lobby_leave_messages_and_update_states(&lobby.data, users_cache, user_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_register_user(
    In((user_id, env_type)) : In<(u128, bevy_simplenet::EnvType)>,
    mut users_cache         : ResMut<UsersCache>,
) -> bool
{
    if let Err(_) = users_cache.add_user(user_id, env_type)
    { tracing::error!(user_id, "failed registering user"); return false; }

    true
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_connect_user_to_game(
    user_id             : u128,
    users_cache         : &mut UsersCache,
    ongoing_games_cache : &OngoingGamesCache,
    user_server         : &HostUserServer,
) -> bool
{
    // check if the user is already connected to a game
    if let Some(UserState::InGame(_)) = users_cache.get_user_state(user_id)
    { tracing::warn!(user_id, "trying to connect a user, but user is already in-game"); return true; };

    // check if the user is in a game
    let Some((game_id, connect_info)) = ongoing_games_cache.get_user_connect_info(user_id)
    else { tracing::trace!(user_id, "trying to connect a user, user is not in a game"); return false; };

    // send game connect info to user
    if let Err(_) = user_server.send(user_id, HostToUserMsg::GameStart{ id: game_id, connect: connect_info.clone() })
    { tracing::error!(user_id, "failed sending game start notification"); }

    // update user state
    if let Err(_) = users_cache.update_user_state(user_id, UserState::InGame(game_id))
    { tracing::error!(game_id, user_id, "failed updating user state to in-game"); }

    true
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_reconnect_user_to_game(
    In(user_id)         : In<u128>,
    mut users_cache     : ResMut<UsersCache>,
    ongoing_games_cache : Res<OngoingGamesCache>,
    user_server         : Res<HostUserServer>,
) -> bool
{
    // check if user is idle
    // - if user is not idle, we assume it doesn't need to reconnect
    let Some(UserState::Idle) = users_cache.get_user_state(user_id)
    else { tracing::warn!(user_id, "trying to reconnect a user, but user is not idle"); return false; };

    // try to connect the user to a game
    return try_connect_user_to_game(user_id, &mut users_cache, &ongoing_games_cache, &user_server);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_ack_pending_lobby(
    In((user_id, lobby_id))   : In<(u128, u64)>,
    mut pending_lobbies_cache : ResMut<PendingLobbiesCache>,
    users_cache               : Res<UsersCache>,
) -> bool
{
    // get the user's pending lobby
    let Some(UserState::InPendingLobby(user_lobby_id)) = users_cache.get_user_state(user_id)
    else { tracing::trace!(user_id, "user tried to ack, but user is not in a pending lobby"); return false; };

    // validate lobby id
    if user_lobby_id != lobby_id
    {
        tracing::trace!(user_id, user_lobby_id, lobby_id, "user tried to ack, but user is in a different lobby");
        return false;
    };

    // ack the lobby
    let Ok(_) = pending_lobbies_cache.add_user_ack(lobby_id, user_id)
    else { tracing::trace!(user_id, "user tried to ack, but ack was rejected"); return false; };

    true
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_nack_pending_lobby(
    In((user_id, lobby_id))   : In<(u128, u64)>,
    mut pending_lobbies_cache : ResMut<PendingLobbiesCache>,
    mut lobbies_cache         : ResMut<LobbiesCache>,
    mut users_cache           : ResMut<UsersCache>,
    user_server               : Res<HostUserServer>,
) -> bool
{
    // get id of pending lobby the user is in
    let Some(UserState::InPendingLobby(user_lobby_id)) = users_cache.get_user_state(user_id)
    else { tracing::trace!(user_id, "user tried to nack, but user is not in a pending lobby"); return false; };

    // validate lobby id
    if user_lobby_id != lobby_id
    {
        tracing::trace!(user_id, user_lobby_id, lobby_id, "could not nack lobby, user is in a different lobby");
        return false;
    };

    // check if the pending lobby is fully acked
    // - we don't nack if fully acked because a fully-acked lobby is negotiating with game hubs to start the game; if
    //   we allowed nacking a fully-acked lobby, then users could abuse nacking to waste game hub resources
    if let Some(_) = pending_lobbies_cache.try_get_full_acked_lobby(lobby_id)
    { tracing::trace!(user_id, lobby_id, "could not nack lobby, lobby is fully acked"); return false; }

    // nack the pending lobby
    let Ok(lobby) = pending_lobbies_cache.remove_nacked_lobby(lobby_id, user_id)
    else { tracing::error!(lobby_id, user_id, "could not find expected pending lobby to nack"); return false; };

    // handle ack failure
    handle_ack_failure(lobby, &mut lobbies_cache, &mut users_cache, &user_server);

    true
}

//-------------------------------------------------------------------------------------------------------------------

/// same as try-nack, but doesn't fail if the lobby is fully acked
pub(crate) fn force_nack_pending_lobby(
    In((user_id, nack_id))   : In<(u128, Option<u64>)>,
    mut pending_lobbies_cache : ResMut<PendingLobbiesCache>,
    mut lobbies_cache         : ResMut<LobbiesCache>,
    mut users_cache           : ResMut<UsersCache>,
    user_server               : Res<HostUserServer>,
) -> bool
{
    // get id of pending lobby the user is in
    let Some(UserState::InPendingLobby(lobby_id)) = users_cache.get_user_state(user_id)
    else { tracing::trace!(user_id, "user tried to force nack, but user is not in a pending lobby"); return false; };

    // validate lobby id
    if let Some(nack_id) = nack_id
    {
        if lobby_id != nack_id
        {
            tracing::trace!(user_id, lobby_id, nack_id, "could not force nack lobby, user is in a different lobby");
            return false;
        }
    }

    // nack the pending lobby
    let Ok(lobby) = pending_lobbies_cache.remove_nacked_lobby(lobby_id, user_id)
    else { tracing::error!(lobby_id, user_id, "could not find expected pending lobby to nack"); return false; };

    // handle ack failure
    handle_ack_failure(lobby, &mut lobbies_cache, &mut users_cache, &user_server);

    true
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_remove_user_from_lobby(
    In((user_id, remove_id)) : In<(u128, Option<u64>)>,
    mut lobbies_cache       : ResMut<LobbiesCache>,
    mut users_cache         : ResMut<UsersCache>,
    user_server             : Res<HostUserServer>
) -> bool
{
    // get id of lobby the user is in
    let Some(UserState::InLobby(lobby_id)) = users_cache.get_user_state(user_id)
    else { tracing::trace!(user_id, "could not remove user from lobby, user is not in the lobby"); return false; };

    // validate lobby id
    if let Some(remove_id) = remove_id
    {
        if lobby_id != remove_id
        {
            tracing::trace!(user_id, lobby_id, remove_id, "could not remove user from lobby, user is in a different lobby");
            return false;
        }
    }

    // access the lobby
    let Some(lobby_ref) = lobbies_cache.lobby_ref_mut(lobby_id)
    else { tracing::error!(user_id, "could not remove user from lobby, lobby doesn't exist"); return false; };

    // if user is lobby owner, we need to discard the lobby
    // otherwise, we need to remove the user and send lobby state updates
    match lobby_ref.is_owner(user_id)
    {
        true =>
        {
            // tell users they have been removed from the lobby
            send_lobby_leave_messages_and_update_states(&lobby_ref.data, &mut users_cache, &user_server);

            // remove the lobby
            if let None = lobbies_cache.extract_lobby(lobby_id)
            { tracing::error!(lobby_id, "failed to remove lobby from lobbies cache when owner is removed"); }
        }
        false =>
        {
            // remove user from lobby
            if let None = lobby_ref.remove_member(user_id)
            { tracing::error!(lobby_id, user_id, "failed to remove user from lobby; user is present and not an owner"); }

            // update user state to idle
            if let Err(_) = users_cache.update_user_state(user_id, UserState::Idle)
            { tracing::error!(user_id, "failed setting user state to idle"); }

            // notify clent who left the lobby
            if let Err(_) = user_server.send(user_id, HostToUserMsg::LobbyLeave{ id: lobby_id })
            { tracing::error!(user_id, "failed sending lobby leave notification"); }

            // notify members of new lobby state
            send_lobby_state_messages(&lobby_ref.data, &user_server);
        }
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_remove_user_from_cache(
    In(user_id)     : In<u128>,
    mut users_cache : ResMut<UsersCache>,
) -> bool
{
    // get user state
    let Some(user_state) = users_cache.get_user_state(user_id)
    else { tracing::error!(user_id, "could not remove user from cache, user has no state"); return false; };

    // verify the user is either idle or in a game
    // - players in a game can reconnect to the game
    match user_state
    {
        UserState::Idle              => (),
        UserState::InLobby(_)        => return false,
        UserState::InPendingLobby(_) => return false,
        UserState::InGame(_)         => (),
    }

    // remove the user
    if let Err(_) = users_cache.remove_user(user_id)
    { tracing::error!(user_id, "user not in users cache as expected"); }

    true
}

//-------------------------------------------------------------------------------------------------------------------

/// try to ask a game hub to start a game for the given user's pending lobby
/// - returns `Ok(true)` if successful
/// - returns `Ok(false)` if lobby is not ready to launch
/// - returns `Err(())` if unable to launch a game due to an error (e.g. no game hubs available)
pub(crate) fn try_request_game_start(
    In((user_id, lobby_id)) : In<(u128, u64)>,
    pending_lobbies_cache   : Res<PendingLobbiesCache>,
    mut game_hubs_cache     : ResMut<GameHubsCache>,
    users_cache             : Res<UsersCache>,
    ongoing_games_cache     : Res<OngoingGamesCache>,
    hub_server              : Res<HostHubServer>,
) -> Result<bool, ()>
{
    // get the user's pending lobby
    let Some(UserState::InPendingLobby(user_lobby_id)) = users_cache.get_user_state(user_id)
    else { tracing::warn!(user_id, "could not request game start, user is not in pending lobby"); return Ok(false); };

    // validate lobby id
    if user_lobby_id != lobby_id
    {
        tracing::trace!(user_id, user_lobby_id, lobby_id, "could not request game start, user is in a different lobby");
        return Ok(false);
    };

    // sanity check: game should not be ongoing
    let None = ongoing_games_cache.get_connect_infos(lobby_id)
    else { tracing::error!(user_id, lobby_id, "user is in pending lobby that already has an ongoing game"); return Err(()); };

    // if lobby is not fully acked then game is not ready to launch
    let Some(lobby_data_ref) = pending_lobbies_cache.try_get_full_acked_lobby(lobby_id)
    else { tracing::trace!(user_id, lobby_id, "could not request game start, lobby is not fully acked"); return Ok(false); };

    // get game hub for requesting a game slot
    let Some(game_hub_id) = game_hubs_cache.highest_nonzero_capacity_hub()
    else { tracing::warn!(user_id, lobby_id, "could not request game start, no available game hubs"); return Err(()); };

    // check if game hub already has this game
    // - this can happen if the lobby was started on this game hub then the lobby expired and was re-acked
    // - note: if this game hub already has a game with this lobby id, it may be a lobby with different data; in that case,
    //         the game will unexpectedly fail to launch if a game start report comes back from the hub
    // - note: if different game hubs have this game then there will be a race between the hubs; all but one of them will
    //         have to shut down their game instance
    if game_hubs_cache.has_pending_game(game_hub_id, lobby_id)
    { tracing::warn!(game_hub_id, lobby_id, "skipped sending game start request, hub already has game"); return Ok(true); };

    // send request to game hub
    if let Err(_) = hub_server.send(
            game_hub_id,
            HostToHubMsg::StartGame(GameStartRequest{ lobby_data: lobby_data_ref.clone() })
        )
    { tracing::error!(game_hub_id, "failed sending game start request to game hub"); return Err(()); }

    // add pending game to game hub
    // - we assume the game hub is running this lobby's game until the hub explicitly notifies us otherwise
    if let Err(_) = game_hubs_cache.add_pending_game(game_hub_id, lobby_id)
    { tracing::error!(game_hub_id, lobby_id, "game hubs cache: game insertion error"); return Err(()); }

    Ok(true)
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn attempt_game_start_request(world: &mut World, user_id: u128, lobby_id: u64)
{
    match syscall(world, (user_id, lobby_id), try_request_game_start)
    {
        Ok(true)  => tracing::trace!(user_id, lobby_id, "requested game start for user's pending lobby"),
        Ok(false) => (),
        Err(())   =>
        {
            tracing::warn!(user_id, lobby_id, "requesting game start for user's pending lobby failed; aborting");

            // nack the pending lobby to abort it
            // - we force-nack since the game start request failed, meaning no game hub resources are tied to this
            //   specific game start request
            if !syscall(world, (user_id, Some(lobby_id)), force_nack_pending_lobby)
            { tracing::error!(user_id, lobby_id, "failed aborting pending lobby"); }
            else
            { tracing::trace!(user_id, lobby_id, "nacked an aborted pending lobby"); }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_abort_hub_pending_game(In((game_hub_id, game_id)) : In<(u128, u64)>, world: &mut World) -> bool
{
    // remove pending game from game hub
    // - do this before checking if there is a pending lobby in case the lobby expired
    if let Err(_) = world.resource_mut::<GameHubsCache>().remove_pending_game(game_hub_id, game_id) { return false; }

    // try to access pending lobby (will fail if lobby expired)
    // - we access lobby data in order to re-start the game on behalf of the lobby owner
    let Some(lobby_data_ref) = world.resource::<PendingLobbiesCache>().try_get_full_acked_lobby(game_id)
    else { tracing::warn!(game_hub_id, game_id, "aborted hub's pending game has no pending lobby"); return true; };

    // try to start the game again (hopefully on a game hub with sufficient capacity)
    attempt_game_start_request(world, lobby_data_ref.owner_id, lobby_data_ref.id);

    true
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_abort_hub_ongoing_game(
    In((game_hub_id, game_id)) : In<(u128, u64)>,
    mut game_hubs_cache        : ResMut<GameHubsCache>,
    mut ongoing_games_cache    : ResMut<OngoingGamesCache>,
    mut users_cache            : ResMut<UsersCache>,
    user_server                : Res<HostUserServer>,
) -> bool
{
    // remove ongoing game from game hub cache
    if let Err(_) = game_hubs_cache.remove_game(game_hub_id, game_id) { return false; }

    // remove game from ongoing games record
    let Ok(dead_game) = ongoing_games_cache.remove_ongoing_game(game_id)
    else { tracing::error!(game_hub_id, game_id, "could not remove aborted ongoing game"); return false; };

    // forward abort game message to users and update states to idle
    send_game_abort_messages_and_update_states(
            game_id,
            &dead_game.connect_infos,
            &mut users_cache,
            &user_server
        );

    true
}

//-------------------------------------------------------------------------------------------------------------------
