//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::network_event::SendPolicy;
use serde::Serialize;

//standard shortcuts
use std::collections::{VecDeque, vec_deque::Drain};
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

/// A message that will be sent to a client/clients.
pub struct PendingGameMessage
{
    pub message            : AimedMsg,
    pub access_constraints : Vec<InfoAccessConstraint>,
    pub send_policy        : SendPolicy,
}

//-------------------------------------------------------------------------------------------------------------------

/// A queue of messages waiting to be dispatched to clients.
#[derive(Resource, Default)]
pub struct GameMessageBuffer
{
    buffer: VecDeque<PendingGameMessage>
}

//todo: adding messages requires synchronization between adders (may be fine for single-threaded game server)
impl GameMessageBuffer
{
    pub fn add_fw_msg<T: Serialize>(&mut self,
        message            : &T,
        access_constraints : Vec<InfoAccessConstraint>,
        send_policy        : impl Into<SendPolicy>
    ){
        self.buffer.push_back(
                PendingGameMessage{
                        message: AimedMsg::Fw{ bytes: ser_msg(message) },
                        access_constraints,
                        send_policy: send_policy.into()
                    }
            );
    }

    pub fn add_core_msg<T: Serialize>(&mut self,
        message            : &T,
        access_constraints : Vec<InfoAccessConstraint>,
        send_policy        : impl Into<SendPolicy>,
    ){
        self.buffer.push_back(
                PendingGameMessage{
                        message: AimedMsg::Core{ bytes: ser_msg(message) },
                        access_constraints,
                        send_policy: send_policy.into()
                    }
            );
    }

    pub fn drain(&mut self) -> Drain<'_, PendingGameMessage>
    {
        self.buffer.drain(..)
    }
}

//-------------------------------------------------------------------------------------------------------------------
