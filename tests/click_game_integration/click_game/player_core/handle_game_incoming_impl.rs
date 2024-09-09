//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Use current game state to update client state.
fn update_client_state(
    In(current_game_state)      : In<GameState>,
    client_initialization_state : Res<State<ClientInitializationState>>,
    current_client_state        : Res<State<ClientCoreState>>,
    mut next_client_state       : ResMut<NextState<ClientCoreState>>
){
    // do not update game state if we are in the process of initializing the client
    if *client_initialization_state != ClientInitializationState::Done { return; }

    // update game state
    let new_client_state =
        match current_game_state
        {
            GameState::Init     => ClientCoreState::Init,
            GameState::Prep     => ClientCoreState::Prep,
            GameState::Play     => ClientCoreState::Play,
            GameState::GameOver => ClientCoreState::GameOver,
        };

    if new_client_state == **current_client_state { return; }
    next_client_state.set(new_client_state);
    tracing::info!(?new_client_state, "new client state");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle current game state.
pub(crate) fn handle_current_game_state(In(current_game_state): In<GameState>, world: &mut World)
{
    syscall(world, current_game_state, update_client_state);
    //todo: this is heavy-handed, re-evaluate mode-change handling
    // - ClientCoreState
    let _ = world.try_run_schedule(StateTransition);
}

//-------------------------------------------------------------------------------------------------------------------
