//local shortcuts
use bevy_girk_game_instance::GameStartInfo;
use bevy_girk_utils::deser_msg;

//third-party shortcuts
use bevy::prelude::*;
use renet2_setup::ServerConnectToken;
use serde::Deserialize;

//standard shortcuts
use std::any::type_name;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::ClientInstanceReport;

//-------------------------------------------------------------------------------------------------------------------

enum ClientFactoryCommand<'a>
{
    AddPlugins(&'a mut App),
    SetupGame(&'a mut World, ServerConnectToken, GameStartInfo),
}

//-------------------------------------------------------------------------------------------------------------------

/// Information used by a client to connect to a game.
#[derive(Default, Debug, Clone)]
pub struct ClientStartInfo<T>
{
    /// The game id.
    pub game_id: u64,
    /// User's server id.
    pub user_id: u128,
    /// User's client id within the game.
    pub client_id: u64,
    /// Data needed for a user to start a game.
    pub data: T,
}

impl<T: for<'de> Deserialize<'de>> ClientStartInfo<T>
{
    /// Returns `None` if deserializing the start data failed.
    pub fn new(start_info: GameStartInfo) -> Option<Self>
    {
        let data = deser_msg(&start_info.serialized_start_data)?;
        Some(Self{
            game_id: start_info.game_id,
            user_id: start_info.user_id,
            client_id: start_info.client_id,
            data
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for client factory implementations.
pub trait ClientFactoryImpl: Debug + Send + Sync + 'static
{
    type Data: for<'de> Deserialize<'de>;

    /// Sets up the current client app so it can play games.
    fn add_plugins(&mut self, app: &mut App);

    /// Sets up a game instance in the current client world.
    fn setup_game(&mut self, world: &mut World, token: ServerConnectToken, start_info: ClientStartInfo<Self::Data>);
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps a client factory implementation.
///
/// Inserted to the app via [`ClientInstancePlugin`].
#[derive(Resource)]
pub struct ClientFactory
{
    callback: Box<dyn FnMut(ClientFactoryCommand) + Send + Sync + 'static>
}

impl ClientFactory
{
    pub fn new<T: ClientFactoryImpl>(mut factory: T) -> Self
    {
        let callback = move |cmd: ClientFactoryCommand| {
            match cmd {
                ClientFactoryCommand::AddPlugins(app) => factory.add_plugins(app),
                ClientFactoryCommand::SetupGame(world, token, start_info) => {
                    let Some(start_info) = ClientStartInfo::new(start_info) else {
                        tracing::error!("failed deserializing client start info {} for {}",
                            type_name::<T::Data>(), type_name::<T>());
                        return;
                    };
                    factory.setup_game(world, token, start_info);
                }
            }
        };

        Self{ callback: Box::new(callback) }
    }

    /// Sets up a client in the current world.
    pub fn add_plugins(&mut self, app: &mut App)
    {
        // Add this event here instead of the ClientInstancePlugin for easier testing.
        app.add_event::<ClientInstanceReport>();

        (self.callback)(ClientFactoryCommand::AddPlugins(app))
    }

    /// Sets up a client in the current world.
    pub fn setup_game(&mut self, world: &mut World, token: ServerConnectToken, start_info: GameStartInfo)
    {
        (self.callback)(ClientFactoryCommand::SetupGame(world, token, start_info))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ClientFactoryPlugin
{
    pub(crate) factory: Arc<Mutex<Option<ClientFactory>>>,
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
        app.insert_resource(factory);
    }
}

//-------------------------------------------------------------------------------------------------------------------
