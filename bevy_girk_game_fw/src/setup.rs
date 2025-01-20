//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// System for initializing the game framework.
pub(crate) fn setup_game_fw_state(world: &mut World)
{
    world.insert_resource(GameEndFlag::default());
    world.insert_resource(GameFwTick::default());
    world.insert_resource(GameFwPreEndTick::default());

    // Spawning this ensures replication initializes in the first world tick.
    // - This way clients can reliably check for the first replication message as part of initialization progress
    //   even if the game does not replicate entities at startup.
    world.spawn(GameInitProgressEntity::default());

    // Initialize readiness for each client.
    let mut readiness = ClientReadiness::new();

    for client in world.resource::<GameFwClients>().iter()
    {
        readiness.set(*client, Readiness::default());
    }

    world.insert_resource(readiness);
}

//-------------------------------------------------------------------------------------------------------------------
