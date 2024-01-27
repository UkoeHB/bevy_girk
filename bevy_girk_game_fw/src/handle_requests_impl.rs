//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_replicon_attributes::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Update client readiness.
pub(crate) fn handle_set_client_init_progress(
    In((client_id, init_progress)) : In<(ClientId, f32)>,
    mut readiness                  : ResMut<ClientReadiness>,
){
    readiness.set(client_id, Readiness::new(init_progress));
}

//-------------------------------------------------------------------------------------------------------------------

/// Send back ping response.
pub(crate) fn handle_ping_request(
    In((client_id, request)) : In<(ClientId, PingRequest)>,
    mut manager              : GameMessageSender,
    attributes               : Res<ClientAttributes>,
){
    manager.fw_send(&attributes, GameFwMsg::PingResponse(PingResponse{ request }), vis!(Client(client_id)));
}

//-------------------------------------------------------------------------------------------------------------------

/// Send game fw mode to the client.
pub(crate) fn handle_game_fw_mode_request(In(client_id): In<ClientId>, world: &mut World)
{
    syscall(world, client_id, notify_game_fw_mode_single);
}

//-------------------------------------------------------------------------------------------------------------------
