//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_connection_reports(world: &mut World)
{
    while let Some(connection_report) = world.resource::<HostUserServer>().next_report()
    {
        match connection_report
        {
            HostHubServerReport::Connected(user_id, env_type, _) => syscall(world, (user_id, env_type), register_user),
            HostHubServerReport::Disconnected(user_id) => syscall(world, user_id, unregister_user),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_user_message(world: &mut World, user_id: u128, user_msg: UserToHostMsg)
{
    match user_msg
    {
        UserToHostMsg::LeaveLobby{ id }       => syscall(world, (user_id, id), user_leave_lobby),
        UserToHostMsg::LaunchLobbyGame{ id }  => syscall(world, (user_id, id), user_launch_lobby_game),
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
        UserToHostRequest::GetLobby(query)                => syscall(world, (token, query), user_get_lobby),
        UserToHostRequest::MakeLobby{ mcolor, pwd, data } => syscall(world, (token, mcolor, pwd, data), user_make_lobby),
        UserToHostRequest::JoinLobby{ id, mcolor, pwd }   => syscall(world, (token, id, mcolor, pwd), user_join_lobby),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_user_incoming(world: &mut World)
{
    // handle connection reports
    handle_user_connection_reports(world);

    // handle user messages
    while let Some((user_id, user_val)) = world.resource::<HostUserServer>().next_val()
    {
        // handle connection reports
        // - we do this after extracting a user message because otherwise there is a race condition where
        //   a user can connect then send a message but the normal message gets handled before the connect message
        handle_user_connection_reports(world);

        // check that the user is registered
        // - reason: the user may have been disconnected after sending the message
        if !syscall(world, user_id, user_is_registered)
        { tracing::warn!(user_id, "ignoring message from unknown user"); continue; }

        // handle the message
        match user_val
        {
            HostUserClientVal::Msg(msg)            => handle_user_message(world, user_id, msg),
            HostUserClientVal::Request(req, token) => handle_user_request(world, token, req),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
