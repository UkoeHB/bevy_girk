//local shortcuts
use bevy_girk_game_fw::{ClientPacket, GamePacket};
use bevy_girk_utils::{deser_bytes_partial, SendUnreliable, SendUnordered, SendOrdered};

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_wiring_common::EventChannel;
use bevy_replicon::client::{ClientSet, ServerUpdateTick};
use bevy_replicon::core::common_conditions::client_connected;
use bevy_replicon::core::replicon_tick::RepliconTick;
use bevy_replicon::prelude::{
    ChannelKind, RepliconClient
};
use ordered_multimap::ListOrderedMultimap;

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Applies all queued events if their tick is less or equal to `ServerUpdateTick`.
fn pop_game_packet_queue(
    init_tick: Res<ServerUpdateTick>,
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

/// Client <- Server
fn receive_server_packets(
    mut client         : ResMut<RepliconClient>,
    mut game_packets   : EventWriter<GamePacket>,
    unreliable_channel : Res<EventChannel<(GamePacket, SendUnreliable)>>,
    unordered_channel  : Res<EventChannel<(GamePacket, SendUnordered)>>,
    ordered_channel    : Res<EventChannel<(GamePacket, SendOrdered)>>,
    mut packet_queue   : ResMut<GamePacketQueue>,
    replicon_tick      : Res<ServerUpdateTick>,
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
