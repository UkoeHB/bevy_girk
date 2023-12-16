//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::network_event::EventType;
use serde::Serialize;

//standard shortcuts
use std::collections::{VecDeque, vec_deque::Drain};
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// A message that will be sent from the client to the server.
pub struct PendingClientMessage
{
    pub message     : AimedMsg,
    pub send_policy : EventType
}

//-------------------------------------------------------------------------------------------------------------------

/// A queue of messages waiting to be dispatched to the game.
//todo: adding messages requires synchronization between adders
#[derive(Resource, Default)]
pub struct ClientMessageBuffer
{
    buffer: VecDeque<PendingClientMessage>
}

impl ClientMessageBuffer
{
    pub fn add_fw_msg<T: Serialize + Debug>(&mut self, message: &T, send_policy: impl Into<EventType>)
    {
        tracing::trace!(?message, "buffering fw message");
        self.buffer.push_back(
                PendingClientMessage{
                        message     : AimedMsg::Fw{ bytes: ser_msg(message) },
                        send_policy : send_policy.into()
                    }
            );
    }

    pub fn add_core_msg<T: Serialize + Debug>(&mut self, message: &T, send_policy: impl Into<EventType>)
    {
        tracing::trace!(?message, "buffering core message");
        self.buffer.push_back(
            PendingClientMessage{
                    message     : AimedMsg::Core{ bytes: ser_msg(message) },
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
