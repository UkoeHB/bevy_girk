//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_request_rejected(reason: RejectionReason, request: GameRequest)
{
    tracing::warn!(?reason, ?request, "game request rejected");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_core_output_init(world: &mut World, message: GameMsg, _tick: Tick)
{
    match message
    {
        GameMsg::RequestRejected{reason, request} => handle_request_rejected(reason, request),
        GameMsg::CurrentGameState(state)          => world.syscall(state, handle_current_game_state),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_core_output_play(world: &mut World, message: GameMsg, _tick: Tick)
{
    match message
    {
        GameMsg::RequestRejected{reason, request} => handle_request_rejected(reason, request),
        GameMsg::CurrentGameState(state)          => world.syscall(state, handle_current_game_state),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_core_output_gameover(world: &mut World, message: GameMsg, _tick: Tick)
{
    match message
    {
        GameMsg::RequestRejected{reason, request} => handle_request_rejected(reason, request),
        GameMsg::CurrentGameState(state)          => world.syscall(state, handle_current_game_state),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle a message sent to the client from the game.
/// Note: this function is meant to be injected to a [`GameMessageHandler`], where it will be invoked by the client
///       framework at the start of each tick to handle incoming game messages.
pub(crate) fn try_handle_game_core_output(
    world   : &mut World,
    tick    : Tick,
    message : GameMsg
)
{
    // handle based on current client state
    match world.syscall((), get_current_client_core_state)
    {
        ClientCoreState::Init     => handle_game_core_output_init(world, message, tick),
        ClientCoreState::Play     => handle_game_core_output_play(world, message, tick),
        ClientCoreState::GameOver => handle_game_core_output_gameover(world, message, tick),
    }
}

//-------------------------------------------------------------------------------------------------------------------
