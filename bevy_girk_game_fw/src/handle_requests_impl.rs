//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Mark a client as ready.
pub(crate) fn handle_set_client_init_progress(
    In((client_id, init_progress)) : In<(ClientId, f32)>,
    mut readiness                  : ResMut<ClientReadiness>,
){
    // update client readiness
    readiness.set(client_id, Readiness::new(init_progress));
}

//-------------------------------------------------------------------------------------------------------------------

/// Send back ping response.
pub(crate) fn handle_ping_request(
    In((client_id, request)) : In<(ClientId, PingRequest)>,
    buffer                   : Res<GameMessageBuffer>
){
    buffer.fw_send(
            GameFwMsg::PingResponse(PingResponse{ request }),
            vec![InfoAccessConstraint::Targets(vec![client_id])]
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Send game fw mode to the client.
pub(crate) fn handle_game_fw_mode_request(In(client_id): In<ClientId>, world: &mut World)
{
    syscall(world, client_id, notify_game_fw_mode_single);
}

//-------------------------------------------------------------------------------------------------------------------
