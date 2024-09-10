//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::{fmt::Debug, sync::{Arc, Mutex}};

//-------------------------------------------------------------------------------------------------------------------

/// Trait for client factory implementations.
pub trait ClientFactoryImpl: Debug
{
    /// Sets up the current client app so it can play games.
    fn add_plugins(&mut self, app: &mut App);

    /// Sets up a game instance in the current client world.
    fn setup_game(&mut self, world: &mut World, token: ServerConnectToken, start_info: GameStartInfo);
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps a client factory implementation.
#[derive(Debug, Resource)]
pub struct ClientFactory
{
    factory: Box<dyn ClientFactoryImpl + Send + Sync>
}

impl ClientFactory
{
    /// Sets up a client in the current world.
    pub fn setup_game(&mut self, world: &mut World, token: ServerConnectToken, start_info: GameStartInfo)
    {
        self.factory.setup_game(world, token, start_info);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a [`ClientFactory`] in the app.
#[derive(Debug)]
pub struct ClientFactoryPlugin
{
    factory: Arc<Mutex<Option<Box<dyn ClientFactoryImpl + Send + Sync>>>>
}

impl ClientFactoryPlugin
{
    /// Create a new client factory.
    pub fn new<F: ClientFactoryImpl + Send + Sync + Debug + 'static>(factory_impl: F) -> Self
    {
        Self { factory: Arc::new(Mutex::new(Some(Box::new(factory_impl)))) }
    }
}

impl Plugin for ClientFactoryPlugin
{
    fn build(&self, app: &mut App)
    {
        let mut factory = self.factory
            .lock()
            .expect("ClientFactoryPlugin should only be built once")
            .take()
            .expect("ClientFactoryPlugin should only be built once");
        app.add_event::<ClientInstanceReport>();
        factory.add_plugins(app);
        app.insert_resource(ClientFactory{ factory });
    }
}

//-------------------------------------------------------------------------------------------------------------------
