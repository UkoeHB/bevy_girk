//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_client_fw_request(world: &mut World, client_packet: &ClientPacket) -> bool
{
    // note: we expect this to fail very cheaply if the client message is AimedMsg::Core
    let Some(message) = deser_msg::<ClientMessage::<()>>(&client_packet.message[..])
    else { tracing::trace!("failed to deserialize game framework request"); return false; };
    let AimedMsg::Fw(request) = message.message else { return false; };

    tracing::trace!(?request, "received game fw request");
    let client_id = client_packet.client_id;

    match request
    {
        GameFwRequest::ClientInitProgress(prog) => syscall(world, (client_id, prog), handle_client_init_progress_request),
        GameFwRequest::PingRequest(req)         => syscall(world, (client_id, req),  handle_ping_request),
        GameFwRequest::GameFwModeRequest        => syscall(world, client_id,         handle_game_fw_mode_request),
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_client_request(handler: &ClientRequestHandler, world: &mut World, client_packet: &ClientPacket) -> bool
{
    handler.try_call(world, client_packet)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles client requests.
pub(crate) fn handle_requests(world: &mut World)
{
    let client_packets     = world.remove_resource::<Receiver<ClientPacket>>().unwrap();
    let client_msg_handler = world.remove_resource::<ClientRequestHandler>().unwrap();

    while let Some(client_packet) = client_packets.try_recv()
    {
        if try_handle_client_fw_request(world, &client_packet) { continue; }
        if try_handle_client_request(&client_msg_handler, world, &client_packet) { continue; }

        tracing::trace!(?client_packet, "failed to handle client packet");
    }

    world.insert_resource(client_packets);
    world.insert_resource(client_msg_handler);
}

//-------------------------------------------------------------------------------------------------------------------
