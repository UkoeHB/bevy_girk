//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initializes the client framework state.
pub(crate) fn setup_client_fw_state(mut commands: Commands, client_config: Res<ClientFwConfig>)
{
    commands.insert_resource::<InitializationProgressCache>(InitializationProgressCache::default());
    commands.insert_resource::<PingTracker>(PingTracker::new(client_config.ticks_per_sec()));
}

//-------------------------------------------------------------------------------------------------------------------
