//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon_attributes::{ClientAttributes, ServerEventSender, VisibilityCondition};
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::any::TypeId;
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Records the `TypeId` of game messages.
///
/// This allows the message type to be injected into the game framework without needing generics everywhere.
#[derive(Resource, Deref, Debug)]
pub struct GameMessageType(TypeId);

impl GameMessageType
{
    pub fn new<T: Serialize + Debug + IntoChannelKind + 'static>() -> Self
    {
        Self(TypeId::of::<T>())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sends game messages to clients based on specified visibility conditions.
///
/// Messages are sent via `bevy_replicon`, which means sent messages will synchronize with spawns/despawns/etc. of
/// replicated entities.
#[derive(SystemParam)]
pub struct GameMessageSender<'w>
{
    message_id  : Res<'w, GameMessageType>,
    tick        : Res<'w, GameFwTick>,
    sender      : ServerEventSender<'w, GamePacket>,
}

impl<'w> GameMessageSender<'w>
{
    /// Sends a game framework message to clients that match the visibility condition.
    pub fn fw_send(&mut self, attributes: &ClientAttributes, message: GameFwMsg, condition: VisibilityCondition)
    {
        let tick = ***self.tick;
        tracing::trace!(tick, ?message, ?condition, "sending fw message");

        let send_policy = message.into_event_type();
        let data = GameMessageData{ tick: **self.tick, msg: AimedMsg::<GameFwMsg, ()>::Fw(message) };

        let packet = GamePacket{ message: ser_msg(&data).into(), send_policy };
        self.sender.send(&attributes, packet, condition);
    }

    /// Sends a user-defined message to clients that match the visibility condition.
    ///
    /// Panics when `debug_assertions` are enabled if `T` does not match the type specified in [`GameMessageType`].
    pub fn send<T>(&mut self, attributes: &ClientAttributes, message: T, condition: VisibilityCondition)
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + IntoChannelKind + 'static
    {
        debug_assert_eq!(TypeId::of::<T>(), **self.message_id);
        let tick = ***self.tick;
        tracing::trace!(tick, ?message, ?condition, "sending message");

        let send_policy = message.into_event_type();
        let data = GameMessageData{ tick: **self.tick, msg: AimedMsg::<GameFwMsg, _>::Core(message) };

        let packet = GamePacket{ message: ser_msg(&data).into(), send_policy };
        self.sender.send(&attributes, packet, condition);
    }
}

//-------------------------------------------------------------------------------------------------------------------
