//local shortcuts
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_kot_utils::*;

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn process_player_inputs<F>(world: &mut World, handler: F)
where
    F: Fn(&mut World, &PlayerInput)
{
    let Some(player_inputs) = world.remove_resource::<Receiver<PlayerInput>>() else { return; };

    while let Some(input) = player_inputs.try_recv()
    {
        handler(world, &input);
    }

    world.insert_resource(player_inputs);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_player_input_init(_world: &mut World, player_input: &PlayerInput)
{
    let PlayerInput::Init(init_input) = player_input else { return; };

    match init_input
    {
        PlayerInputInit::None => ()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_player_input_prep(_world: &mut World, player_input: &PlayerInput)
{
    let PlayerInput::Prep(prep_input) = player_input else { return; };

    match prep_input
    {
        PlayerInputPrep::None => ()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_player_input_play(world: &mut World, player_input: &PlayerInput)
{
    let PlayerInput::Play(play_input) = player_input else { return; };

    match play_input
    {
        PlayerInputPlay::ClickButton => syscall(world, GameRequest::ClickButton, send_game_request),
        PlayerInputPlay::None        => ()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_player_input_gameover(_world: &mut World, player_input: &PlayerInput)
{
    let PlayerInput::GameOver(gameover_input) = player_input else { return; };

    match gameover_input
    {
        PlayerInputGameOver::None => ()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle player inputs for ClientCoreMode::Init.
pub(crate) fn handle_player_inputs_init(world: &mut World)
{
    process_player_inputs(world, | world, player_input | handle_player_input_init(world, player_input));
}

//-------------------------------------------------------------------------------------------------------------------

/// Handle player inputs for ClientCoreMode::Prep.
pub(crate) fn handle_player_inputs_prep(world: &mut World)
{
    process_player_inputs(world, | world, player_input | handle_player_input_prep(world, player_input));
}

//-------------------------------------------------------------------------------------------------------------------

/// Handle player inputs for ClientCoreMode::Play.
pub(crate) fn handle_player_inputs_play(world: &mut World)
{
    process_player_inputs(world, | world, player_input | handle_player_input_play(world, player_input));
}

//-------------------------------------------------------------------------------------------------------------------

/// Handle player inputs for ClientCoreMode::GameOver.
pub(crate) fn handle_player_inputs_gameover(world: &mut World)
{
    process_player_inputs(world, | world, player_input | handle_player_input_gameover(world, player_input));
}

//-------------------------------------------------------------------------------------------------------------------
