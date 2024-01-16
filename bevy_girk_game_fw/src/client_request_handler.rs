//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::Deserialize;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Deserializes bytes from a [`ClientPacket`] into a client request.
pub fn deserialize_client_request<T: Debug + for<'de> Deserialize<'de> + IntoEventType>(
    client_id     : ClientIdType,
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
            { tracing::trace!(client_id, "ignoring client fw request with invalid send policy"); return Err(None); }

            tracing::trace!(client_id, ?send_policy, ?fw_request, "received client fw request");
            return Err(Some(fw_request));
        }
        AimedMsg::Core(request) =>
        {
            if request.into_event_type() != send_policy
            { tracing::trace!(client_id, "ignoring client request with invalid send policy"); return Err(None); }

            tracing::trace!(client_id, ?send_policy, ?request, "received client request");
            return Ok(request);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling client requests.
///
/// The function is expected to return a `Some(ClientFwRequest)` if that's the deserialization result, or a `None`
/// if handling the client reqeust fails.
/// It is recommended to use [`deserialize_client_request`].
/// We allow access to the client framework request in case a user wants to read it or transform a custom client
/// request into a framework request (not recommended).
///
/// Example:
/**
```no_run
# use bevy::prelude::*;
# use bevy_girk_game_fw::*;
# use std::vec::Vec;
fn handler(world: &mut World, client_id: ClientIdType, client_packet: ClientPacket) -> Result<(), Option<ClientFwRequest>>
{
    let request = deserialize_client_request::<MyClientRequestType>(client_id, &client_packet)?;

    //handle deserialized message
    Ok(())
}
```
*/
#[derive(Resource)]
pub struct ClientRequestHandler
{
    handler: Box<dyn Fn(&mut World, ClientIdType, &ClientPacket) -> Result<(), Option<ClientFwRequest>> + Sync + Send>
}

impl ClientRequestHandler
{
    pub fn new(
        handler: impl Fn(&mut World, ClientIdType, &ClientPacket) -> Result<(), Option<ClientFwRequest>> + Sync + Send + 'static
    ) -> ClientRequestHandler
    {
        ClientRequestHandler{ handler: Box::new(handler) }
    }

    pub fn try_call(&self,
        world         : &mut World,
        client_id     : ClientIdType,
        client_packet : &ClientPacket
    ) -> Result<(), Option<ClientFwRequest>>
    {
        (self.handler)(world, client_id, client_packet)
    }
}

//-------------------------------------------------------------------------------------------------------------------
