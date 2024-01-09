//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_replicon::network_event::EventType;
use bytes::Bytes;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A message aimed at a particular deserializer.
#[derive(Serialize, Deserialize)]
pub enum AimedMsg<F, T>
{
    // Server/client framework
    Fw(F),
    // Server/client core
    Core(T),
}

//-------------------------------------------------------------------------------------------------------------------

/// A game message consumed by a client/clients.
#[derive(Serialize, Deserialize)]
pub struct GameMessage<T>
{
    /// Number of ticks elapsed in the game framework.
    pub ticks: Ticks,
    /// The message.
    pub msg: AimedMsg<GameFwMsg, T>,
}

//-------------------------------------------------------------------------------------------------------------------

/// A client request consumed by the game server.
#[derive(Serialize, Deserialize)]
pub struct ClientRequest<T>
{
    /// The request.
    pub req: AimedMsg<ClientFwRequest, T>,
}

//-------------------------------------------------------------------------------------------------------------------

/// A serialized message to send from the game to a client.
#[derive(Debug)]
pub struct GamePacket
{
    /// Id of destination client.
    pub client_id: ClientIdType,
    /// Packet send policy (reliability and ordering guarantee).
    pub send_policy: EventType,
    /// The message.
    pub message: Bytes,
}

//-------------------------------------------------------------------------------------------------------------------

/// A serialized request to send from a client to the game.
#[derive(Debug)]
pub struct ClientPacket
{
    /// Id of originating client.
    pub client_id: ClientIdType,
    /// Packet send policy (reliability and ordering guarantee).
    pub send_policy: EventType,
    /// The request.
    pub request: Bytes,
}

//-------------------------------------------------------------------------------------------------------------------
