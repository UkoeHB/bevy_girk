//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_replicon::{prelude::FromClient, shared::{backend::connected_client::NetworkId, SERVER}};
use renet2::ClientId;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_fw_request(world: &mut World, client_id: ClientId, request: ClientFwRequest)
{
    // Note: We log the framework request in [`deserialize_client_request()`].
    match request
    {
        ClientFwRequest::SetInitProgress(prog) => world.syscall((client_id, prog), handle_set_client_init_progress),
        ClientFwRequest::GetPing(req) => world.syscall((client_id, req),  handle_ping_request),
        ClientFwRequest::GetGameFwState => world.syscall(client_id, handle_game_fw_state_request),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles client requests.
pub(crate) fn handle_requests(world: &mut World)
{
    let mut packets = world.remove_resource::<Events<FromClient<ClientPacket>>>().unwrap();
    let handler = world.remove_resource::<ClientRequestHandler>().unwrap();

    for FromClient{ client_entity, event } in packets.drain()
    {
        // Note: We assume client ids have been pre-validated by the event sender.
        let id = match client_entity == SERVER {
            true => NetworkId::new(0), // Server's client id, should only be used for testing.
            false => {
                let Some(id) = world.get::<NetworkId>(client_entity).copied() else { continue };
                id
            }
        };
        let client_id = id.get();

        match handler.try_call(world, client_id, &event)
        {
             Err(Some(fw_request)) => handle_client_fw_request(world, client_id, fw_request),
             Err(None)             => tracing::trace!(?client_id, ?event, "failed to handle client packet"),
             Ok(())                => (),
        }
    }

    world.insert_resource(packets);
    world.insert_resource(handler);
}

//-------------------------------------------------------------------------------------------------------------------
