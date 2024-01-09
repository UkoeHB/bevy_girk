//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_fw_command(world: &mut World, client_command: &ClientFwCommand)
{
    match client_command
    {
        ClientFwCommand::ReInitialize => syscall(world, (), handle_reinitialize_client_fw_command),
        ClientFwCommand::None         => ()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle client controller inputs.
pub(crate) fn handle_commands(world: &mut World)
{
    let Some(client_fw_commands) = world.remove_resource::<Receiver<ClientFwCommand>>() else { return; };

    while let Some(command) = client_fw_commands.try_recv()
    {
        handle_client_fw_command(world, &command);
    }

    world.insert_resource(client_fw_commands);
}

//-------------------------------------------------------------------------------------------------------------------
