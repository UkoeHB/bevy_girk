//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Initializer for client state entities.
/// This resource is removed during app startup.
#[derive(Resource, Default)]
pub struct GameFwInitializer
{
    pub clients: Vec<ClientState>
}

//-------------------------------------------------------------------------------------------------------------------
