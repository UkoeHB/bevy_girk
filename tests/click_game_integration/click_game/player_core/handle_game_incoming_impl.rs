//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Use current game mode to update client mode.
fn update_client_mode(
    In(current_game_mode)       : In<GameMode>,
    client_initialization_state : Res<State<ClientInitializationState>>,
    current_client_mode         : Res<State<ClientCoreMode>>,
    mut next_client_mode        : ResMut<NextState<ClientCoreMode>>
){
    // do not update game mode if we are in the process of initializing the client
    if *client_initialization_state != ClientInitializationState::Done { return; }

    // update game mode
    let new_client_mode =
        match current_game_mode
        {
            GameMode::Init     => ClientCoreMode::Init,
            GameMode::Prep     => ClientCoreMode::Prep,
            GameMode::Play     => ClientCoreMode::Play,
            GameMode::GameOver => ClientCoreMode::GameOver,
        };

    if new_client_mode == **current_client_mode { return; }
    next_client_mode.set(new_client_mode);
    tracing::info!(?new_client_mode, "new client mode");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle current game mode.
pub(crate) fn handle_current_game_mode(In(current_game_mode): In<GameMode>, world: &mut World)
{
    syscall(world, current_game_mode, update_client_mode);
    syscall(world, (), apply_state_transition::<ClientCoreMode>);
}

//-------------------------------------------------------------------------------------------------------------------
