//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_command_abort(
    runner_state : Res<GameRunnerState>,
    mut app_exit : EventWriter<AppExit>,
){
    // send game aborted report
    if let Err(_) = runner_state.report_sender.send(GameInstanceReport::GameAborted(runner_state.game_id))
    {
        tracing::error!(runner_state.game_id, "failed sending game abort message");
        app_exit.send(AppExit::from_code(65));
    }

    // exit the game
    // WARNING: we assume sending AppExit guarantees the app will clean up all its resources and shut down; if that
    //          guarantee does not hold, we should panic instead
    app_exit.send(AppExit::from_code(66));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_command(world: &mut World, command: GameInstanceCommand)
{
    match command
    {
        GameInstanceCommand::Abort => world.syscall((), handle_command_abort),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_command_incoming(world: &mut World)
{
    // handle game instance commands
    while let Some(command) = world.resource_mut::<GameRunnerState>().command_receiver.try_recv()
    {
        // handle the command
        handle_command(world, command);
    }
}

//-------------------------------------------------------------------------------------------------------------------
