//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_connection_report(world: &mut World, user_id: u128, report: HostUserServerReport)
{
    match report
    {
        HostHubServerReport::Connected(env_type, ()) => syscall(world, (user_id, env_type), register_user),
        HostHubServerReport::Disconnected            => syscall(world, user_id, unregister_user),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_message(world: &mut World, user_id: u128, user_msg: UserToHostMsg)
{
    match user_msg
    {
        UserToHostMsg::NackPendingLobby{ id } => syscall(world, (user_id, id), user_nack_pending_lobby),
        UserToHostMsg::AckPendingLobby{ id }  => syscall(world, (user_id, id), user_ack_pending_lobby),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_request(world: &mut World, token: bevy_simplenet::RequestToken, user_req: UserToHostRequest)
{
    match user_req
    {
        UserToHostRequest::ResetLobby                     => syscall(world, token, user_reset_lobby),
        UserToHostRequest::LobbySearch(query)             => syscall(world, (token, query), user_get_lobby),
        UserToHostRequest::MakeLobby{ mcolor, pwd, data } => syscall(world, (token, mcolor, pwd, data), user_make_lobby),
        UserToHostRequest::JoinLobby{ id, mcolor, pwd }   => syscall(world, (token, id, mcolor, pwd), user_join_lobby),
        UserToHostRequest::LeaveLobby{ id }               => syscall(world, (token, id), user_leave_lobby),
        UserToHostRequest::LaunchLobbyGame{ id }          => syscall(world, (token, id), user_launch_lobby_game),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_user_incoming(world: &mut World)
{
    // handle user events
    while let Some((user_id, server_event)) = world.resource::<HostUserServer>().next()
    {
        // handle the event
        match server_event
        {
            HostUserServerEvent::Report(report)      => handle_user_connection_report(world, user_id, report),
            HostUserServerEvent::Msg(msg)            => handle_user_message(world, user_id, msg),
            HostUserServerEvent::Request(req, token) => handle_user_request(world, token, req),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
