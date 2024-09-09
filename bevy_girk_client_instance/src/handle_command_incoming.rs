//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_command_abort(
    runner_state : Res<ClientRunnerState>,
    mut app_exit : EventWriter<AppExit>,
){
    // send client aborted report
    if let Err(_) = runner_state.report_sender.send(ClientInstanceReport::Aborted(runner_state.game_id))
    {
        tracing::error!(runner_state.game_id, "failed sending client abort message");
        panic!("failed sending client abort message");  //panic so the client instance result is 'error'
    }

    // exit the client
    // WARNING: we assume sending AppExit guarantees the app will clean up all its resources and shut down; if that
    //          guarantee does not hold, we should panic instead
    app_exit.send(AppExit::Success);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_command_connect(In(token): In<ServerConnectToken>, mut commands: Commands, runner: Res<ClientRunnerState>)
{
    let connect_pack = RenetClientConnectPack::new(runner.protocol_id, token).unwrap();
    commands.insert_resource(connect_pack);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_command(world: &mut World, command: ClientInstanceCommand)
{
    match command
    {
        ClientInstanceCommand::Abort          => syscall(world, (), handle_command_abort),
        ClientInstanceCommand::Connect(token) => syscall(world, token, handle_command_connect),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns `false` when aborted.
pub(crate) fn handle_command_incoming(world: &mut World)
{
    // handle client instance commands
    while let Some(command) = world.resource_mut::<ClientRunnerState>().command_receiver.try_recv()
    {
        // handle the command
        handle_command(world, command);
    }
}

//-------------------------------------------------------------------------------------------------------------------
