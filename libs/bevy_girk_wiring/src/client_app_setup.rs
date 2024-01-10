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

fn track_replication_initialized(
    In(just_connected) : In<bool>,
    mut initialized    : Local<bool>,
    client             : Res<bevy_renet::renet::RenetClient>,
    tick               : Res<RepliconTick>
) -> Progress
{
    // reset initialized
    if just_connected
    || client.is_disconnected()
    || client.is_connecting()
    {
        *initialized = false;
    }

    // set initialized
    if client.is_connected()
    && tick.is_changed()
    {
        *initialized = true;
    }

    match *initialized
    {
        false => Progress{ done: 0, total: 1 },
        true  => Progress{ done: 1, total: 1 },
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reinitialize_client(client_fw_command_sender: Res<Sender<ClientFwCommand>>)
{
    if let Err(_) = client_fw_command_sender.send(ClientFwCommand::ReInitialize)
    { tracing::error!("failed commanding client framework to reinitialize"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_transport_errors(mut transport_errors: EventReader<renet::transport::NetcodeTransportError>)
{
    for error in transport_errors.read()
    {
        tracing::warn!(?error);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Set up a client app with the `bevy_girk` client framework.
///
/// REQUIREMENTS:
/// - `bevy::time::TimePlugin`.
pub fn prepare_client_app_framework(client_app: &mut App, client_fw_config: ClientFwConfig) -> Sender<ClientFwCommand>
{
    // prepare message channels
    let (game_packet_sender, game_packet_receiver)             = new_channel::<GamePacket>();
    let (client_packet_sender, client_packet_receiver)         = new_channel::<ClientPacket>();
    let (client_fw_command_sender, client_fw_command_receiver) = new_channel::<ClientFwCommand>();

    // prepare client app framework
    client_app
        //setup components
        .add_plugins(ClientFwPlugin)
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

/// Set up `bevy_replicon` in a client app.
pub fn prepare_client_app_replication(client_app: &mut App, client_fw_command_sender: Sender<ClientFwCommand>)
{
    // depends on client framework

    // setup client with bevy_replicon (includes bevy_renet)
    client_app
        //add bevy_replicon client
        .add_plugins(ReplicationPlugins
            .build()
            .disable::<ServerPlugin>())
        //enable replication repair for reconnects
        //todo: add custom input-status tracking mechanism w/ custom prespawn cleanup
        .add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: true })
        //prepare message channels
        .add_server_event_with::<EventConfig<GamePacket, SendUnreliable>, _, _>(EventType::Unreliable, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendUnordered>, _, _>(EventType::Unordered, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendOrdered>, _, _>(EventType::Ordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnreliable>, _, _>(EventType::Unreliable, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnordered>, _, _>(EventType::Unordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendOrdered>, _, _>(EventType::Ordered, dummy, dummy)
        //add framework command endpoint for use by connection controls
        .insert_resource(client_fw_command_sender)
        //message receiving
        .add_systems(PreUpdate,
            (
                // reinitialize when disconnected and not at game end
                // - at game end the server will shut down, we don't want to reinitialize in that case
                reinitialize_client
                    .run_if(bevy_renet::client_just_disconnected())
                    .run_if(not(in_state(ClientFwMode::End))),
                receive_server_packets
                    .run_if(bevy_renet::client_connected())
            )
                .chain()
                .after(bevy_replicon::prelude::ClientSet::Receive)
                .before(ClientFwTickSetPrivate::FwStart),
        )
        //client logic
        .configure_sets(Update, ClientFwSet.before(iyes_progress::prelude::AssetsTrackProgress))
        //track connection status
        .add_systems(Update, track_connection_state.track_progress().in_set(ClientFwLoadingSet))
        //track whether the first replication message post-connect has been received
        //- note: we spawn an empty replicated entity in the game framework to ensure an init message is always sent
        //        when a client connects (for reconnects we assume the user did not despawn that entity, or spawned
        //        some other entity/entities)
        .add_systems(Update,
            (
                bevy_renet::client_just_connected()
                    .pipe(track_replication_initialized)
                    .track_progress()
            )
                .in_set(ClientFwLoadingSet)
        )
        //message sending
        .add_systems(PostUpdate,
            //todo: if the client is disconnected then messages will pile up until reconnected; it is probably
            //      better to drop those messages, but need to do a full analysis to establish a precise framework
            //      for handling reconnects and resynchronization
            //      - one problem here is the client sends ClientInitProgress messages while loading, and dropping
            //        those may cause problems (resend them periodically?) (waiting for replication init solves this,
            //        the client won't be fully initialized until after connected)
            //note that bevy_replicon events also internally pile up while waiting, but since we add a layer between
            //  our events and replicon here, and both systems currently use client_connected(), there should not be
            //  any race conditions where messages can hang as replicon events
            send_client_packets
                .run_if(bevy_renet::client_connected())
                .after(ClientFwTickSetPrivate::FwEnd)
                .before(bevy_replicon::prelude::ClientSet::Send)
        )
        //message cleanup while disconnected
        .add_systems(PostUpdate,
            clear_client_packets
                .run_if(not(bevy_renet::client_connected()))
                .after(ClientFwTickSetPrivate::FwEnd)
                .before(bevy_replicon::prelude::ClientSet::Send)
        )
        //log transport errors
        //- note that these will be logged out of order, since we need to collect both receive and send errors
        .add_systems(Last, log_transport_errors)
        ;
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up a renet client and enable renet reconnects.
///
/// Note that this method simply waits for a new connect pack to appear, then sets up a renet client.
/// For requesting a new connect pack when disconnected, see the `bevy_girk_client_instance` crate.
pub fn prepare_client_app_network(client_app: &mut App, connect_pack: RenetClientConnectPack)
{
    client_app.insert_resource(connect_pack)
        .add_systems(Startup, setup_renet_client.map(Result::unwrap))
        .add_systems(PreUpdate,
            setup_renet_client.map(Result::unwrap)
                .before(bevy_renet::RenetReceive)
                .run_if(not(bevy_renet::client_just_disconnected()))  //allow at least 1 tick while disconnected
                .run_if(bevy_renet::client_disconnected())
                .run_if(resource_exists::<RenetClientConnectPack>())
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up a client app to hook into the `bevy_girk` backend.
/// - Sets up the client framework.
/// - Sets up replication.
/// - Sets up the renet client.
//todo: 'backend' is wrong term here?
pub fn prepare_client_app_backend(
    client_app       : &mut App,
    client_fw_config : ClientFwConfig,
    connect_pack     : RenetClientConnectPack,
){
    let client_fw_command_sender = prepare_client_app_framework(client_app, client_fw_config);
    prepare_client_app_replication(client_app, client_fw_command_sender);
    prepare_client_app_network(client_app, connect_pack);
}

//-------------------------------------------------------------------------------------------------------------------
