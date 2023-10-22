//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_reinitialize_client_fw_command(world: &mut World)
{
    syscall(world, (), reinitialize_client_fw);
}

//-------------------------------------------------------------------------------------------------------------------
