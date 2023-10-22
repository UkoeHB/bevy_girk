//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_hub_connection_reports(world: &mut World)
{
    while let Some(connection_report) = world.resource::<HostHubServer>().next_report()
    {
        match connection_report
        {
            HostHubServerReport::Connected(game_hub_id, _, _) => syscall(world, game_hub_id, connected_game_hub),
            HostHubServerReport::Disconnected(game_hub_id) => syscall(world, game_hub_id, disconnected_game_hub),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_hub_message(world: &mut World, game_hub_id: u128, game_hub_msg: HubToHostMsg)
{
    match game_hub_msg
    {
        HubToHostMsg::Capacity(capacity)             => syscall(world, (game_hub_id, capacity), hub_update_capacity),
        HubToHostMsg::AbortGame{id}                  => syscall(world, (game_hub_id, id), hub_abort_game),
        HubToHostMsg::GameStart{id, request, report} => syscall(world, (game_hub_id, id, request, report), hub_start_game),
        HubToHostMsg::GameOver{id, report}           => syscall(world, (game_hub_id, id, report), hub_game_over),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_game_hub_incoming(world: &mut World)
{
    // handle connection reports
    handle_game_hub_connection_reports(world);

    // handle game hub messages
    while let Some((game_hub_id, game_hub_val)) = world.resource::<HostHubServer>().next_val()
    {
        // handle connection reports
        // - we do this after extracting a message because otherwise there is a race condition where
        //   a game hub can connect then send a message but the normal message gets handled before the connect message
        handle_game_hub_connection_reports(world);

        // check that the game hub is registered
        // - reason: the game hub may have been disconnected after sending the message
        if !syscall(world, game_hub_id, game_hub_is_registered)
        { tracing::warn!(game_hub_id, "ignoring message from unknown game hub"); continue; }

        // handle the message
        match game_hub_val
        {
            HostHubClientVal::Msg(msg) => handle_game_hub_message(world, game_hub_id, msg),
            _ => tracing::error!("received non-message hub val"),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
