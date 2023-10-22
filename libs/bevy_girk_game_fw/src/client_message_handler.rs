//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling client messages.
/// Example:
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_girk_game_fw::*;
/// # use std::vec::Vec;
/// fn handler(world: &mut World, bytes: Vec<u8>, client_id: ClientIdType) -> bool
/// {
///     //deserialize bytes
///     //handle deserialized message
///     true
/// }
/// ```
#[derive(Resource)]
pub struct ClientMessageHandler
{
    handler: Box<dyn Fn(&mut World, Vec<u8>, ClientIdType) -> bool + Sync + Send>
}

impl ClientMessageHandler
{
    pub fn new(handler: impl Fn(&mut World, Vec<u8>, ClientIdType) -> bool + Sync + Send + 'static) -> ClientMessageHandler
    {
        ClientMessageHandler{ handler: Box::new(handler) }
    }

    pub fn try_call(&self, world: &mut World, serialized_message: Vec<u8>, client_id: ClientIdType) -> bool
    {
        (self.handler)(world, serialized_message, client_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------
