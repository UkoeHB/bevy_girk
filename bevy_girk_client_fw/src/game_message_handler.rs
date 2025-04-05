//local shortcuts
use bevy_girk_game_fw::{AimedMsg, GameFwMsg, GameMessageData, GamePacket, Tick};
use bevy_girk_utils::{deser_msg, IntoChannel};

//third-party shortcuts
use bevy::prelude::*;
use serde::Deserialize;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Deserializes bytes from a [`GamePacket`] into a specified game message type.
fn deserialize_game_message<T: Debug + for<'de> Deserialize<'de> + IntoChannel>(
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
#[derive(Resource)]
pub struct GameMessageHandler
{
    handler: Box<dyn Fn(&mut World, &GamePacket) -> Result<(), Option<(Tick, GameFwMsg)>> + Sync + Send>
}

impl GameMessageHandler
{
    pub fn new<T>(handler: impl Fn(&mut World, Tick, T) + Sync + Send + 'static) -> Self
    where
        T: Debug + for<'de> Deserialize<'de> + IntoChannel
    {
        Self{
            handler: Box::new(move |world, packet| {
                let (tick, game_msg) = deserialize_game_message(packet)?;
                (handler)(world, tick, game_msg);
                Ok(())
            })
        }
    }

    pub fn try_call(&self, world: &mut World, game_packet: &GamePacket) -> Result<(), Option<(Tick, GameFwMsg)>>
    {
        (self.handler)(world, game_packet)
    }
}

//-------------------------------------------------------------------------------------------------------------------
