//local shortcuts
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::fmt::Debug;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for client factory implementations.
pub trait ClientFactoryImpl: Debug
{
    fn new_client(&self, token: ServerConnectToken, start_info: GameStartInfo) -> Result<(App, u64), ()>;
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps a client factory implementation.
#[derive(Clone, Debug)]
pub struct ClientFactory
{
    factory: Arc<dyn ClientFactoryImpl + Send + Sync>
}

impl ClientFactory
{
    /// Create a new client factory.
    pub fn new<F: ClientFactoryImpl + Send + Sync + Debug + 'static>(factory_impl: F) -> ClientFactory
    {
        ClientFactory { factory: Arc::new(factory_impl) }
    }

    /// Create a new client.
    ///
    /// Returns the client app and the expected protocol id of connect tokens, which can be used to validate new
    /// connect tokens.
    pub fn new_client(&self, token: ServerConnectToken, start_info: GameStartInfo) -> Result<(App, u64), ()>
    {
        self.factory.new_client(token, start_info)
    }
}

//-------------------------------------------------------------------------------------------------------------------
