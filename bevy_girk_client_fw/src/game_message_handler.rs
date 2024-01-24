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
pub fn deserialize_game_message<T: Debug + for<'de> Deserialize<'de> + IntoEventType>(
    game_packet: &GamePacket,
) -> Result<(Tick, T), Option<(Tick, GameFwMsg)>>
{
    let Some(message) = deser_msg::<GameMessageData::<T>>(&game_packet.message[..]) else { return Err(None); };
    let send_policy = game_packet.send_policy;

    match message.msg
    {
        AimedMsg::Fw(fw_msg) =>
        {
            if fw_msg.into_event_type() != send_policy
            { tracing::error!("ignoring game fw message with invalid send policy"); return Err(None); }

            tracing::trace!(?send_policy, ?fw_msg, "received game fw message");
            return Err(Some((message.tick, fw_msg)));
        }
        AimedMsg::Core(msg) =>
        {
            if msg.into_event_type() != send_policy
            { tracing::error!("ignoring game message with invalid send policy"); return Err(None); }

            tracing::trace!(?send_policy, ?msg, "received game message");
            return Ok((message.tick, msg));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling game messages.
///
/// The function is expected to return a `Some((Tick, GameFwMsg))` if that's the deserialization result, or a `None`
/// if handling the game message fails.
/// It is recommended to use [`deserialize_game_message`].
/// We allow access to the game framework message in case a user wants to read it or transform a custom game
/// message into a framework message (not recommended).
///
/// Example:
/**
```no_run
# use bevy::prelude::*;
# use bevy_girk_game_fw::*;
fn handler(world: &mut World, game_packet: GamePacket) -> Result<(), Option<(Tick, GameFwMsg)>>
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
    handler: Box<dyn Fn(&mut World, &GamePacket) -> Result<(), Option<(Tick, GameFwMsg)>> + Sync + Send>
}

impl GameMessageHandler
{
    pub fn new(
        handler: impl Fn(&mut World, &GamePacket) -> Result<(), Option<(Tick, GameFwMsg)>> + Sync + Send + 'static
    ) -> GameMessageHandler
    {
        GameMessageHandler{ handler: Box::new(handler) }
    }

    pub fn try_call(&self, world: &mut World, game_packet: &GamePacket) -> Result<(), Option<(Tick, GameFwMsg)>>
    {
        (self.handler)(world, game_packet)
    }
}

//-------------------------------------------------------------------------------------------------------------------
