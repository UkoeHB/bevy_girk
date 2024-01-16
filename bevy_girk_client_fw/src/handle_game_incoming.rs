//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_fw_message(world: &mut World, ticks: Ticks, msg: GameFwMsg)
{
    tracing::trace!(?msg, "received game fw message");

    match msg
    {
        GameFwMsg::CurrentMode(mode)      => syscall(world, mode, handle_current_game_fw_mode),
        GameFwMsg::PingResponse(ping_rsp) => syscall(world, (ticks, ping_rsp), handle_ping_response),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles messages sent to the client from the game.
pub(crate) fn handle_game_incoming(world: &mut World)
{
    let mut packets = world.remove_resource::<Events<GamePacket>>().unwrap();
    let handler = world.remove_resource::<GameMessageHandler>().unwrap();

    for packet in packets.drain()
    {
        match handler.try_call(world, &packet)
        {
             Err(Some((ticks, fw_message))) => handle_game_fw_message(world, ticks, fw_message),
             Err(None)                      => tracing::trace!(?packet, "failed to handle game packet"),
             Ok(())                         => (),
        }
    }

    world.insert_resource(handler);
    world.insert_resource(packets);
}

//-------------------------------------------------------------------------------------------------------------------