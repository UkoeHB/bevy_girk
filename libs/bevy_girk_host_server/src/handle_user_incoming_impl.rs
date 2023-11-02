//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn register_user(In((user_id, env_type)): In<(u128, bevy_simplenet::EnvType)>, world: &mut World)
{
    // register user
    if syscall(world, (user_id, env_type), try_register_user)
    { tracing::trace!(user_id, ?env_type, "registered user"); }
    else
    { tracing::error!(user_id, "failed trying to register a user"); }

    // reconnect to game if in a game
    if syscall(world, user_id, try_reconnect_user_to_game)
    { tracing::trace!(user_id, "reconnected user to game"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn unregister_user(In(user_id): In<u128>, world: &mut World)
{
    // nack pending lobby the user is in
    if syscall(world, (user_id, None), force_nack_pending_lobby)
    { tracing::trace!(user_id, "force nacked pending lobby while unregistering user"); }

    // remove user from lobby
    if syscall(world, (user_id, None), try_remove_user_from_lobby)
    { tracing::trace!(user_id, "removed user from lobby while unregistering user"); }

    // unregister the user
    if syscall(world, user_id, try_remove_user_from_cache)
    { tracing::trace!(user_id, "unregistered user"); }
    else
    { tracing::error!(user_id, "failed removing user from cache while unregistering user"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_reset_lobby(In(token): In<bevy_simplenet::RequestToken>, world: &mut World)
{
    // check if the user is in-game
    // - returning here auto-rejects the request
    let user_id = token.client_id();
    if let Some(UserState::InGame(_)) = world.resource::<UsersCache>().get_user_state(user_id)
    { tracing::trace!(user_id, "unable to reset user's lobby state, user is in-game"); return; }

    // nack pending lobby the user is in
    if syscall(world, (user_id, None), force_nack_pending_lobby)
    { tracing::trace!(user_id, "force nacked pending lobby while reseting user's lobby state"); }

    // remove user from lobby
    if syscall(world, (user_id, None), try_remove_user_from_lobby)
    { tracing::trace!(user_id, "removed user from lobby while reseting user's lobby state"); }

    // acknowledge the request on success
    if let Err(_) = world.resource_mut::<HostUserServer>().acknowledge(token)
    { tracing::error!(user_id, "failed acking reset lobby request"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_get_lobby(
    In((token, request)) : In<(bevy_simplenet::RequestToken, LobbySearchRequest)>,
    lobbies_cache        : Res<LobbiesCache>,
    user_server          : Res<HostUserServer>,
){
    // query the lobbies cache
    let result = get_searched_lobbies(&*lobbies_cache, request);

    // send result to user
    let user_id = token.client_id();
    if let Err(_) = user_server.respond(token, HostToUserResponse::LobbySearchResult(result))
    { tracing::error!(user_id, "failed sending lobby request response"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_make_lobby(
    In((
        token,
        member_color,
        password,
        custom_data,
    ))                : In<(bevy_simplenet::RequestToken, LobbyMemberColor, String, Vec<u8>)>,
    mut lobbies_cache : ResMut<LobbiesCache>,
    mut users_cache   : ResMut<UsersCache>,
    user_server       : Res<HostUserServer>,
){
    // verify user is not already in a lobby
    let user_id = token.client_id();
    let Some(UserState::Idle) = users_cache.get_user_state(user_id)
    else { tracing::trace!(user_id, "could not make lobby, user is not idle"); return; };

    // assemble lobby member data
    let Some(env) = users_cache.get_user_env(user_id)
    else { tracing::error!(user_id, "failed getting user env"); return; };
    let member_data = LobbyMemberData{ env, color: member_color };

    // make lobby
    let Ok(lobby_id) = lobbies_cache.new_lobby(user_id, member_data, password, custom_data)
    else { tracing::trace!(user_id, ?member_data, "failed making new lobby"); return; };
    tracing::trace!(lobby_id, user_id, ?member_data, "created new lobby");

    // access the lobby
    let Some(lobby_ref) = lobbies_cache.lobby_ref(lobby_id)
    else { tracing::error!(lobby_id, user_id, "error getting new lobby's reference"); return; };

    // send join message and update state
    send_lobby_join_message_and_update_state(token, &lobby_ref.data, &mut users_cache, &user_server);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_join_lobby(
    In((
        token,
        lobby_id,
        member_color,
        password
    ))                : In<(bevy_simplenet::RequestToken, u64, LobbyMemberColor, String)>,
    mut lobbies_cache : ResMut<LobbiesCache>,
    mut users_cache   : ResMut<UsersCache>,
    user_server       : Res<HostUserServer>,
){
    // verify user is not already in a lobby
    let user_id = token.client_id();
    let Some(UserState::Idle) = users_cache.get_user_state(user_id)
    else { tracing::trace!(lobby_id, user_id, "could not join lobby, user is not idle"); return; };

    // assemble lobby member data
    let Some(env) = users_cache.get_user_env(user_id)
    else { tracing::error!(user_id, "failed getting user env"); return; };
    let member_data = LobbyMemberData{ env, color: member_color };

    // try to join the lobby
    if !lobbies_cache.try_add_member(lobby_id, user_id, member_data, &password)
    { tracing::trace!(lobby_id, user_id, ?member_data, "could not join lobby, join request rejected"); return; };

    // try to get the lobby
    let Some(lobby_ref) = lobbies_cache.lobby_ref_mut(lobby_id)
    else { tracing::trace!(lobby_id, user_id, "could not join lobby, lobby doesn't exist"); return; };

    // send join message and update state
    tracing::trace!(lobby_id, user_id, "user joined lobby");
    send_lobby_join_message_and_update_state(token, &lobby_ref.data, &mut users_cache, &user_server);

    // notify lobby members of new lobby state
    send_lobby_state_messages(&lobby_ref.data, &user_server);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_leave_lobby(In((user_id, lobby_id)): In<(u128, u64)>, world: &mut World)
{
    // nack pending lobby the user is in
    // - if the pending lobby is fully acked then nothing will happen
    if syscall(world, (user_id, lobby_id), try_nack_pending_lobby)
    { tracing::trace!(user_id, lobby_id, "nacked pending lobby because user left lobby"); }

    // remove user from lobby
    if syscall(world, (user_id, Some(lobby_id)), try_remove_user_from_lobby)
    { tracing::trace!(user_id, lobby_id, "removed user from lobby because user left lobby"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_launch_lobby_game(
    In((user_id, lobby_id))   : In<(u128, u64)>,
    mut pending_lobbies_cache : ResMut<PendingLobbiesCache>,
    mut lobbies_cache         : ResMut<LobbiesCache>,
    mut users_cache           : ResMut<UsersCache>,
    user_server               : Res<HostUserServer>,
){
    // get id of lobby the user is in
    let Some(UserState::InLobby(users_lobby_id)) = users_cache.get_user_state(user_id)
    else { tracing::trace!(user_id, "failed launching game, user is not in the lobby"); return; };

    // validate lobby id
    if users_lobby_id != lobby_id
    { tracing::trace!(user_id, users_lobby_id, lobby_id, "failed launching game, user is in a different lobby"); return; };

    // check if user is owner of the lobby
    let Some(lobby_ref) = lobbies_cache.lobby_ref(lobby_id)
    else { tracing::error!(user_id, lobby_id, "user's lobby is missing"); return; };

    if !lobby_ref.is_owner(user_id)
    { tracing::trace!(user_id, lobby_id, "failed launching game, user is not lobby owner"); return; }

    // check if lobby can be launched
    if !lobbies_cache.lobby_checker().can_launch(lobby_ref)
    { tracing::trace!(user_id, lobby_id, "failed launching game, lobby has insufficient players"); return; }

    // send ack requests to users and update states to 'in pending lobby'
    send_pending_lobby_ack_requests_and_update_states(&lobby_ref.data, &mut users_cache, &user_server);

    // extract lobby
    // - warning: failure here is a critical error
    let Some(lobby) = lobbies_cache.extract_lobby(lobby_id)
    else { tracing::error!(user_id, "extract lobby error"); return; };

    // move lobby to pending
    // - warning: failure here is a critical error
    if let Err(_) = pending_lobbies_cache.add_lobby(lobby) { tracing::error!("insert pending lobby error"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_nack_pending_lobby(In((user_id, lobby_id)): In<(u128, u64)>, world: &mut World)
{
    // nack pending lobby the user is in
    // - if the pending lobby is fully acked then nothing will happen
    if syscall(world, (user_id, lobby_id), try_nack_pending_lobby)
    { tracing::trace!(user_id, lobby_id, "nacked pending lobby because user rejected ack"); }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn user_ack_pending_lobby(In((user_id, lobby_id)): In<(u128, u64)>, world: &mut World)
{
    // try to ack the user's current pending lobby
    if !syscall(world, (user_id, lobby_id), try_ack_pending_lobby)
    { tracing::trace!(user_id, lobby_id, "failed trying to ack pending lobby"); return; }
    else
    { tracing::trace!(user_id, lobby_id, "acked pending lobby"); }

    // try to start the pending lobby as a game
    attempt_game_start_request(world, user_id, lobby_id);
}

//-------------------------------------------------------------------------------------------------------------------
