//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::network_event::EventType;
use bytes::Bytes;
use serde::Serialize;

//standard shortcuts
use std::collections::{VecDeque, vec_deque::Drain};
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// A request that will be sent from the client to the server.
//todo: PendingClientRequest
pub struct PendingClientMessage
{
    pub request     : Bytes,
    pub send_policy : EventType,
}

//-------------------------------------------------------------------------------------------------------------------

/// A queue of requests waiting to be dispatched to the game.
//todo: adding messages requires synchronization between adders
#[derive(Resource, Default)]
pub struct ClientMessageBuffer
{
    buffer: VecDeque<PendingClientMessage>
}

impl ClientMessageBuffer
{
    /// Resets the buffer for a new tick.
    pub fn reset(&mut self)
    {
        self.buffer.clear();
    }

    /// Adds a client framework request to the buffer.
    pub fn add_fw_msg(&mut self, message: ClientFwRequest, send_policy: impl Into<EventType>)
    {
        tracing::trace!(?message, "buffering fw message");
        self.buffer.push_back(
                PendingClientMessage{
                        request     : Bytes::from(ser_msg(&ClientMessage{ message: AimedMsg::<_, ()>::Fw(message) })),
                        send_policy : send_policy.into()
                    }
            );
    }

    /// Adds a user-defined client request to the buffer.
    //todo: parameterize the buffer on T for robustness (or set the expected type id when constructing the buffer)
    pub fn add_core_msg<T: Serialize + Debug>(&mut self, message: T, send_policy: impl Into<EventType>)
    {
        tracing::trace!(?message, "buffering core message");
        self.buffer.push_back(
            PendingClientMessage{
                    request     : Bytes::from(ser_msg(&ClientMessage{ message: AimedMsg::<ClientFwRequest, _>::Core(message) })),
                    send_policy : send_policy.into()
                }
        );
    }

    pub fn drain(&mut self) -> Drain<'_, PendingClientMessage>
    {
        self.buffer.drain(..)
    }
}

//-------------------------------------------------------------------------------------------------------------------
