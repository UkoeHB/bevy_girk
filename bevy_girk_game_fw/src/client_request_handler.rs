//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::ClientId;
use serde::Deserialize;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Deserializes bytes from a [`ClientPacket`] into a client request.
fn deserialize_client_request<T: Debug + for<'de> Deserialize<'de> + IntoChannelKind>(
    client_id     : ClientId,
    client_packet : &ClientPacket,
) -> Result<T, Option<ClientFwRequest>>
{
    let Some(req) = deser_msg::<ClientRequestData::<T>>(&client_packet.request[..]) else { return Err(None); };
    let send_policy = client_packet.send_policy;

    match req.req
    {
        AimedMsg::Fw(fw_request) =>
        {
            if fw_request.into_event_type() != send_policy
            { tracing::trace!(?client_id, "ignoring client fw request with invalid send policy"); return Err(None); }

            tracing::trace!(?client_id, ?send_policy, ?fw_request, "received client fw request");
            return Err(Some(fw_request));
        }
        AimedMsg::Core(request) =>
        {
            if request.into_event_type() != send_policy
            { tracing::trace!(?client_id, "ignoring client request with invalid send policy"); return Err(None); }

            tracing::trace!(?client_id, ?send_policy, ?request, "received client request");
            return Ok(request);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling client requests.
#[derive(Resource)]
pub struct ClientRequestHandler
{
    handler: Box<dyn Fn(&mut World, ClientId, &ClientPacket) -> Result<(), Option<ClientFwRequest>> + Sync + Send>
}

impl ClientRequestHandler
{
    pub fn new<T>(handler: impl Fn(&mut World, ClientId, T) + Sync + Send + 'static) -> Self
    where
        T: Debug + for<'de> Deserialize<'de> + IntoChannelKind
    {
        Self{
            handler: Box::new(move |world, id, packet| {
                let client_req = deserialize_client_request(id, packet)?;
                (handler)(world, id, client_req);
                Ok(())
            })
        }
    }

    pub fn try_call(
        &self,
        world         : &mut World,
        client_id     : ClientId,
        client_packet : &ClientPacket
    ) -> Result<(), Option<ClientFwRequest>>
    {
        (self.handler)(world, client_id, client_packet)
    }
}

//-------------------------------------------------------------------------------------------------------------------
