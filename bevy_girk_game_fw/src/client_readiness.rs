//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

/// Tracks the readiness of each client.
///
/// Readiness is used to set the [`GameInitProgress`].
///
/// This resource can be used to iterate all clients associated with this game.
#[derive(Resource)]
pub struct ClientReadiness
{
    clients: HashMap<ClientId, Readiness>,    
}

impl ClientReadiness
{
    pub(crate) fn new() -> Self
    {
        Self{ clients: HashMap::default() }
    }

    /// Sets the readiness of a client.
    pub fn set(&mut self, client: ClientId, readiness: Readiness)
    {
        let _ = self.clients.insert(client, readiness);
    }

    /// Gets the readiness of a client if available.
    pub fn get(&self, client: ClientId) -> Option<Readiness>
    {
        self.clients.get(&client).copied()
    }

    /// Iterates the readiness of tracked clients.
    pub fn iter(&self) -> impl Iterator<Item = (ClientId, Readiness)> + '_
    {
        self.clients.iter().map(|(c, r)| (*c, *r))
    }

    /// Calculates total readiness on a scale of \[0.0 - 1.0\].
    pub fn total_progress(&self) -> f32
    {
        if self.clients.len() == 0 { return 0.0; }

        let mut total = 0.0;
        for (_, readiness) in self.clients.iter()
        {
            total += readiness.value();
        }

        total / (self.clients.len() as f32)
    }

    /// Returns `true` if all clients are ready.
    pub fn all_ready(&self) -> bool
    {
        self.total_progress() >= 1.0
    }
}

//-------------------------------------------------------------------------------------------------------------------
