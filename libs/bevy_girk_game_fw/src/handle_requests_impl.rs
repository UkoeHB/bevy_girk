//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Mark a client as ready.
pub(crate) fn handle_client_init_progress_request(
    In((client_id, init_progress)) : In<(ClientIdType, f32)>,
    client_entity_map              : Res<ClientEntityMap>,
    mut client_readiness           : Query<&mut Readiness, With<ClientId>>
){
    // access client entity
    let Some(client_entity) = client_entity_map.get_entity(client_id)
    else { tracing::error!(client_id, "missing client id for init progress report"); return; };

    // update client readiness
    let Ok(mut readiness) = client_readiness.get_component_mut::<Readiness>(client_entity)
    else { tracing::error!(client_id, ?client_entity, "client entity is missing Readiness component"); return; };
    *readiness = Readiness::new(init_progress);
}

//-------------------------------------------------------------------------------------------------------------------

/// Send back ping response.
pub(crate) fn handle_ping_request(
    In((client_id, request)) : In<(ClientIdType, PingRequest)>,
    mut game_message_buffer  : ResMut<GameMessageBuffer>
){
    game_message_buffer.add_fw_msg(
            GameFwMsg::PingResponse(PingResponse{ request }),
            vec![InfoAccessConstraint::Targets(vec![client_id])],
            SendUnordered
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Send game fw mode to the client.
pub(crate) fn handle_game_fw_mode_request(In(client_id): In<ClientIdType>, world: &mut World)
{
    syscall(world, client_id, notify_game_fw_mode_single);
}

//-------------------------------------------------------------------------------------------------------------------
