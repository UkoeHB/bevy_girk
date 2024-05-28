//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::client::ServerInitTick;
use bevy_replicon::core::replicon_channels::RepliconChannel;
use bevy_replicon::core::replicon_tick::RepliconTick;
use bevy_replicon::core::ClientId;
use bevy_replicon::network_event::server_event::ServerEventQueue;
use bevy_replicon::prelude::{
    ChannelKind, ConnectedClients, FromClient, RepliconChannels, RepliconClient, RepliconServer, SendMode, ToClients
};
use bincode::Options;
use bytes::Bytes;

use std::collections::HashMap;
//standard shortcuts
use std::marker::PhantomData;
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Holds a server's channel ID for `T`.
#[derive(Resource)]
pub struct EventChannel<T> {
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
fn serialize_bytes_with_change_tick(
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

/// Server -> Client
pub(crate) fn send_server_packets(
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
        let Ok(message) = serialize_bytes_with_change_tick(cached, client.change_tick(), &packet.event.message)
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

/// Client <- Server
pub(crate) fn receive_server_packets(
    mut client         : ResMut<RepliconClient>,
    mut game_packets   : EventWriter<GamePacket>,
    unreliable_channel : Res<EventChannel<(GamePacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(GamePacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(GamePacket, SendOrdered)>>,
    mut event_queue    : ResMut<ServerEventQueue<GamePacket>>,
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
            // extract the layered-in replicon change tick
            let Some(change_tick) = deser_bytes_partial::<RepliconTick>(&mut message)
            else { tracing::error!("failed deserializing change tick, ignoring server message"); continue; };
            let packet = GamePacket{ send_policy, message };

            match change_tick <= **replicon_tick
            {
                true  => { game_packets.send(packet); }
                false => { event_queue.insert(change_tick, packet); }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client -> Server
pub(crate) fn send_client_packets(
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

/// Server <- Client
pub(crate) fn receive_client_packets(
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
