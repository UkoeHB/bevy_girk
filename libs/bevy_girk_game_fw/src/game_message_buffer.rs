//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_replicon::prelude::RepliconTick;
use bevy_replicon::network_event::EventType;
use bytes::Bytes;
use serde::Serialize;

//standard shortcuts
use std::any::TypeId;
use std::fmt::Debug;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

/// A message that will be sent to a client/clients.
pub struct PendingGameMessage
{
    pub message            : Bytes,
    pub access_constraints : Vec<InfoAccessConstraint>,
    pub send_policy        : EventType,
}

//-------------------------------------------------------------------------------------------------------------------

/// A queue of messages waiting to be dispatched to clients.
#[derive(Resource)]
pub struct GameMessageBuffer
{
    user_message_id : TypeId,
    sender          : Sender<PendingGameMessage>,
    receiver        : Receiver<PendingGameMessage>,
    change_tick     : RepliconTick,
    ticks           : Ticks,
}

impl GameMessageBuffer
{
    /// Makes a new buffer from the expected user-defined game message type.
    pub fn new<T: Serialize + Debug + IntoEventType + 'static>() -> Self
    {
        let (sender, receiver) = new_channel();
        Self{
            user_message_id: TypeId::of::<T>(),
            sender,
            receiver,
            change_tick : RepliconTick::default(),
            ticks       : Ticks::default(),
        }
    }

    /// Resets the buffer for a new tick.
    pub(crate) fn reset(&mut self, change_tick: RepliconTick, elapsed_ticks: Ticks)
    {
        self.change_tick = change_tick;
        self.ticks = elapsed_ticks;

        let mut count = 0;
        while let Some(_) = self.next() { count += 1; }
        if count != 0 { tracing::warn!(count, "buffer not empty on reset"); }
    }

    /// Adds a game framework message to the buffer.
    pub fn fw_send(&self, message: GameFwMsg, access_constraints: Vec<InfoAccessConstraint>)
    {
        tracing::trace!(?message, "buffering fw message");

        let send_policy = message.into_event_type();
        let message = Bytes::from(ser_msg(&(
                self.change_tick,
                GameMessageData{ ticks: self.ticks, msg: AimedMsg::<_, ()>::Fw(message) }
            )));
        self.sender.send(PendingGameMessage{ message, access_constraints, send_policy })
            .expect("failed buffering fw message");
    }

    /// Adds a user-defined game message to the buffer.
    ///
    /// Panics if `T` does not match the type used to create this buffer with [`Self::new`].
    pub fn send<T: Serialize + Debug + IntoEventType + 'static>(
        &self,
        message            : T,
        access_constraints : Vec<InfoAccessConstraint>
    ){
        if TypeId::of::<T>() != self.user_message_id { panic!("game message type does not match registered type"); }

        tracing::trace!(?message, "buffering core message");

        let send_policy = message.into_event_type();
        let message = Bytes::from(ser_msg(&(
                self.change_tick,
                GameMessageData{ ticks: self.ticks, msg: AimedMsg::<GameFwMsg, _>::Core(message) }
            )));
        self.sender.send(PendingGameMessage{ message, access_constraints, send_policy })
            .expect("failed buffering user message");
    }

    /// Takes the next available pending message.
    pub(crate) fn next(&mut self) -> Option<PendingGameMessage>
    {
        self.receiver.try_recv()
    }
}

//-------------------------------------------------------------------------------------------------------------------
