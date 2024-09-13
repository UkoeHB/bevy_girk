//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::{SendOrdered, SendUnordered, SendUnreliable};
use bevy_replicon::prelude::{ChannelKind, RepliconChannel, RepliconChannels};
use bevy_girk_game_fw::{ClientPacket, GamePacket};

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
