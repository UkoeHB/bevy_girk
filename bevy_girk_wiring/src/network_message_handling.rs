//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::client::{ClientSet, ServerInitTick};
use bevy_replicon::core::common_conditions::{client_connected, server_running};
use bevy_replicon::core::channels::RepliconChannel;
use bevy_replicon::core::replicon_tick::RepliconTick;
use bevy_replicon::core::ClientId;
use bevy_replicon::prelude::{
    ChannelKind, ConnectedClients, FromClient, RepliconChannels, RepliconClient, RepliconServer, SendMode, ToClients
};
use bevy_replicon::server::ServerSet;
use bincode::Options;
use bytes::Bytes;
use ordered_multimap::ListOrderedMultimap;

//standard shortcuts
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Holds a server's channel ID for `T`.
#[derive(Resource)]
struct EventChannel<T>
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
//-------------------------------------------------------------------------------------------------------------------

struct SerializedGamePacket
{
    change_tick: RepliconTick,
    bytes: Bytes,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Serializes change ticks into a message.
fn serialize_bytes_with_init_tick(
    cached      : Option<SerializedGamePacket>,
    change_tick : RepliconTick,
    data        : &[u8],
) -> bincode::Result<SerializedGamePacket>
{
    if let Some(cached) = cached
    {
        if cached.change_tick == change_tick
        {
            return Ok(cached);
        }
    }

    let tick_size = bincode::DefaultOptions::new().serialized_size(&change_tick)? as usize;
    let mut bytes = Vec::with_capacity(tick_size + data.len());
    bincode::DefaultOptions::new().serialize_into(&mut bytes, &change_tick)?;
    bytes.extend_from_slice(data);
    Ok(SerializedGamePacket {
        change_tick,
        bytes: bytes.into(),
    })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// Maximum number of client messages that the server will accept per tick.
const MAX_CLIENT_MESSAGES_PER_TICK: u16 = 64;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Applies all queued events if their tick is less or equal to `ServerInitTick`.
fn pop_game_packet_queue(
    init_tick: Res<ServerInitTick>,
    mut server_events: EventWriter<GamePacket>,
    mut event_queue: ResMut<GamePacketQueue>,
){
    while let Some((_, event)) = event_queue.pop_if_le(**init_tick)
    {
        server_events.send(event);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Clears queued events.
///
/// We clear events while waiting for a connection to ensure clean reconnects.
fn reset_game_packet_queue(mut event_queue: ResMut<GamePacketQueue>)
{
    if !event_queue.0.is_empty()
    {
        warn!(
            "discarding {} queued server events due to a disconnect",
            event_queue.0.values_len()
        );
    }
    event_queue.0.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Discards all pending events.
///
/// We discard events while waiting to connect to ensure clean reconnects.
fn reset_client_packet_events(mut events: ResMut<Events<ClientPacket>>)
{
    let drained_count = events.drain().count();
    if drained_count > 0 { warn!("discarded {drained_count} client events due to a disconnect"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Stores all received events from server that arrived earlier then replication message with their tick.
///
/// Stores data sorted by ticks and maintains order of arrival.
/// Needed to ensure that when an event is triggered, all the data that it affects or references already exists.
#[derive(Resource)]
struct GamePacketQueue(ListOrderedMultimap<RepliconTick, GamePacket>);

impl GamePacketQueue {
    /// Inserts a new event.
    ///
    /// The event will be queued until [`RepliconTick`] is bigger or equal to the tick specified here.
    fn insert(&mut self, tick: RepliconTick, event: GamePacket)
    {
        self.0.insert(tick, event);
    }

    /// Pops the next event that is at least as old as the specified replicon tick.
    fn pop_if_le(&mut self, init_tick: RepliconTick) -> Option<(RepliconTick, GamePacket)>
    {
        let (tick, _) = self.0.front()?;
        if *tick > init_tick { return None; }
        self.0
            .pop_front()
            .map(|(tick, event)| (tick.into_owned(), event))
    }
}

impl Default for GamePacketQueue
{
    fn default() -> Self
    {
        Self(Default::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Server -> Client
fn send_server_packets(
    mut game_packets   : ResMut<Events<ToClients<GamePacket>>>,
    client_cache       : Res<ConnectedClients>,
    mut server         : ResMut<RepliconServer>,
    unreliable_channel : Res<EventChannel<(GamePacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(GamePacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(GamePacket, SendOrdered)>>,
){
    let mut prev: Option<*const u8> = None;
    let mut buffer: Option<SerializedGamePacket> = None;

    for packet in game_packets.drain()
    {
        let SendMode::Direct(client_id) = packet.mode else { panic!("invalid game packet send mode {:?}", packet.mode); };

        // if sending a different message than the previous packet, clear the buffer
        let this_packet = packet.event.message.as_ptr();
        if let Some(prev) = prev.take()
        {
            if !std::ptr::eq(prev, this_packet)
            {
                buffer = None;
            }
        }
        prev = Some(this_packet);

        // extract the buffered state before the early-outs
        let cached = buffer.take();

        // access this client's replicon state
        let Some(client) = client_cache.get_client(client_id)
        else { tracing::debug!(?client_id, "ignoring game packet sent to disconnected client"); continue; };

        // construct the final message, using the cached bytes if possible
        let Ok(message) = serialize_bytes_with_init_tick(cached, client.init_tick(), &packet.event.message)
        else { tracing::error!(?client_id, "failed serializing game packet for client"); continue; };

        match packet.event.send_policy
        {
            ChannelKind::Unreliable => server.send(client_id, unreliable_channel.id, message.bytes.clone()),
            ChannelKind::Unordered  => server.send(client_id, unordered_channel.id, message.bytes.clone()),
            ChannelKind::Ordered    => server.send(client_id, ordered_channel.id, message.bytes.clone())
        }

        // cache this message for possible re-use with the next client
        buffer = Some(message);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Client <- Server
fn receive_server_packets(
    mut client         : ResMut<RepliconClient>,
    mut game_packets   : EventWriter<GamePacket>,
    unreliable_channel : Res<EventChannel<(GamePacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(GamePacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(GamePacket, SendOrdered)>>,
    mut packet_queue   : ResMut<GamePacketQueue>,
    replicon_tick      : Res<ServerInitTick>,
){
    // receive ordered messages first since they are probably oldest
    for &(channel_id, send_policy) in
        [
            (Into::<u8>::into(ordered_channel.id), ChannelKind::Ordered),
            (Into::<u8>::into(unordered_channel.id), ChannelKind::Unordered),
            (Into::<u8>::into(unreliable_channel.id), ChannelKind::Unreliable),
        ].iter()
    {
        for mut message in client.receive(channel_id)
        {
            // extract the layered-in replicon init tick
            let Some(init_tick) = deser_bytes_partial::<RepliconTick>(&mut message)
            else { tracing::error!("failed deserializing init tick, ignoring server message"); continue; };
            let packet = GamePacket{ send_policy, message };

            match init_tick <= **replicon_tick
            {
                true  => { game_packets.send(packet); }
                false => { packet_queue.insert(init_tick, packet); }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Client -> Server
fn send_client_packets(
    mut client_packets : ResMut<Events<ClientPacket>>,
    mut client         : ResMut<RepliconClient>,
    unreliable_channel : Res<EventChannel<(ClientPacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(ClientPacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(ClientPacket, SendOrdered)>>,
){
    for client_packet in client_packets.drain()
    {
        match client_packet.send_policy
        {
            ChannelKind::Unreliable => client.send(unreliable_channel.id, client_packet.request),
            ChannelKind::Unordered  => client.send(unordered_channel.id, client_packet.request),
            ChannelKind::Ordered    => client.send(ordered_channel.id, client_packet.request)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Server <- Client
fn receive_client_packets(
    mut messages_count : Local<HashMap<ClientId, u16>>,
    mut server         : ResMut<RepliconServer>,
    mut client_packets : EventWriter<FromClient<ClientPacket>>,
    unreliable_channel : Res<EventChannel<(ClientPacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(ClientPacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(ClientPacket, SendOrdered)>>,
    clients            : Res<GameFwClients>
){
    messages_count.clear();

    // receive ordered messages first since they are probably oldest
    for &(channel_id, send_policy) in
        [
            (Into::<u8>::into(ordered_channel.id), ChannelKind::Ordered),
            (Into::<u8>::into(unordered_channel.id), ChannelKind::Unordered),
            (Into::<u8>::into(unreliable_channel.id), ChannelKind::Unreliable),
        ].iter()
    {
        for (client_id, request) in server.receive(channel_id)
        {
            // ignore unregistered client ids
            // - if this error is encountered, then you are issuing connect tokens to clients that weren't registered
            if !clients.contains(&client_id) { tracing::error!(?client_id, "ignoring client with unknown id"); continue; };

            // if too many messages were received this tick, ignore the remaining messages
            let messages_count = messages_count.entry(client_id).or_default();
            *messages_count += 1;
            if *messages_count > MAX_CLIENT_MESSAGES_PER_TICK
            {
                tracing::trace!(?client_id, channel_id, messages_count, "client exceeded max messages per tick");
                continue;
            }

            // send packet into server
            client_packets.send(FromClient{ client_id, event: ClientPacket{ send_policy, request } });
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn prepare_network_channels(app: &mut App, resend_time: Duration)
{
    app.init_resource::<RepliconChannels>();
    let mut channels = app.world.resource_mut::<RepliconChannels>();

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

/// System set that runs in [`PostUpdate`] between [`ServerSet::Send`] and [`ServerSet::SendPackets`].
///
/// [`GamePackets`](GamePacket) are sent here.
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SendServerEventsSet;

/// Plugin for server.
pub(crate) struct ServerEventHandlingPlugin;

impl Plugin for ServerEventHandlingPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<Events<FromClient<ClientPacket>>>()
            .init_resource::<Events<ToClients<GamePacket>>>()
            .configure_sets(PostUpdate,
                SendServerEventsSet
                    .after(ServerSet::Send)
                    .before(ServerSet::SendPackets)
                    .run_if(server_running)
            )
            .add_systems(
                PreUpdate,
                receive_client_packets
                    .in_set(ServerSet::Receive)
                    .run_if(server_running)
            )
            .add_systems(
                PostUpdate,
                send_server_packets
                    .in_set(SendServerEventsSet)
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System set that runs in [`PreUpdate`] after [`ClientSet::Receive`].
///
/// [`GamePackets`](GamePacket) are collected here.
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ReceiveServerEventsSet;

/// Plugin for client.
pub(crate) struct ClientEventHandlingPlugin;

impl Plugin for ClientEventHandlingPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<GamePacket>()  //from game
            .add_event::<ClientPacket>()  //to game
            .init_resource::<GamePacketQueue>()
            .configure_sets(PreUpdate,
                ReceiveServerEventsSet
                    .after(ClientSet::Receive)
                    .run_if(client_connected)
            )
            .add_systems(
                PreUpdate,
                (
                    reset_client_packet_events,
                    reset_game_packet_queue
                )
                    .in_set(ClientSet::ResetEvents)
            )
            .add_systems(
                PreUpdate,
                (
                    pop_game_packet_queue,
                    receive_server_packets
                )
                    .chain()
                    .in_set(ReceiveServerEventsSet)
            )
            .add_systems(
                PostUpdate,
                (
                    send_client_packets.run_if(client_connected)
                )
                    .chain()
                    .in_set(ClientSet::Send)
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
