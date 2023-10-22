//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Initialize miscellaneous resources.
fn setup_misc_resources(world: &mut World)
{
    world.insert_resource(GameEndFlag::default());
    world.insert_resource(GameFWTicksElapsed::default());
    world.spawn(GameInitProgressEntity::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// System for initializing the game framework.
//todo: deprecate client entities (need bevy_replicon rooms)
pub(crate) fn setup_game_fw_state(world: &mut World)
{
    // extract initializer
    let mut game_fw_initializer = world.remove_resource::<GameFWInitializer>().expect("initializer missing");

    // initialize clients
    // - client entity map
    // - client entity
    let mut client_entity_map = HashMap::<ClientIdType, Entity>::default();

    for client_state in game_fw_initializer.clients.drain(..)
    {
        // make client entity
        let mut entity_commands = world.spawn_empty();

        // save [ client id : entity ]
        client_entity_map.insert(client_state.id.id(), entity_commands.id());

        // finish client entity
        entity_commands.insert(ClientStateFull{ client_state, readiness: Readiness::default() });
    }

    world.insert_resource(ClientEntityMap::new(client_entity_map));
    world.insert_resource(GameMessageBuffer::default());

    // add misc resources
    setup_misc_resources(world);
}

//-------------------------------------------------------------------------------------------------------------------
