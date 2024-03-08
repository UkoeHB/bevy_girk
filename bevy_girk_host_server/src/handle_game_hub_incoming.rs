//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_hub_connection_report(world: &mut World, game_hub_id: u128, report: HostHubServerReport)
{
    match report
    {
        HostHubServerReport::Connected(_, ()) => syscall(world, game_hub_id, connected_game_hub),
        HostHubServerReport::Disconnected     => syscall(world, game_hub_id, disconnected_game_hub),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_hub_message(world: &mut World, game_hub_id: u128, game_hub_msg: HubToHostMsg)
{
    match game_hub_msg
    {
        HubToHostMsg::Capacity(capacity)             => syscall(world, (game_hub_id, capacity), hub_update_capacity),
        HubToHostMsg::Abort{id}                  => syscall(world, (game_hub_id, id), hub_abort_game),
        HubToHostMsg::GameStart{id, request, report} => syscall(world, (game_hub_id, id, request, report), hub_start_game),
        HubToHostMsg::GameOver{id, report}           => syscall(world, (game_hub_id, id, report), hub_game_over),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_game_hub_incoming(world: &mut World)
{
    // handle game hub events
    while let Some((game_hub_id, server_event)) = world.resource_mut::<HostHubServer>().next()
    {
        // handle the event
        match server_event
        {
            HostHubServerEvent::Report(report) => handle_game_hub_connection_report(world, game_hub_id, report),
            HostHubServerEvent::Msg(msg)       => handle_game_hub_message(world, game_hub_id, msg),
            HostHubServerEvent::Request(_, _)  => tracing::error!("received non-message hub val"),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
