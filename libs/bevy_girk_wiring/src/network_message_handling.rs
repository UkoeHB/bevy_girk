//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_renet::renet::{ClientId, RenetClient, RenetServer, transport::NetcodeClientTransport};
use bevy_replicon::network_event::{EventChannel, EventType};

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
pub(crate) fn send_server_messages(
    server_output_receiver : Res<Receiver<GamePacket>>,
    mut server             : ResMut<RenetServer>,
    unreliable_channel     : Res<EventChannel<EventConfig<GamePacket, SendUnreliable>>>,
    unordered_channel      : Res<EventChannel<EventConfig<GamePacket, SendUnordered>>>,
    ordered_channel        : Res<EventChannel<EventConfig<GamePacket, SendOrdered>>>,
){
    while let Some(game_packet) = server_output_receiver.try_recv()
    {
        // send message directly to client
        let client_id      = ClientId::from_raw(game_packet.client_id as u64);
        let serialized_msg = ser_msg(&game_packet.message);

        match game_packet.send_policy
        {
            EventType::Unreliable => server.send_message(client_id, *unreliable_channel, serialized_msg),
            EventType::Unordered  => server.send_message(client_id, *unordered_channel, serialized_msg),
            EventType::Ordered    => server.send_message(client_id, *ordered_channel, serialized_msg)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client <- Server
pub(crate) fn receive_server_messages(
    mut client          : ResMut<RenetClient>,
    client_transport    : Res<NetcodeClientTransport>,
    client_input_sender : Res<Sender<GamePacket>>,
    unreliable_channel  : Res<EventChannel<EventConfig<GamePacket, SendUnreliable>>>,
    unordered_channel   : Res<EventChannel<EventConfig<GamePacket, SendUnordered>>>,
    ordered_channel     : Res<EventChannel<EventConfig<GamePacket, SendOrdered>>>,
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
            let message = deser_msg::<GameMessage>(&message).expect("the server sent an invalid message");

            client_input_sender.send( GamePacket{ client_id, send_policy, message } )
                .expect("client input receiver is missing");
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client -> Server
pub(crate) fn send_client_messages(
    client_output_receiver : Res<Receiver<ClientPacket>>,
    mut client             : ResMut<RenetClient>,
    unreliable_channel     : Res<EventChannel<EventConfig<ClientPacket, SendUnreliable>>>,
    unordered_channel      : Res<EventChannel<EventConfig<ClientPacket, SendUnordered>>>,
    ordered_channel        : Res<EventChannel<EventConfig<ClientPacket, SendOrdered>>>,
){
    while let Some(client_packet) = client_output_receiver.try_recv()
    {
        let serialized_msg = ser_msg(&client_packet.message);

        match client_packet.send_policy
        {
            EventType::Unreliable => client.send_message(*unreliable_channel, serialized_msg),
            EventType::Unordered  => client.send_message(*unordered_channel, serialized_msg),
            EventType::Ordered    => client.send_message(*ordered_channel, serialized_msg)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Server <- Client
pub(crate) fn receive_client_messages(
    mut server          : ResMut<RenetServer>,
    server_input_sender : Res<Sender<ClientPacket>>,
    unreliable_channel  : Res<EventChannel<EventConfig<ClientPacket, SendUnreliable>>>,
    unordered_channel   : Res<EventChannel<EventConfig<ClientPacket, SendUnordered>>>,
    ordered_channel     : Res<EventChannel<EventConfig<ClientPacket, SendOrdered>>>,
    registered_clients  : Res<ClientEntityMap>
){
    for client_id in server.clients_id()
    {
        // ignore unregistered client ids
        let Some(_) = registered_clients.get_entity(client_id.raw() as ClientIdType)
        else { tracing::warn!("ignoring renet server client with unknown id"); continue; };

        // receive ordered messages first since they are probably oldest
        let mut messages_count = 0;

        for &(channel_id, send_policy) in
            [
                (Into::<u8>::into(*ordered_channel), EventType::Ordered),
                (Into::<u8>::into(*unordered_channel), EventType::Unordered),
                (Into::<u8>::into(*unreliable_channel), EventType::Unreliable),
            ].iter()
        {
            while let Some(message) = server.receive_message(client_id, channel_id)
            {
                // if too many messages were received this tick, ignore the remaining messages
                messages_count += 1;
                if messages_count > MAX_CLIENT_MESSAGES_PER_TICK
                {
                    tracing::trace!(?client_id, channel_id, messages_count, "client exceeded max messages per tick");
                    continue;
                }

                // deserialize message and send packet into server
                let Some(message) = deser_msg::<ClientMessage>(&message)
                else { tracing::trace!(?client_id, "client message failed to deserialize"); continue; };
                server_input_sender
                    .send( ClientPacket{ client_id: client_id.raw() as ClientIdType, send_policy, message } )
                    .expect("server input receiver is missing");
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
