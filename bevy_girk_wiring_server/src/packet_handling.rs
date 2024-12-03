//local shortcuts
use bevy_girk_game_fw::{ClientPacket, GameFwClients, GamePacket};

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::{SendOrdered, SendUnordered, SendUnreliable};
use bevy_girk_wiring_common::EventChannel;
use bevy_replicon::core::common_conditions::server_running;
use bevy_replicon::core::replicon_tick::RepliconTick;
use bevy_replicon::prelude::{
    ClientId, ChannelKind, FromClient, ReplicatedClients,
    RepliconServer, SendMode, ToClients
};
use bevy_replicon::server::ServerSet;
use bincode::Options;
use bytes::Bytes;

//standard shortcuts
use std::collections::HashMap;

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
fn serialize_bytes_with_update_tick(
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

/// Server -> Client
fn send_server_packets(
    mut game_packets   : ResMut<Events<ToClients<GamePacket>>>,
    client_cache       : Res<ReplicatedClients>,
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
        let Ok(message) = serialize_bytes_with_update_tick(cached, client.update_tick(), &packet.event.message)
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
