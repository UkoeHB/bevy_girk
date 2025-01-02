//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::core::ClientId;
use bevy_replicon_attributes::{vis, into_condition, Client, Global, ClientAttributes, ServerEventSender, VisibilityCondition};
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
///
/// Can be read by `GameMessageHandler` on the client.
#[derive(SystemParam)]
pub struct GameMessageSender<'w>
{
    message_id  : Res<'w, GameMessageType>,
    tick        : Res<'w, GameFwTick>,
    sender      : ServerEventSender<'w, GamePacket>,
    attributes  : ClientAttributes<'w>,
}

impl<'w> GameMessageSender<'w>
{
    /// Sends a game framework message to clients that match the visibility condition.
    pub fn fw_send(&mut self, message: GameFwMsg, condition: VisibilityCondition)
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
    pub fn send<T>(&mut self, message: T, condition: VisibilityCondition)
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + IntoChannelKind + 'static
    {
        debug_assert_eq!(TypeId::of::<T>(), **self.message_id);
        let tick = ***self.tick;
        tracing::trace!(tick, ?message, ?condition, "sending message");

        let send_policy = message.into_event_type();
        let data = GameMessageData{ tick: **self.tick, msg: AimedMsg::<GameFwMsg, _>::Core(message) };

        let packet = GamePacket{ message: ser_msg(&data).into(), send_policy };
        self.sender.send(&self.attributes, packet, condition);
    }

    /// Sends a user-defined message to a specific client.
    ///
    /// Equivalent to `self.send(message, vis!(Client(client)))`.
    pub fn send_to_client<T>(&mut self, message: T, client: u64)
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + IntoChannelKind + 'static
    {
        self.send(message, vis!(Client(ClientId::new(client))))
    }

    /// Sends a user-defined message to all clients.
    ///
    /// Equivalent to `self.send(message, vis!(Global))`.
    pub fn send_to_all<T>(&mut self, message: T)
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug + IntoChannelKind + 'static
    {
        self.send(message, vis!(Global))
    }

    /// Gets the [`ClientAttributes`] stored internally.
    pub fn client_attrs(&mut self) -> &mut ClientAttributes<'w>
    {
        &mut self.attributes
    }
}

//-------------------------------------------------------------------------------------------------------------------
