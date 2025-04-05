//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::{SendOrdered, SendUnordered, SendUnreliable};
use bevy_girk_game_fw::{ClientPacket, GamePacket};
use renet2::{ChannelConfig, SendType};

//standard shortcuts
use std::{marker::PhantomData, time::Duration};

//-------------------------------------------------------------------------------------------------------------------

/// Holds a server's channel ID for `T`.
#[derive(Resource)]
pub struct EventChannel<T>
{
    pub id: u8,
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
// TODO: resend_time and max bytes are currently hard-coded in renet2
pub fn prepare_network_channels(
    world: &mut World,
    server_channels: &mut Vec<ChannelConfig>,
    client_channels: &mut Vec<ChannelConfig>,
    resend_time: Duration
)
{
    let unreliable_game_packet = server_channels.len() as u8;
    server_channels.push(ChannelConfig{
        channel_id: unreliable_game_packet,
        max_memory_usage_bytes: 5 * 1024 * 1024,
        send_type: SendType::Unreliable
    });
    let unordered_game_packet = server_channels.len() as u8;
    server_channels.push(ChannelConfig{
        channel_id: unordered_game_packet,
        max_memory_usage_bytes: 5 * 1024 * 1024,
        send_type: SendType::ReliableUnordered { resend_time }
    });
    let ordered_game_packet = server_channels.len() as u8;
    server_channels.push(ChannelConfig{
        channel_id: ordered_game_packet,
        max_memory_usage_bytes: 5 * 1024 * 1024,
        send_type: SendType::ReliableOrdered { resend_time }
    });

    let unreliable_client_packet = client_channels.len() as u8;
    client_channels.push(ChannelConfig{
        channel_id: unreliable_client_packet,
        max_memory_usage_bytes: 5 * 1024 * 1024,
        send_type: SendType::Unreliable
    });
    let unordered_client_packet = client_channels.len() as u8;
    client_channels.push(ChannelConfig{
        channel_id: unordered_client_packet,
        max_memory_usage_bytes: 5 * 1024 * 1024,
        send_type: SendType::ReliableUnordered { resend_time }
    });
    let ordered_client_packet = client_channels.len() as u8;
    client_channels.push(ChannelConfig{
        channel_id: ordered_client_packet,
        max_memory_usage_bytes: 5 * 1024 * 1024,
        send_type: SendType::ReliableOrdered { resend_time }
    });

    world.insert_resource(EventChannel::<(GamePacket, SendUnreliable)>::new(unreliable_game_packet));
    world.insert_resource(EventChannel::<(GamePacket, SendUnordered)>::new(unordered_game_packet));
    world.insert_resource(EventChannel::<(GamePacket, SendOrdered)>::new(ordered_game_packet));
    world.insert_resource(EventChannel::<(ClientPacket, SendUnreliable)>::new(unreliable_client_packet));
    world.insert_resource(EventChannel::<(ClientPacket, SendUnordered)>::new(unordered_client_packet));
    world.insert_resource(EventChannel::<(ClientPacket, SendOrdered)>::new(ordered_client_packet));
}

//-------------------------------------------------------------------------------------------------------------------
