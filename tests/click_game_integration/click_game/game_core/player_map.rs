//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct PlayerMap
{
    /// [ client id : player entity ]
    client_to_entity: HashMap<ClientIdType, Entity>,
}

impl PlayerMap
{
    pub fn new(
        client_to_entity: HashMap<ClientIdType, Entity>,
    ) -> PlayerMap
    {
        PlayerMap{ client_to_entity }
    }

    pub fn is_player(&self, client_id: ClientIdType) -> bool
    {
        self.client_to_entity.contains_key(&client_id)
    }

    pub fn client_to_entity(&self, player_id: ClientIdType) -> Result<Entity, ()>
    {
        self.client_to_entity.get(&player_id).ok_or(()).map(|e| *e)
    }
}

//-------------------------------------------------------------------------------------------------------------------
