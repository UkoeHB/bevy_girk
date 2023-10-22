//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling game messages.
///
/// Example:
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_girk_game_fw::*;
/// # use std::vec::Vec;
/// fn handler(world: &mut World, bytes: Vec<u8>, game_elapsed_ticks: Ticks) -> bool
/// {
///     //deserialize bytes
///     //handle deserialized message
///     true
/// }
/// ```
#[derive(Resource)]
pub struct GameMessageHandler
{
    handler: Box<dyn Fn(&mut World, Vec<u8>, Ticks) -> bool + Sync + Send>
}

impl GameMessageHandler
{
    pub fn new(handler: impl Fn(&mut World, Vec<u8>, Ticks) -> bool + Sync + Send + 'static) -> GameMessageHandler
    {
        GameMessageHandler{ handler: Box::new(handler) }
    }

    pub fn try_call(&self, world: &mut World, serialized_message: Vec<u8>, ticks: Ticks) -> bool
    {
        (self.handler)(world, serialized_message, ticks)
    }
}

//-------------------------------------------------------------------------------------------------------------------
