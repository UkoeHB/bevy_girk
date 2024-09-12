//local shortcuts

//third-party shortcuts
use bevy_renet2::renet2::transport::{ConnectToken, ServerCertHash};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct TargetClient(pub u64);
#[derive(Debug, Copy, Clone)]
pub struct TargetAll;
#[derive(Debug, Copy, Clone)]
pub struct TargetAllExcept(pub u64);

impl From<TargetClient> for SendMode
{
    fn from(client: TargetClient) -> SendMode { return SendMode::Direct(ClientId::new(client.0)) }
}
impl From<TargetAll> for SendMode
{
    fn from(_: TargetAll) -> SendMode { return SendMode::Broadcast }
}
impl From<TargetAllExcept> for SendMode
{
    fn from(exception: TargetAllExcept) -> SendMode { return SendMode::BroadcastExcept(ClientId::new(exception.0)) }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct SendUnreliable;
#[derive(Debug, Copy, Clone)]
pub struct SendUnordered;
#[derive(Debug, Copy, Clone)]
pub struct SendOrdered;

impl From<SendUnreliable> for ChannelKind
{
    fn from(_: SendUnreliable) -> ChannelKind { return ChannelKind::Unreliable }
}
impl From<SendUnordered> for ChannelKind
{
    fn from(_: SendUnordered) -> ChannelKind { return ChannelKind::Unordered }
}
impl From<SendOrdered> for ChannelKind
{
    fn from(_: SendOrdered) -> ChannelKind { return ChannelKind::Ordered }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for converting a message type into its send policy.
///
/// Especially useful for enum-type messages where different variants have different send policies.
pub trait IntoChannelKind
{
    fn into_event_type(&self) -> ChannelKind;
}

//-------------------------------------------------------------------------------------------------------------------

/// Default implementation for tests.
impl IntoChannelKind for ()
{
    fn into_event_type(&self) -> ChannelKind { ChannelKind::Unreliable }
}

//-------------------------------------------------------------------------------------------------------------------

/// Holds a server's channel ID for `T`.
#[derive(Resource)]
pub struct EventChannel<T>
{
    id: u8,
    marker: PhantomData<T>,
}

impl<T> EventChannel<T>
{
    fn new(id: u8) -> Self
    {
        Self{ id, marker: PhantomData::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Prepares network channels for setting up `bevy_girk` clients and servers to send and receive packets.
pub fn prepare_network_channels(app: &mut App, resend_time: Duration)
{
    app.init_resource::<RepliconChannels>();
    let mut channels = app.world_mut().resource_mut::<RepliconChannels>();

    let unordered = RepliconChannel{
        kind: ChannelKind::Unordered,
        resend_time,
        max_bytes: None,
    };
    let ordered = RepliconChannel{
        kind: ChannelKind::Ordered,
        resend_time,
        max_bytes: None,
    };

    let unreliable_game_packet   = channels.create_server_channel(ChannelKind::Unreliable.into());
    let unordered_game_packet    = channels.create_server_channel(unordered.clone());
    let ordered_game_packet      = channels.create_server_channel(ordered.clone());
    let unreliable_client_packet = channels.create_client_channel(ChannelKind::Unreliable.into());
    let unordered_client_packet  = channels.create_client_channel(unordered);
    let ordered_client_packet    = channels.create_client_channel(ordered);

    app
        .insert_resource(EventChannel::<(GamePacket, SendUnreliable)>::new(unreliable_game_packet))
        .insert_resource(EventChannel::<(GamePacket, SendUnordered)>::new(unordered_game_packet))
        .insert_resource(EventChannel::<(GamePacket, SendOrdered)>::new(ordered_game_packet))
        .insert_resource(EventChannel::<(ClientPacket, SendUnreliable)>::new(unreliable_client_packet))
        .insert_resource(EventChannel::<(ClientPacket, SendUnordered)>::new(unordered_client_packet))
        .insert_resource(EventChannel::<(ClientPacket, SendOrdered)>::new(ordered_client_packet));
}

//-------------------------------------------------------------------------------------------------------------------
