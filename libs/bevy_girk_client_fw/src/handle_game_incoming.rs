//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_game_fw_message(world: &mut World, game_packet: &GamePacket) -> bool
{
    // note: we expect this to fail very cheaply if the game message is AimedMsg::Core
    let Some(message) = deser_msg::<GameMessage::<()>>(&game_packet.message[..]) else { return false; };
    let AimedMsg::Fw(msg) = message.msg else { return false; };

    tracing::trace!(?msg, "received game fw message");
    let ticks = message.ticks;

    match msg
    {
        GameFwMsg::CurrentGameFwMode(mode) => syscall(world, mode, handle_current_game_fw_mode),
        GameFwMsg::PingResponse(ping_rsp)  => syscall(world, (ticks, ping_rsp), handle_ping_response),
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_game_message(handler: &GameMessageHandler, world: &mut World, game_packet: &GamePacket) -> bool
{
    handler.try_call(world, game_packet)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles messages sent to the client from the game.
pub(crate) fn handle_game_incoming(world: &mut World)
{
    let mut game_packets = world.remove_resource::<Events<GamePacket>>().unwrap();
    let game_msg_handler = world.remove_resource::<GameMessageHandler>().unwrap();

    for game_packet in game_packets.drain()
    {
        // handle the packet's message
        if try_handle_game_fw_message(world, &game_packet) { continue; }
        if try_handle_game_message(&game_msg_handler, world, &game_packet) { continue; }

        tracing::error!(?game_packet, "failed to handle game packet");
    }

    world.insert_resource(game_msg_handler);
    world.insert_resource(game_packets);
}

//-------------------------------------------------------------------------------------------------------------------
