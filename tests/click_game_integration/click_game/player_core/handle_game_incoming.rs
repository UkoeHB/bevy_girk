//local shortcuts
use bevy_girk_client_fw::*;
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
        GameMsg::CurrentGameMode(mode)            => syscall(world, mode, handle_current_game_mode),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_core_output_prep(world: &mut World, message: GameMsg, _tick: Tick)
{
    match message
    {
        GameMsg::RequestRejected{reason, request} => handle_request_rejected(reason, request),
        GameMsg::CurrentGameMode(mode)            => syscall(world, mode, handle_current_game_mode),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_core_output_play(world: &mut World, message: GameMsg, _tick: Tick)
{
    match message
    {
        GameMsg::RequestRejected{reason, request} => handle_request_rejected(reason, request),
        GameMsg::CurrentGameMode(mode)            => syscall(world, mode, handle_current_game_mode),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_game_core_output_gameover(world: &mut World, message: GameMsg, _tick: Tick)
{
    match message
    {
        GameMsg::RequestRejected{reason, request} => handle_request_rejected(reason, request),
        GameMsg::CurrentGameMode(mode)            => syscall(world, mode, handle_current_game_mode),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle a message sent to the client from the game.
/// Note: this function is meant to be injected to a [`GameMessageHandler`], where it will be invoked by the client
///       framework at the start of each tick to handle incoming game messages.
pub(crate) fn try_handle_game_core_output(
    world       : &mut World,
    game_packet : &GamePacket
) -> Result<(), Option<(Tick, GameFwMsg)>>
{
    let (ticks, message) = deserialize_game_message(game_packet)?;

    // handle based on current client mode
    match syscall(world, (), get_current_client_core_mode)
    {
        ClientCoreMode::Init     => handle_game_core_output_init(world, message, ticks),
        ClientCoreMode::Prep     => handle_game_core_output_prep(world, message, ticks),
        ClientCoreMode::Play     => handle_game_core_output_play(world, message, ticks),
        ClientCoreMode::GameOver => handle_game_core_output_gameover(world, message, ticks),
    }

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
