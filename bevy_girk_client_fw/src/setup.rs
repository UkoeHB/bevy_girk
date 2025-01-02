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
    world.insert_resource::<InitProgressCache>(InitProgressCache::default());
    world.insert_resource::<PingTracker>(PingTracker::new(ticks_per_sec));
}

//-------------------------------------------------------------------------------------------------------------------

/// Cleans up client framework state when transitioning away from [`ClientInstanceState::Game`].
pub(crate) fn cleanup_client_fw_state(world: &mut World)
{
    world.remove_resource::<ClientFwConfig>();
    world.remove_resource::<InitProgressCache>();
    world.remove_resource::<PingTracker>();
    world.remove_resource::<ClientRequestType>();
    world.remove_resource::<GameMessageHandler>();
}

//-------------------------------------------------------------------------------------------------------------------
