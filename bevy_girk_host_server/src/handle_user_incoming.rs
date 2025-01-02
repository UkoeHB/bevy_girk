//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_connection_report(world: &mut World, user_id: u128, report: HostUserServerReport)
{
    match report
    {
        HostUserServerReport::Connected(env_type, connect_msg) => world.syscall(
            (user_id, UserInfo::new(env_type, connect_msg.connection_type)),
            register_user
        ),
        HostUserServerReport::Disconnected => world.syscall(user_id, unregister_user),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_message(world: &mut World, user_id: u128, user_msg: UserToHostMsg)
{
    match user_msg
    {
        UserToHostMsg::NackPendingLobby{ id } => world.syscall((user_id, id), user_nack_pending_lobby),
        UserToHostMsg::AckPendingLobby{ id }  => world.syscall((user_id, id), user_ack_pending_lobby),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_request(world: &mut World, token: bevy_simplenet::RequestToken, user_req: UserToHostRequest)
{
    match user_req
    {
        UserToHostRequest::LobbySearch(query)             => world.syscall((token, query), user_get_lobby),
        UserToHostRequest::MakeLobby{ mcolor, pwd, data } => world.syscall((token, mcolor, pwd, data), user_make_lobby),
        UserToHostRequest::JoinLobby{ id, mcolor, pwd }   => world.syscall((token, id, mcolor, pwd), user_join_lobby),
        UserToHostRequest::LeaveLobby{ id }               => world.syscall((token, id), user_leave_lobby),
        UserToHostRequest::LaunchLobbyGame{ id }          => world.syscall((token, id), user_launch_lobby_game),
        UserToHostRequest::GetConnectToken{ id }          => world.syscall((token, id), user_get_connect_token),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_user_incoming(world: &mut World)
{
    // handle user events
    while let Some((user_id, server_event)) = world.resource_mut::<HostUserServer>().next()
    {
        // handle the event
        match server_event
        {
            HostUserServerEvent::Report(report)      => handle_user_connection_report(world, user_id, report),
            HostUserServerEvent::Msg(msg)            => handle_user_message(world, user_id, msg),
            HostUserServerEvent::Request(token, req) => handle_user_request(world, token, req),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
