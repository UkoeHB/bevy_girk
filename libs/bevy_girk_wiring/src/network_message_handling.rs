//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_renet::renet::{ClientId, RenetClient, RenetServer, transport::NetcodeClientTransport};
use bevy_replicon::network_event::{ClientEventChannel, EventType, ServerEventChannel};

//standard shortcuts
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

// Maximum number of client messages that the server will accept per tick.
const MAX_CLIENT_MESSAGES_PER_TICK: u16 = 64;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) trait SendPolicyConfig {}

impl SendPolicyConfig for SendUnreliable {}
impl SendPolicyConfig for SendUnordered {}
impl SendPolicyConfig for SendOrdered {}

#[derive(Debug, Event)]
pub(crate) struct EventConfig<T, P: SendPolicyConfig>
{
    _p1: PhantomData<T>,
    _p2: PhantomData<P>
}

//-------------------------------------------------------------------------------------------------------------------

/// Server -> Client
pub(crate) fn send_server_packets(
    server_output_receiver : Res<Receiver<GamePacket>>,
    mut server             : ResMut<RenetServer>,
    unreliable_channel     : Res<ServerEventChannel<EventConfig<GamePacket, SendUnreliable>>>,
    unordered_channel      : Res<ServerEventChannel<EventConfig<GamePacket, SendUnordered>>>,
    ordered_channel        : Res<ServerEventChannel<EventConfig<GamePacket, SendOrdered>>>,
){
    while let Some(game_packet) = server_output_receiver.try_recv()
    {
        // send message directly to client
        let client_id = ClientId::from_raw(game_packet.client_id as u64);

        match game_packet.send_policy
        {
            EventType::Unreliable => server.send_message(client_id, *unreliable_channel, game_packet.message),
            EventType::Unordered  => server.send_message(client_id, *unordered_channel, game_packet.message),
            EventType::Ordered    => server.send_message(client_id, *ordered_channel, game_packet.message)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client <- Server
pub(crate) fn receive_server_packets(
    mut client          : ResMut<RenetClient>,
    client_transport    : Res<NetcodeClientTransport>,
    client_input_sender : Res<Sender<GamePacket>>,
    unreliable_channel  : Res<ServerEventChannel<EventConfig<GamePacket, SendUnreliable>>>,
    unordered_channel   : Res<ServerEventChannel<EventConfig<GamePacket, SendUnordered>>>,
    ordered_channel     : Res<ServerEventChannel<EventConfig<GamePacket, SendOrdered>>>,
){
    // receive ordered messages first since they are probably oldest
    let client_id = client_transport.client_id() as ClientIdType;

    for &(channel_id, send_policy) in
        [
            (Into::<u8>::into(*ordered_channel), EventType::Ordered),
            (Into::<u8>::into(*unordered_channel), EventType::Unordered),
            (Into::<u8>::into(*unreliable_channel), EventType::Unreliable),
        ].iter()
    {
        while let Some(message) = client.receive_message(channel_id)
        {
            client_input_sender
                .send( GamePacket{ client_id, send_policy, message } )
                .expect("client input receiver is missing");
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn clear_client_packets(client_output_receiver: Res<Receiver<ClientPacket>>)
{
    while let Some(_client_packet) = client_output_receiver.try_recv()
    {
        tracing::warn!("dropping client packet while disconnected");
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client -> Server
pub(crate) fn send_client_packets(
    client_output_receiver : Res<Receiver<ClientPacket>>,
    mut client             : ResMut<RenetClient>,
    unreliable_channel     : Res<ClientEventChannel<EventConfig<ClientPacket, SendUnreliable>>>,
    unordered_channel      : Res<ClientEventChannel<EventConfig<ClientPacket, SendUnordered>>>,
    ordered_channel        : Res<ClientEventChannel<EventConfig<ClientPacket, SendOrdered>>>,
){
    while let Some(client_packet) = client_output_receiver.try_recv()
    {
        match client_packet.send_policy
        {
            EventType::Unreliable => client.send_message(*unreliable_channel, client_packet.request),
            EventType::Unordered  => client.send_message(*unordered_channel, client_packet.request),
            EventType::Ordered    => client.send_message(*ordered_channel, client_packet.request)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Server <- Client
pub(crate) fn receive_client_packets(
    mut server          : ResMut<RenetServer>,
    server_input_sender : Res<Sender<ClientPacket>>,
    unreliable_channel  : Res<ClientEventChannel<EventConfig<ClientPacket, SendUnreliable>>>,
    unordered_channel   : Res<ClientEventChannel<EventConfig<ClientPacket, SendUnordered>>>,
    ordered_channel     : Res<ClientEventChannel<EventConfig<ClientPacket, SendOrdered>>>,
    registered_clients  : Res<ClientEntityMap>
){
    for client_id in server.clients_id()
    {
        // ignore unregistered client ids
        // - if this error is encountered, then you are issuing connect tokens to clients that weren't registered
        let Some(_) = registered_clients.get_entity(client_id.raw() as ClientIdType)
        else { tracing::error!("ignoring renet server client with unknown id"); continue; };

        // receive ordered messages first since they are probably oldest
        let mut messages_count = 0;

        for &(channel_id, send_policy) in
            [
                (Into::<u8>::into(*ordered_channel), EventType::Ordered),
                (Into::<u8>::into(*unordered_channel), EventType::Unordered),
                (Into::<u8>::into(*unreliable_channel), EventType::Unreliable),
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
                server_input_sender
                    .send( ClientPacket{ client_id: client_id.raw() as ClientIdType, send_policy, request } )
                    .expect("server input receiver is missing");
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
