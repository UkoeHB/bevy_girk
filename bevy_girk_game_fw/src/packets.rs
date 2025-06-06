//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::Channel;
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
pub struct GameMessageData<T>
{
    /// Game framework tick where this message was created.
    pub tick: Tick,
    /// The message.
    pub msg: AimedMsg<GameFwMsg, T>,
}

//-------------------------------------------------------------------------------------------------------------------

/// A client request consumed by the game server.
#[derive(Serialize, Deserialize)]
pub struct ClientRequestData<T>
{
    /// The request.
    pub req: AimedMsg<ClientFwRequest, T>,
}

//-------------------------------------------------------------------------------------------------------------------

/// A serialized message to send from the game to a client.
#[derive(Debug, Event, Clone)]
pub struct GamePacket
{
    /// Packet send policy (reliability and ordering guarantee).
    pub send_policy: Channel,
    /// The message.
    /// - On the server this equals: `(replicon change tick, message)`.
    /// - On the client this equals: `message`.
    pub message: Bytes,
}

//-------------------------------------------------------------------------------------------------------------------

/// A serialized request to send from a client to the game.
#[derive(Debug, Event)]
pub struct ClientPacket
{
    /// Packet send policy (reliability and ordering guarantee).
    pub send_policy: Channel,
    /// The request.
    pub request: Bytes,
}

//-------------------------------------------------------------------------------------------------------------------
