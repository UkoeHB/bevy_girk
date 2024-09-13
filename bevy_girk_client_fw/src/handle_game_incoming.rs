//local shortcuts
use crate::*;
use bevy_girk_game_fw::{GameFwMsg, GamePacket, Tick};

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_fw_message(world: &mut World, tick: Tick, msg: GameFwMsg)
{
    // Note: We log the framework message in [`deserialize_game_message()`].
    match msg
    {
        GameFwMsg::CurrentState(state)    => syscall(world, state, handle_current_game_fw_state),
        GameFwMsg::PingResponse(ping_rsp) => syscall(world, (tick, ping_rsp), handle_ping_response),
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
            Err(Some((tick, fw_message))) => handle_game_fw_message(world, tick, fw_message),
            Err(None)                     => tracing::trace!(?packet, "failed to handle game packet"),
            Ok(())                        => (),
        }
    }

    world.insert_resource(handler);
    world.insert_resource(packets);
}

//-------------------------------------------------------------------------------------------------------------------
