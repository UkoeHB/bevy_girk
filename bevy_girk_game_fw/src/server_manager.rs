//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::{ClientAttributes, ServerEventSender, Visibility};
use bincode::Options;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::any::TypeId;
use std::fmt::Debug;
use std::io::Cursor;

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

        self.send_game_packet(condition, &data, send_policy);
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
        tracing::trace!(tick, ?message, ?condition, "sending core message");

        let send_policy = message.into_event_type();
        let data = GameMessageData{ tick: **self.tick, msg: AimedMsg::<GameFwMsg, _>::Core(message) };

        self.send_game_packet(condition, &data, send_policy);
    }

    /// Gets a mutable reference to `ClientAttributes`.
    ///
    /// This can be used to add/remove attributes from clients.
    pub fn attributes<'a: 'w>(&'a mut self) -> &'a mut ClientAttributes
    {
        &mut self.attributes
    }

    /// Sends a game packet to the appropriate clients.
    fn send_game_packet<T: Serialize + for<'de> Deserialize<'de>>(
        &mut self,
        condition   : Visibility,
        data        : &GameMessageData<T>,
        send_policy : EventType,
    ){
        let mut previous_message = None;
        let serialize_fn = |cursor: &mut Cursor<Vec<u8>>| -> bincode::Result<()>
        {
            bincode::DefaultOptions::new().serialize_into(cursor, data)
        };
        let producer = |client_state: Option<&ClientState>| -> GamePacket
        {
            let message = match client_state
            {
                Some(client_state) => serialize_with(client_state, previous_message.take(), serialize_fn).unwrap(),
                None =>
                {
                    if let Some(previous) = previous_message.take()
                    {
                        // If we can reuse the previous message then do so. We assume no client state means the change
                        // tick will be discarded.
                        previous
                    }
                    else
                    {
                        // This branch most likely executes when doing local singleplayer with the server-as-client.
                        let mut cursor = Cursor::new(Vec::new());
                        bincode::DefaultOptions::new().serialize_into(&mut cursor, &RepliconTick::default()).unwrap();
                        let tick_size = cursor.get_ref().len();
                        (serialize_fn)(&mut cursor).unwrap();
                        SerializedMessage {
                            tick: RepliconTick::default(),
                            tick_size,
                            bytes: cursor.into_inner().into(),
                        }
                    }
                }
            };
            let packet = GamePacket{ message: message.bytes.clone(), send_policy };
            previous_message = Some(message);
            packet
        };

        self.sender.send_with(&self.attributes, condition, producer);
    }
}

//-------------------------------------------------------------------------------------------------------------------
