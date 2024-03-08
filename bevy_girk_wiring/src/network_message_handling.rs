//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer, SendType};
use bevy_replicon::network_event::EventType;
use bevy_replicon::prelude::{
    ClientCache, FromClient, NetworkChannels, RepliconTick, SendMode, ServerEventQueue, ToClients
};
use bincode::Options;
use bytes::Bytes;

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
    app.init_resource::<NetworkChannels>();

    let mut channels = app.world.resource_mut::<NetworkChannels>();

    let unordered = SendType::ReliableUnordered{ resend_time };
    let ordered   = SendType::ReliableOrdered{ resend_time };

    let unreliable_game_packet   = channels.create_server_channel(EventType::Unreliable.into());
    let unordered_game_packet    = channels.create_server_channel(unordered.clone());
    let ordered_game_packet      = channels.create_server_channel(ordered.clone());
    let unreliable_client_packet = channels.create_client_channel(EventType::Unreliable.into());
    let unorderd_client_packet   = channels.create_client_channel(unordered);
    let ordered_client_packet    = channels.create_client_channel(ordered);

    app
        .insert_resource(EventChannel::<(GamePacket, SendUnreliable)>::new(unreliable_game_packet))
        .insert_resource(EventChannel::<(GamePacket, SendUnordered)>::new(unordered_game_packet))
        .insert_resource(EventChannel::<(GamePacket, SendOrdered)>::new(ordered_game_packet))
        .insert_resource(EventChannel::<(ClientPacket, SendUnreliable)>::new(unreliable_client_packet))
        .insert_resource(EventChannel::<(ClientPacket, SendUnordered)>::new(unorderd_client_packet))
        .insert_resource(EventChannel::<(ClientPacket, SendOrdered)>::new(ordered_client_packet));
}

//-------------------------------------------------------------------------------------------------------------------

/// Server -> Client
pub(crate) fn send_server_packets(
    mut game_packets   : ResMut<Events<ToClients<GamePacket>>>,
    client_cache       : Res<ClientCache>,
    mut server         : ResMut<RenetServer>,
    unreliable_channel : Res<EventChannel<(GamePacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(GamePacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(GamePacket, SendOrdered)>>,
){
    let mut prev: Option<*const u8> = None;
    let mut buffer: Option<SerializedGamePacket> = None;

    for packet in game_packets.drain()
    {
        let SendMode::Direct(client_id) = packet.mode else { panic!("invalid game packet send mode"); };

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
            EventType::Unreliable => server.send_message(client_id, unreliable_channel.id, message.bytes.clone()),
            EventType::Unordered  => server.send_message(client_id, unordered_channel.id, message.bytes.clone()),
            EventType::Ordered    => server.send_message(client_id, ordered_channel.id, message.bytes.clone())
        }

        // cache this message for possible re-use with the next client
        buffer = Some(message);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client <- Server
pub(crate) fn receive_server_packets(
    mut client         : ResMut<RenetClient>,
    mut game_packets   : EventWriter<GamePacket>,
    unreliable_channel : Res<EventChannel<(GamePacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(GamePacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(GamePacket, SendOrdered)>>,
    mut event_queue    : ResMut<ServerEventQueue<GamePacket>>,
    replicon_tick      : Res<RepliconTick>,
){
    // receive ordered messages first since they are probably oldest
    for &(channel_id, send_policy) in
        [
            (Into::<u8>::into(ordered_channel.id), EventType::Ordered),
            (Into::<u8>::into(unordered_channel.id), EventType::Unordered),
            (Into::<u8>::into(unreliable_channel.id), EventType::Unreliable),
        ].iter()
    {
        while let Some(mut message) = client.receive_message(channel_id)
        {
            // extract the layered-in replicon change tick
            let Some(change_tick) = deser_bytes_partial::<RepliconTick>(&mut message)
            else { tracing::error!("failed deserializing change tick, ignoring server message"); continue; };
            let packet = GamePacket{ send_policy, message };

            match change_tick <= *replicon_tick
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
    mut client         : ResMut<RenetClient>,
    unreliable_channel : Res<EventChannel<(ClientPacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(ClientPacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(ClientPacket, SendOrdered)>>,
){
    for client_packet in client_packets.drain()
    {
        match client_packet.send_policy
        {
            EventType::Unreliable => client.send_message(unreliable_channel.id, client_packet.request),
            EventType::Unordered  => client.send_message(unordered_channel.id, client_packet.request),
            EventType::Ordered    => client.send_message(ordered_channel.id, client_packet.request)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Server <- Client
pub(crate) fn receive_client_packets(
    mut server         : ResMut<RenetServer>,
    mut client_packets : EventWriter<FromClient<ClientPacket>>,
    unreliable_channel : Res<EventChannel<(ClientPacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(ClientPacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(ClientPacket, SendOrdered)>>,
    clients            : Res<GameFwClients>
){
    for client_id in server.clients_id()
    {
        // ignore unregistered client ids
        // - if this error is encountered, then you are issuing connect tokens to clients that weren't registered
        if !clients.contains(&client_id) { tracing::error!(?client_id, "ignoring client with unknown id"); continue; };

        // receive ordered messages first since they are probably oldest
        let mut messages_count = 0;

        for &(channel_id, send_policy) in
            [
                (Into::<u8>::into(ordered_channel.id), EventType::Ordered),
                (Into::<u8>::into(unordered_channel.id), EventType::Unordered),
                (Into::<u8>::into(unreliable_channel.id), EventType::Unreliable),
            ].iter()
        {
            while let Some(request) = server.receive_message(client_id, channel_id)
            {
                // if too many messages were received this tick, ignore the remaining messages
                messages_count += 1;
                if messages_count > MAX_CLIENT_MESSAGES_PER_TICK
                {
                    tracing::trace!(?client_id, channel_id, messages_count, "client exceeded max messages per tick");
                    continue;
                }

                // send packet into server
                client_packets.send(FromClient{client_id, event: ClientPacket{ send_policy, request } });
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
