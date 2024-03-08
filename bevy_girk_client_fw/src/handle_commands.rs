//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_reinitialize_client_fw_command(world: &mut World)
{
    syscall(world, (), reinitialize_client_fw);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles client controller inputs.
pub(crate) fn handle_commands(world: &mut World)
{
    let Some(client_fw_commands) = world.remove_resource::<Receiver<ClientFwCommand>>() else { return; };

    while let Some(command) = client_fw_commands.try_recv()
    {
        match command
        {
            ClientFwCommand::ReInitialize => syscall(world, (), handle_reinitialize_client_fw_command),
            ClientFwCommand::None         => ()
        }
    }

    world.insert_resource(client_fw_commands);
}

//-------------------------------------------------------------------------------------------------------------------
