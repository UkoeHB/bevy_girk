//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initializes the client framework state.
pub(crate) fn setup_client_fw_state(world: &mut World)
{
    let ticks_per_sec = world.resource::<ClientFwConfig>().ticks_per_sec();
    world.insert_resource::<InitializationProgressCache>(InitializationProgressCache::default());
    world.insert_resource::<PingTracker>(PingTracker::new(ticks_per_sec));
}

//-------------------------------------------------------------------------------------------------------------------
