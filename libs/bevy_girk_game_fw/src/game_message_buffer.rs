//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::network_event::EventType;
use bytes::Bytes;
use serde::Serialize;

//standard shortcuts
use std::collections::{VecDeque, vec_deque::Drain};
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
#[derive(Resource, Default)]
pub struct GameMessageBuffer
{
    ticks: Ticks,
    buffer: VecDeque<PendingGameMessage>,
}

//todo: adding messages requires synchronization between adders (may be fine for single-threaded game server)
impl GameMessageBuffer
{
    /// Resets the buffer for a new tick.
    pub fn reset(&mut self, elapsed_ticks: Ticks)
    {
        self.ticks = elapsed_ticks;
        self.buffer.clear();
    }

    /// Adds a game framework message to the buffer.
    pub fn add_fw_msg(&mut self,
        message            : GameFwMsg,
        access_constraints : Vec<InfoAccessConstraint>,
        send_policy        : impl Into<EventType>
    ){
        tracing::trace!(?message, "buffering fw message");
        self.buffer.push_back(
                PendingGameMessage{
                        message: Bytes::from(
                            ser_msg(&GameMessage{ ticks: self.ticks, message: AimedMsg::<_, ()>::Fw(message) })
                        ),
                        access_constraints,
                        send_policy: send_policy.into()
                    }
            );
    }

    /// Adds a user-defined game message to the buffer.
    //todo: parameterize the buffer on T for robustness (or set the expected type id when constructing the buffer)
    pub fn add_core_msg<T: Serialize + Debug>(&mut self,
        message            : T,
        access_constraints : Vec<InfoAccessConstraint>,
        send_policy        : impl Into<EventType>,
    ){
        tracing::trace!(?message, "buffering core message");
        self.buffer.push_back(
                PendingGameMessage{
                        message: Bytes::from(
                            ser_msg(&GameMessage{ ticks: self.ticks, message: AimedMsg::<GameFwMsg, _>::Core(message) })
                        ),
                        access_constraints,
                        send_policy: send_policy.into()
                    }
            );
    }

    /// Drains all pending messages.
    pub fn drain(&mut self) -> Drain<'_, PendingGameMessage>
    {
        self.buffer.drain(..)
    }
}

//-------------------------------------------------------------------------------------------------------------------
