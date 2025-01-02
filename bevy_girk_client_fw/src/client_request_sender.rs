//local shortcuts
use bevy_girk_game_fw::{AimedMsg, ClientFwRequest, ClientPacket, ClientRequestData};
use bevy_girk_utils::{ser_msg, IntoChannelKind};

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bytes::Bytes;
use serde::Serialize;

//standard shortcuts
use std::any::TypeId;
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Records the `TypeId` of client requests.
///
/// This allows the request type to be injected into the client framework without needing generics everywhere.
#[derive(Resource, Deref, Debug)]
pub struct ClientRequestType(TypeId);

impl ClientRequestType
{
    pub fn new<T: Serialize + Debug + IntoChannelKind + 'static>() -> Self
    {
        Self(TypeId::of::<T>())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sends client requests to the game.
///
/// Requests are sent via `bevy_replicon`, which means the sender will synchronize with client connection events.
/// Client requests are guaranteed to be dropped between a client disconnect and the client re-entering
/// [`ClientFwState::Syncing`].
///
/// Can be read by `ClientRequestHandler` on the server.
#[derive(SystemParam)]
pub struct ClientRequestSender<'w>
{
    req_type: Res<'w, ClientRequestType>,
    writer: EventWriter<'w, ClientPacket>,
}

impl<'w> ClientRequestSender<'w>
{
    /// Sends a client framework request.
    pub fn fw_request(&mut self, request: ClientFwRequest)
    {
        tracing::trace!("sending fw request: {request:?}");

        let send_policy = request.into_event_type();
        let request = Bytes::from(ser_msg(&ClientRequestData{ req: AimedMsg::<_, ()>::Fw(request) }));
        self.writer.send(ClientPacket{ send_policy, request });
    }

    /// Sends a user-defined client request.
    ///
    /// Panics when `debug_assertions` are enabled if `T` does not match the [`ClientRequestType`].
    pub fn request<T: Serialize + Debug + IntoChannelKind + 'static>(&mut self, request: T)
    {
        debug_assert_eq!(TypeId::of::<T>(), **self.req_type);

        tracing::trace!("sending core request: {request:?}");

        let send_policy = request.into_event_type();
        let request = Bytes::from(ser_msg(&ClientRequestData{ req: AimedMsg::<ClientFwRequest, _>::Core(request) }));
        self.writer.send(ClientPacket{ send_policy, request });
    }
}

//-------------------------------------------------------------------------------------------------------------------
