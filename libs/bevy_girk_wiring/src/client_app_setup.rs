//local shortcuts
use crate::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_replicon::prelude::*;
use iyes_progress::*;

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Dummy system, does nothing.
fn dummy() {}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_connection_state(client: Res<bevy_renet::renet::RenetClient>) -> Progress
{
    if client.is_disconnected() { return Progress{ done: 0, total: 2 }; }
    if client.is_connecting()   { return Progress{ done: 1, total: 2 }; }
    if client.is_connected()    { return Progress{ done: 2, total: 2 }; }

    Progress{ done: 0, total: 2 }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reinitialize_client(client_fw_command_sender: Res<Sender<ClientFWCommand>>)
{
    if let Err(_) = client_fw_command_sender.send(ClientFWCommand::ReInitialize)
    { tracing::error!("failed commanding client framework to reinitialize"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Set up a client app with the bevy_girk client framework.
///
/// REQUIREMENTS:
/// - `bevy::time::TimePlugin`.
pub fn prepare_client_app_framework(client_app: &mut App, client_fw_config: ClientFWConfig) -> Sender<ClientFWCommand>
{
    // prepare message channels
    let (game_packet_sender, game_packet_receiver)             = new_channel::<GamePacket>();
    let (client_packet_sender, client_packet_receiver)         = new_channel::<ClientPacket>();
    let (client_fw_command_sender, client_fw_command_receiver) = new_channel::<ClientFWCommand>();

    // prepare client app framework
    client_app
        //setup components
        .add_plugins(ClientFWPlugin)
        //client framework
        .insert_resource(client_fw_config)
        .insert_resource(game_packet_sender)
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_packet_receiver)
        .insert_resource(client_fw_command_receiver);

    client_fw_command_sender
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_client_app_replication(client_app: &mut App, client_fw_command_sender: Sender<ClientFWCommand>)
{
    // depends on client framework

    // setup client with bevy_replicon (includes bevy_renet)
    client_app
        //add bevy_replicon client
        .add_plugins(ReplicationPlugins
            .build()
            .disable::<ServerPlugin>())
        //bracket the client logic with message receiving/sending (client logic is in `Update`)
        .add_systems(PreUpdate,
            receive_server_messages
                //todo: if the client is disconnected then messages will pile up until reconnected; it is probably
                //      better to drop those messages, but need to do a full analysis to establish a precise framework
                //      for handling reconnects and resynchronization
                .run_if(bevy_renet::client_connected())
                .after(bevy_replicon::prelude::ClientSet::Receive)
                .before(ClientFWTickSetPrivate::FWStart),
        )
        .configure_sets(Update, ClientFWSet.before(iyes_progress::prelude::AssetsTrackProgress))
        .add_systems(PostUpdate,
            send_client_messages
                .run_if(bevy_renet::client_connected())
                .after(ClientFWTickSetPrivate::FWEnd)
                .before(bevy_replicon::prelude::ClientSet::Send)
        )
        //prepare message channels
        .add_server_event_with::<EventConfig<GamePacket, SendUnreliable>, _, _>(EventType::Unreliable, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendUnordered>, _, _>(EventType::Unordered, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendOrdered>, _, _>(EventType::Ordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnreliable>, _, _>(EventType::Unreliable, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnordered>, _, _>(EventType::Unordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendOrdered>, _, _>(EventType::Ordered, dummy, dummy)
        //add framework command endpoint for use by connection controls
        .insert_resource(client_fw_command_sender)
        //track connection status
        .add_systems(Update, track_connection_state.track_progress().in_set(ClientFWLoadingSet))
        .add_systems(Update,
            reinitialize_client
                .before(ClientFWSet)
                .run_if(bevy_renet::client_just_disconnected())
        );
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_client_app_network(client_app: &mut App, connect_pack: RenetClientConnectPack)
{
    client_app.insert_resource(connect_pack)
        .add_systems(Startup, setup_renet_client)
        .add_systems(Update,
            setup_renet_client
                .before(ClientFWSet)
                .run_if(bevy_renet::client_just_disconnected())
        );
}

//-------------------------------------------------------------------------------------------------------------------
