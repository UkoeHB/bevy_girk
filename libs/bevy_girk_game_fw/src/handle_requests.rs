//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_replicon::prelude::FromClient;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_client_fw_request(world: &mut World, client_id: ClientIdType, client_packet: &ClientPacket) -> bool
{
    // note: we expect this to fail very cheaply if the client message is AimedMsg::Core
    let Some(req) = deser_msg::<ClientRequest::<()>>(&client_packet.request[..]) else { return false; };
    let AimedMsg::Fw(request) = req.req else { return false; };

    tracing::trace!(?client_id, ?client_packet.send_policy, ?request, "received client fw request");

    match request
    {
        ClientFwRequest::SetInitProgress(prog) => syscall(world, (client_id, prog), handle_set_client_init_progress),
        ClientFwRequest::GetPing(req)          => syscall(world, (client_id, req),  handle_ping_request),
        ClientFwRequest::GetGameFwMode         => syscall(world, client_id,         handle_game_fw_mode_request),
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_handle_client_request(
    handler       : &ClientRequestHandler,
    world         : &mut World,
    client_id     : ClientIdType,
    client_packet : &ClientPacket
) -> bool
{
    handler.try_call(world, client_id, client_packet)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles client requests.
pub(crate) fn handle_requests(world: &mut World)
{
    let mut client_packets = world.remove_resource::<Events<FromClient<ClientPacket>>>().unwrap();
    let client_msg_handler = world.remove_resource::<ClientRequestHandler>().unwrap();

    for FromClient{ client_id, event } in client_packets.drain()
    {
        let client_id = client_id.raw() as ClientIdType;
        if try_handle_client_fw_request(world, client_id, &event) { continue; }
        if try_handle_client_request(&client_msg_handler, world, client_id, &event) { continue; }

        tracing::trace!(client_id, ?event, "failed to handle client packet");
    }

    world.insert_resource(client_packets);
    world.insert_resource(client_msg_handler);
}

//-------------------------------------------------------------------------------------------------------------------
