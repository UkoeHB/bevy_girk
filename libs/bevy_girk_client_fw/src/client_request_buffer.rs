//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_replicon::network_event::EventType;
use bytes::Bytes;
use serde::Serialize;

//standard shortcuts
use std::any::TypeId;
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// A request that will be sent from the client to the server.
pub struct PendingClientRequest
{
    pub request     : Bytes,
    pub send_policy : EventType,
}

//-------------------------------------------------------------------------------------------------------------------

/// A queue of requests waiting to be dispatched to the game.
#[derive(Resource)]
pub struct ClientRequestBuffer
{
    user_request_id : TypeId,
    sender          : Sender<PendingClientRequest>,
    receiver        : Receiver<PendingClientRequest>,
}

impl ClientRequestBuffer
{
    /// Makes a new buffer from the expected user-defined client request type.
    pub fn new<T: Serialize + Debug + IntoEventType + 'static>() -> Self
    {
        let (sender, receiver) = new_channel::<PendingClientRequest>();
        Self{
            user_request_id: TypeId::of::<T>(),
            sender,
            receiver,
        }
    }

    /// Resets the buffer for a new tick.
    pub fn reset(&mut self)
    {
        let mut count = 0;
        while let Some(_) = self.next()
        {
            count += 1;
        }
        if count != 0 { tracing::warn!(count, "buffer not empty on reset"); }
    }

    /// Adds a client framework request to the buffer.
    pub fn fw_request(&self, request: ClientFwRequest)
    {
        tracing::trace!(?request, "buffering fw request");

        let send_policy = request.into_event_type();
        self.sender.send(
                PendingClientRequest{
                        request: Bytes::from(
                            ser_msg(&ClientRequest{ req: AimedMsg::<_, ()>::Fw(request) })
                        ),
                        send_policy
                    }
            ).expect("failed buffering fw request");
    }

    /// Adds a user-defined client request to the buffer.
    ///
    /// Panics if `T` does not match the type used to create this buffer with [`Self::new`].
    pub fn request<T: Serialize + Debug + IntoEventType + 'static>(&self, request: T)
    {
        if TypeId::of::<T>() != self.user_request_id { panic!("client request type does not match registered type"); }

        tracing::trace!(?request, "buffering core request");

        let send_policy = request.into_event_type();
        self.sender.send(
                PendingClientRequest{
                        request: Bytes::from(
                            ser_msg(&ClientRequest{ req: AimedMsg::<ClientFwRequest, _>::Core(request) })
                        ),
                        send_policy
                    }
            ).expect("failed buffering user request");
    }

    /// Get the next pending client request.
    pub fn next(&mut self) -> Option<PendingClientRequest>
    {
        self.receiver.try_recv()
    }
}

//-------------------------------------------------------------------------------------------------------------------
