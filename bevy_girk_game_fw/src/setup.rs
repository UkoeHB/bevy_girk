//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// System for initializing the game framework.
pub(crate) fn setup_game_fw_state(clients: Res<GameFwClients>, mut commands: Commands)
{
    commands.insert_resource(GameEndFlag::default());
    commands.insert_resource(GameFwTick::default());
    commands.insert_resource(GameFwPreEndTick::default());
    commands.spawn(GameInitProgressEntity::default());

    // Spawn empty replicated entity to ensure replication initializes in the first world tick.
    // - This way clients can reliably check for the first replication message as part of initialization progress
    //   even if the game does not replicate entities at startup.
    commands.spawn(Replication);

    // Initialize readiness for each client.
    let mut readiness = ClientReadiness::new();

    for client in clients.iter()
    {
        readiness.set(*client, Readiness::default());
    }

    commands.insert_resource(readiness);
}

//-------------------------------------------------------------------------------------------------------------------
