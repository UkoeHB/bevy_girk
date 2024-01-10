//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::Deserialize;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Deserializes bytes from a [`ClientPacket`] into a specified client request type.
//todo: validate send policy
pub fn deserialize_client_request<T: Debug + for<'de> Deserialize<'de>>(client_packet: &ClientPacket) -> Option<T>
{
    let Some(req) = deser_msg::<ClientRequest::<T>>(&client_packet.request[..])
    else { tracing::trace!("failed to deserialize client request"); return None; };
    let AimedMsg::Core(request) = req.req
    else { tracing::trace!("failed to deserialize client request"); return None; };

    tracing::trace!(?client_packet.send_policy, ?request, "received client request");
    Some(request)
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps an injected function for handling client requests.
///
/// Example:
/**
```no_run
# use bevy::prelude::*;
# use bevy_girk_game_fw::*;
# use std::vec::Vec;
fn handler(world: &mut World, client_packet: ClientPacket) -> bool
{
    let Some(request) = deserialize_client_request::<MyClientRequestType>(&client_packet) else { return false; };

    //handle deserialized message
    true
}
```
*/
#[derive(Resource)]
pub struct ClientRequestHandler
{
    handler: Box<dyn Fn(&mut World, &ClientPacket) -> bool + Sync + Send>
}

impl ClientRequestHandler
{
    pub fn new(handler: impl Fn(&mut World, &ClientPacket) -> bool + Sync + Send + 'static) -> ClientRequestHandler
    {
        ClientRequestHandler{ handler: Box::new(handler) }
    }

    pub fn try_call(&self, world: &mut World, client_packet: &ClientPacket) -> bool
    {
        (self.handler)(world, client_packet)
    }
}

//-------------------------------------------------------------------------------------------------------------------
