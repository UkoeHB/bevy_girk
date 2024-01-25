//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Initialize miscellaneous resources.
fn setup_misc_resources(world: &mut World)
{
    world.insert_resource(GameEndFlag::default());
    world.insert_resource(GameFwTick::default());
    world.insert_resource(GameFwPreEndTick::default());
    world.spawn(GameInitProgressEntity::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// System for initializing the game framework.
//todo: deprecate client entities (need bevy_replicon rooms)
pub(crate) fn setup_game_fw_state(world: &mut World)
{
    // extract initializer
    let mut game_fw_initializer = world.remove_resource::<GameFwInitializer>().expect("initializer missing");

    // initialize clients
    // - client entity map
    // - client entity
    let mut client_readiness = ClientReadiness::new();
    let mut client_entity_map = HashMap::<ClientIdType, Entity>::default();

    for client_state in game_fw_initializer.clients.drain(..)
    {
        // make client entity
        let mut entity_commands = world.spawn_empty();

        // save [ client id : entity ]
        client_entity_map.insert(client_state.id.id(), entity_commands.id());

        // finish client entity
        client_readiness.set(client_state.id, Readiness::default());
        entity_commands.insert(client_state);
    }

    world.insert_resource(client_readiness);
    world.insert_resource(crate::ClientEntityMap::new(client_entity_map));

    // add misc resources
    setup_misc_resources(world);

    // spawn empty replicated entity to ensure replication initializes in the first world tick
    // - This way clients can reliably check for the first replication message as part of initialization progress
    //   even if the game does not replicate entities at startup.
    world.spawn(Replication);
}

//-------------------------------------------------------------------------------------------------------------------
