//local shortcuts
use crate::{ClientFactory, ClientFactoryImpl, ClientFactoryPlugin, LocalGamePlugin};
use bevy_girk_game_instance::GameFactory;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a [`ClientFactory`] and [`LocalGameManager`] in the app.
pub struct ClientInstancePlugin
{
    factory: Arc<Mutex<Option<ClientFactory>>>,
    local_factory: Arc<Mutex<Option<GameFactory>>>,
}

impl ClientInstancePlugin
{
    /// Creates a new plugin.
    ///
    /// The `local_game_factory` allows you to set up and play local-player games without a network connection.
    pub fn new<F>(client_factory: F, local_game_factory: Option<GameFactory>) -> Self
    where
        F: ClientFactoryImpl + Send + Sync + Debug + 'static
    {
        Self{
            factory: Arc::new(Mutex::new(Some(ClientFactory::new(client_factory)))),
            local_factory: Arc::new(Mutex::new(local_game_factory)),
        }
    }
}

impl Plugin for ClientInstancePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(LocalGamePlugin{ local_factory: self.local_factory.clone() })
            .add_plugins(ClientFactoryPlugin{ factory: self.factory.clone() });
    }
}

//-------------------------------------------------------------------------------------------------------------------
