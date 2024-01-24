//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_replicon::prelude::FromClient;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_fw_request(world: &mut World, client_id: ClientIdType, request: ClientFwRequest)
{
    // Note: We log the framework request in [`deserialize_client_request()`].

    match request
    {
        ClientFwRequest::SetInitProgress(prog) => syscall(world, (client_id, prog), handle_set_client_init_progress),
        ClientFwRequest::GetPing(req)          => syscall(world, (client_id, req),  handle_ping_request),
        ClientFwRequest::GetGameFwMode         => syscall(world, client_id,         handle_game_fw_mode_request),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles client requests.
pub(crate) fn handle_requests(world: &mut World)
{
    let mut packets = world.remove_resource::<Events<FromClient<ClientPacket>>>().unwrap();
    let handler = world.remove_resource::<ClientRequestHandler>().unwrap();

    for FromClient{ client_id, event } in packets.drain()
    {
        let client_id = client_id.raw() as ClientIdType;

        match handler.try_call(world, client_id, &event)
        {
             Err(Some(fw_request)) => handle_client_fw_request(world, client_id, fw_request),
             Err(None)             => tracing::trace!(client_id, ?event, "failed to handle client packet"),
             Ok(())                => (),
        }
    }

    world.insert_resource(packets);
    world.insert_resource(handler);
}

//-------------------------------------------------------------------------------------------------------------------
