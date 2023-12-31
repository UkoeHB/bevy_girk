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

fn try_handle_game_fw_request(world: &mut World, ser_message: Vec<u8>, client_id: ClientIdType) -> bool
{
    let Some(request) = deser_msg::<GameFWRequest>(&ser_message[..])
    else { tracing::trace!("failed to deserialize game framework request"); return false; };

    tracing::trace!(?request, "received game fw request");
    match request
    {
        GameFWRequest::ClientInitProgress(prog) => syscall(world, (client_id, prog), handle_client_init_progress_request),
        GameFWRequest::PingRequest(req)         => syscall(world, (client_id, req),  handle_ping_request),
        GameFWRequest::GameFWModeRequest        => syscall(world, client_id,         handle_game_fw_mode_request),
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_game_request(
    world       : &mut World,
    ser_message : Vec<u8>,
    client_id   : ClientIdType,
    handler     : &ClientMessageHandler
) -> bool
{
    tracing::trace!("received client message");  //todo: use generic and deserialize client message here, then log it
    handler.try_call(world, ser_message, client_id)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle game framework requests.
pub(crate) fn handle_requests(world: &mut World)
{
    let client_packets     = world.remove_resource::<Receiver<ClientPacket>>().unwrap();
    let client_msg_handler = world.remove_resource::<ClientMessageHandler>().unwrap();

    while let Some(client_packet) = client_packets.try_recv()
    {
        // handle the packet's message
        let client_id = client_packet.client_id;
        let result =
            match client_packet.message.message
            {
                // try to handle with framework request handler
                AimedMsg::Fw{ bytes } => try_handle_game_fw_request(world, bytes, client_id),
                // try to handle with core request handler
                AimedMsg::Core{ bytes } => try_handle_game_request(world, bytes, client_id, &client_msg_handler),
            };

        if !result { tracing::trace!(client_packet.client_id, "failed to handle client packet"); }
    }

    world.insert_resource(client_packets);
    world.insert_resource(client_msg_handler);
}

//-------------------------------------------------------------------------------------------------------------------
