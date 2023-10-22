//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_replicon::network_event::SendPolicy;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A message aimed at a particular deserializer.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AimedMsg
{
    // Server/client framework
    Fw{
        #[serde_as(as = "Bytes")]
        bytes: Vec<u8>
    },
    // Server/client core
    Core{
        #[serde_as(as = "Bytes")]
        bytes: Vec<u8>
    },
}

//-------------------------------------------------------------------------------------------------------------------

/// A game message consumed by a client/clients.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameMessage
{
    /// Message
    pub message : AimedMsg,
    /// Number of ticks elapsed in the game framework.
    pub ticks   : Ticks,
}

//-------------------------------------------------------------------------------------------------------------------

/// A client message consumed by the game server.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientMessage
{
    /// Message
    pub message: AimedMsg,
}

//-------------------------------------------------------------------------------------------------------------------

/// A serialized message to send from the game to a client.
#[derive(Debug)]
pub struct GamePacket
{
    /// Id of destination client.
    pub client_id: ClientIdType,
    /// Packet send policy (reliability and ordering guarantee).
    pub send_policy: SendPolicy,
    /// The message.
    pub message: GameMessage,
}

//-------------------------------------------------------------------------------------------------------------------

/// A serialized message to send from a client to the game.
#[derive(Debug)]
pub struct ClientPacket
{
    /// Id of originating client.
    pub client_id: ClientIdType,
    /// Packet send policy (reliability and ordering guarantee).
    pub send_policy: SendPolicy,
    /// The message.
    pub message: ClientMessage,
}

//-------------------------------------------------------------------------------------------------------------------
