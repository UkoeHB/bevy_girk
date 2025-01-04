//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_replicon::prelude::ClientId;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn player_syscall<A, S, Marker>(world: &mut World, req: GameRequest, id: ClientId, arg: A, sys: S)
where
    A: SystemInput + Send + Sync + 'static,
    S: IntoSystem<In<(Entity, A)>, (), Marker> + Send + Sync + 'static,
{
    match world.resource::<PlayerMap>().client_to_entity(id)
    {
        Ok(player_entity) => world.syscall((player_entity, arg), sys),
        Err(err) =>
        {
            tracing::trace!(?id, ?err, "player syscall failed, client is not player");
            world.syscall((id, req, RejectionReason::Invalid), notify_request_rejected);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_input_init(world: &mut World, req: GameRequest, id: ClientId)
{
    match req
    {
        GameRequest::GameStateRequest => world.syscall(id, handle_game_state_request),
        GameRequest::ClickButton      => world.syscall((id, req, RejectionReason::StateMismatch), notify_request_rejected),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_input_play(world: &mut World, req: GameRequest, id: ClientId)
{
    match req
    {
        GameRequest::GameStateRequest => world.syscall(id, handle_game_state_request),
        GameRequest::ClickButton      => player_syscall(world, req, id, (), handle_player_click_button),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_input_gameover(world: &mut World, req: GameRequest, id: ClientId)
{
    match req
    {
        GameRequest::GameStateRequest => world.syscall(id, handle_game_state_request),
        GameRequest::ClickButton      => world.syscall((id, req, RejectionReason::StateMismatch), notify_request_rejected),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle a message sent to the game from a client.
/// Note: this function is meant to be injected to a [`ClientRequestHandler`].
pub(crate) fn try_handle_game_core_input(
    world     : &mut World,
    client_id : ClientId,
    request   : GameRequest
)
{
    match world.syscall((), get_current_game_state)
    {
        GameState::Init     => handle_client_input_init(world, request, client_id),
        GameState::Play     => handle_client_input_play(world, request, client_id),
        GameState::GameOver => handle_client_input_gameover(world, request, client_id),
    }
}

//-------------------------------------------------------------------------------------------------------------------
