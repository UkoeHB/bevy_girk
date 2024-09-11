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
pub trait ClientFactoryImpl: Debug + Send + Sync + 'static
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
    factory: Box<dyn ClientFactoryImpl>
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

pub(crate) struct ClientFactoryPlugin
{
    pub(crate) factory: Arc<Mutex<Option<Box<dyn ClientFactoryImpl>>>>,
}

impl Plugin for ClientFactoryPlugin
{
    fn build(&self, app: &mut App)
    {
        let mut factory = self.factory
            .lock()
            .expect("ClientInstancePlugin should only be built once")
            .take()
            .expect("ClientInstancePlugin should only be built once");
        factory.add_plugins(app);
        app.insert_resource(ClientFactory{ factory });
    }
}

//-------------------------------------------------------------------------------------------------------------------
