//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::Deserialize;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Deserializes bytes from a [`GamePacket`] into a specified game message type.
//todo: validate send policy
pub fn deserialize_game_message<T: Debug + for<'de> Deserialize<'de>>(game_packet: &GamePacket) -> Option<(Ticks, T)>
{
    let Some(message) = deser_msg::<GameMessage::<T>>(&game_packet.message[..]) else { return None; };
    let AimedMsg::Core(msg) = message.msg else { return None; };

    tracing::trace!(?msg, "received game message");
    Some((message.ticks, msg))
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling game messages.
///
/// Example:
/**
```no_run
# use bevy::prelude::*;
# use bevy_girk_game_fw::*;
fn handler(world: &mut World, game_packet: GamePacket) -> bool
{
    let Some((ticks, message)) = deserialize_game_message::<MyGameMessageType>(&game_packet) else { return false; };

    //handle deserialized message
    true
}
```
*/
#[derive(Resource)]
pub struct GameMessageHandler
{
    handler: Box<dyn Fn(&mut World, &GamePacket) -> bool + Sync + Send>
}

impl GameMessageHandler
{
    pub fn new(handler: impl Fn(&mut World, &GamePacket) -> bool + Sync + Send + 'static) -> GameMessageHandler
    {
        GameMessageHandler{ handler: Box::new(handler) }
    }

    pub fn try_call(&self, world: &mut World, game_packet: &GamePacket) -> bool
    {
        (self.handler)(world, game_packet)
    }
}

//-------------------------------------------------------------------------------------------------------------------
