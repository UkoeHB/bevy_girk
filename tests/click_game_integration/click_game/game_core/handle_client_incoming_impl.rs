//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_replicon_attributes::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn notify_request_rejected(
    In((client_id, request, reason)) : In<(ClientId, GameRequest, RejectionReason)>,
    mut sender                       : GameMessageSender,
    attributes                       : Res<ClientAttributes>,
){
    sender.send(
            &attributes,
            GameMsg::RequestRejected{reason, request},
            vis!(Client(client_id))
        );
}

//-------------------------------------------------------------------------------------------------------------------

//todo: consider converting this to an event, which can be responded to in a 'player click' plugin
pub(crate) fn handle_player_click_button(
    In((player_entity, _)) : In<(Entity, ())>,
    mut players            : Query<&mut PlayerScore, With<PlayerId>>,
){
    let Ok(player_score) = players.get_component_mut::<PlayerScore>(player_entity)
    else { tracing::error!("handle player click button: unknown player entity"); return; };

    player_score.into_inner().increment();
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_game_mode_request(In(client_id): In<ClientId>, world: &mut World)
{
    syscall(world, client_id, notify_game_mode_single);
}

//-------------------------------------------------------------------------------------------------------------------
