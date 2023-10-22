//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_client_fw_command(world: &mut World, client_command: &ClientFWCommand)
{
    match client_command
    {
        ClientFWCommand::ReInitialize => syscall(world, (), handle_reinitialize_client_fw_command),
        ClientFWCommand::None         => ()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handle client controller inputs.
pub(crate) fn handle_commands(world: &mut World)
{
    let Some(client_fw_commands) = world.remove_resource::<MessageReceiver<ClientFWCommand>>() else { return; };

    while let Some(command) = client_fw_commands.try_get_next()
    {
        handle_client_fw_command(world, &command);
    }

    world.insert_resource(client_fw_commands);
}

//-------------------------------------------------------------------------------------------------------------------
