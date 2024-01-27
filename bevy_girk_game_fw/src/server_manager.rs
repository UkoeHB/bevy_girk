//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon_attributes::{ClientAttributes, ServerEventSender, Visibility};
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
    pub fn new<T: Serialize + Debug + IntoEventType + 'static>() -> Self
    {
        Self(TypeId::of::<T>())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Uses server resources to expose a concise API.
//todo: rename to ClientManager
#[derive(SystemParam)]
pub struct ServerManager<'w>
{
    message_id  : Res<'w, GameMessageType>,
    tick        : Res<'w, GameFwTick>,
    sender      : ServerEventSender<'w, GamePacket>,
    attributes  : ClientAttributes<'w>,
}

impl<'w> ServerManager<'w>
{
    /// Sends a game framework message to clients that match the visibility condition.
    pub fn fw_send(&mut self, message: GameFwMsg, condition: Visibility)
    {
        let tick = ***self.tick;
        tracing::trace!(tick, ?message, ?condition, "sending fw message");

        let send_policy = message.into_event_type();
        let data = GameMessageData{ tick: **self.tick, msg: AimedMsg::<GameFwMsg, ()>::Fw(message) };

        let packet = GamePacket{ message: ser_msg(&data).into(), send_policy };
        self.sender.send(&self.attributes, packet, condition);
    }

    /// Sends a user-defined message to clients that match the visibility condition.
    ///
    /// Panics when `debug_assertions` are enabled if `T` does not match the type specified in [`GameMessageType`].
    pub fn send<T>(&mut self, message: T, condition: Visibility)
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + IntoEventType + 'static
    {
        debug_assert_eq!(TypeId::of::<T>(), **self.message_id);
        let tick = ***self.tick;
        tracing::trace!(tick, ?message, ?condition, "sending message");

        let send_policy = message.into_event_type();
        let data = GameMessageData{ tick: **self.tick, msg: AimedMsg::<GameFwMsg, _>::Core(message) };

        let packet = GamePacket{ message: ser_msg(&data).into(), send_policy };
        self.sender.send(&self.attributes, packet, condition);
    }

    /// Gets a mutable reference to `ClientAttributes`.
    ///
    /// This can be used to add/remove attributes from clients.
    pub fn attributes<'a: 'w>(&'a mut self) -> &'a mut ClientAttributes
    {
        &mut self.attributes
    }
}

//-------------------------------------------------------------------------------------------------------------------
