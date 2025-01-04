//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_replicon::prelude::ClientId;
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
){
    manager.fw_send(GameFwMsg::PingResponse(PingResponse{ request }), vis!(Client(client_id)));
}

//-------------------------------------------------------------------------------------------------------------------

/// Send game fw state to the client.
pub(crate) fn handle_game_fw_state_request(In(client_id): In<ClientId>, world: &mut World)
{
    world.syscall(client_id, notify_game_fw_state_single);
}

//-------------------------------------------------------------------------------------------------------------------
