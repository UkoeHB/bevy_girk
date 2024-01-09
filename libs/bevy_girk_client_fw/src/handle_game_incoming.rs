//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_game_fw_message(world: &mut World, game_packet: &GamePacket) -> bool
{
    // note: we expect this to fail very cheaply if the game message is AimedMsg::Core
    let Some(message) = deser_msg::<GameMessage::<()>>(&game_packet.message[..])
    else { tracing::trace!("failed to deserialize game fw message"); return false; };
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
    let game_packets     = world.remove_resource::<Receiver<GamePacket>>().unwrap();
    let game_msg_handler = world.remove_resource::<GameMessageHandler>().unwrap();
    let this_client_id   = world.resource::<ClientFwConfig>().client_id();

    while let Some(game_packet) = game_packets.try_recv()
    {
        // validate destination id
        if game_packet.client_id != this_client_id
        {
            tracing::error!(game_packet.client_id, this_client_id, "received game packet destined for another client");
            continue;
        }

        // handle the packet's message
        if try_handle_game_fw_message(world, &game_packet) { continue; }
        if try_handle_game_message(&game_msg_handler, world, &game_packet) { continue; }

        tracing::error!(?game_packet, "failed to handle game packet");
    }

    world.insert_resource(game_msg_handler);
    world.insert_resource(game_packets);
}

//-------------------------------------------------------------------------------------------------------------------
