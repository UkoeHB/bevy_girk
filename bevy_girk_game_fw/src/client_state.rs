//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

/// Client id type re-exports `renet`'s client id.
pub type ClientId = renet::ClientId;

/// Client identifier component.
#[derive(Component, Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClientIdComponent
{
    /// Client id
    id: ClientId
}

impl ClientIdComponent
{
    pub fn new(id: ClientId) -> ClientIdComponent
    {
        ClientIdComponent{ id: id }
    }

    pub fn id(&self) -> ClientId { self.id }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client state bundle.
#[derive(Bundle, Clone, Debug)]
pub struct ClientState
{
    /// Client id
    pub id: ClientIdComponent,

    /// Information access rights of this client (todo: replace with room membership)
    pub access_rights: InfoAccessRights,
}

//-------------------------------------------------------------------------------------------------------------------

/// Map: [ client id : client entity ]
#[derive(Resource)]
pub struct ClientEntityMap
{
    map: HashMap<ClientId, Entity>
}

impl ClientEntityMap
{
    pub fn new(map: HashMap<ClientId, Entity>) -> ClientEntityMap
    {
        ClientEntityMap{ map }
    }

    pub fn get_entity(&self, client_id: ClientId) -> Option<Entity>
    {
        self.map.get(&client_id).copied()
    }

    pub fn len(&self) -> usize { self.map.len() }
}

//-------------------------------------------------------------------------------------------------------------------
