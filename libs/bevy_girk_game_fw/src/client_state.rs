//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

/// Client id type.
pub use u16 as ClientIdType;

/// Client identifier component.
#[derive(Component, Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct ClientId
{
    /// Client id
    id: ClientIdType
}

impl ClientId
{
    pub fn new(id: ClientIdType) -> ClientId
    {
        ClientId{ id: id }
    }

    pub fn id(&self) -> ClientIdType { self.id }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client state bundle.
#[derive(Bundle, Clone, Default)]
pub struct ClientState
{
    /// Client id
    pub id: ClientId,

    /// Information access rights of this client (todo: replace with room membership)
    pub access_rights: InfoAccessRights,
}

//-------------------------------------------------------------------------------------------------------------------

/// Client state with additional components that don't need to be initialized.
#[derive(Bundle)]
pub struct ClientStateFull
{
    /// Client state
    pub client_state: ClientState,

    /// Readiness tracker
    pub readiness: Readiness,
}

//-------------------------------------------------------------------------------------------------------------------

/// Map: [ client id : client entity ]
#[derive(Resource)]
pub struct ClientEntityMap
{
    map: HashMap<ClientIdType, Entity>
}

impl ClientEntityMap
{
    pub fn new(map: HashMap<ClientIdType, Entity>) -> ClientEntityMap
    {
        ClientEntityMap{ map }
    }

    pub fn get_entity(&self, client_id: ClientIdType) -> Option<Entity>
    {
        self.map.get(&client_id).copied()
    }

    pub fn len(&self) -> usize { self.map.len() }
}

//-------------------------------------------------------------------------------------------------------------------
