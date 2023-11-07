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

fn get_client_id(client_config: Res<ClientFWConfig>) -> ClientIdType
{
    client_config.client_id()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_game_fw_message(world: &mut World, ser_message: Vec<u8>, ticks: Ticks) -> bool
{
    let Some(message) = deser_msg::<GameFWMsg>(&ser_message[..])
    else { tracing::warn!("failed to deserialize game framework message"); return false; };

    match message
    {
        GameFWMsg::CurrentGameFWMode(mode) => syscall(world, mode, handle_current_game_fw_mode),
        GameFWMsg::PingResponse(ping_rsp)  => syscall(world, (ticks, ping_rsp), handle_ping_response),
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_game_message(world: &mut World, ser_message: Vec<u8>, ticks: Ticks, handler: &GameMessageHandler) -> bool
{
    handler.try_call(world, ser_message, ticks)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle messages sent to the client from the game.
pub(crate) fn handle_game_incoming(world: &mut World)
{
    let game_packets     = world.remove_resource::<MessageReceiver<GamePacket>>().unwrap();
    let game_msg_handler = world.remove_resource::<GameMessageHandler>().unwrap();
    let this_client_id   = syscall(world, (), get_client_id);

    while let Some(game_packet) = game_packets.try_get_next()
    {
        // validate destination id
        if game_packet.client_id != this_client_id
        {
            tracing::warn!(game_packet.client_id, this_client_id, "received game packet destined for another client");
            continue;
        }

        // handle the packet's message
        let ticks = game_packet.message.ticks;
        let result =
            match game_packet.message.message
            {
                // try to handle with game framework message handler
                AimedMsg::Fw{bytes} => try_handle_game_fw_message(world, bytes, ticks),
                // try to handle with game message handler
                AimedMsg::Core{bytes} => try_handle_game_message(world, bytes, ticks, &game_msg_handler),
            };

        if !result { tracing::trace!("failed to handle game packet"); }
    }

    world.insert_resource(game_msg_handler);
    world.insert_resource(game_packets);
}

//-------------------------------------------------------------------------------------------------------------------
