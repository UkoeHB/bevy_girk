//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------------------------

/// The client id list for this game.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Deref)]
pub struct GameFwClients(HashSet<u64>);

impl GameFwClients
{
    /// Makes a new client list.
    pub fn new(clients: HashSet<u64>) -> Self 
    {
        if clients.len() == 0 { panic!("clients length must be > 0!"); }
        GameFwClients(clients)
    }
}

//-------------------------------------------------------------------------------------------------------------------
