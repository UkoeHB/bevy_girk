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

fn log_just_connected()
{
   tracing::info!("client connected");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_just_disconnected()
{
   tracing::info!("client disconnected");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_transport_errors(mut errors: EventReader<renet::transport::NetcodeTransportError>)
{
    for error in errors.read()
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
    let (client_fw_command_sender, client_fw_command_receiver) = new_channel::<ClientFwCommand>();

    // prepare client app framework
    client_app
        //setup components
        .add_plugins(ClientFwPlugin)
        //client framework
        .insert_resource(client_fw_config)
        .insert_resource(client_fw_command_receiver);

    client_fw_command_sender
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up `bevy_replicon` in a client app.
pub fn prepare_client_app_replication(client_app: &mut App, client_fw_command_sender: Sender<ClientFwCommand>)
{
    // depends on client framework

    // prepare channels
    prepare_framework_channels(client_app);

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
        //- note: the event types specified here do nothing
        .add_server_event_with::<GamePacket, _, _>(EventType::Unreliable, dummy, receive_server_packets)
        .add_client_event_with::<ClientPacket, _, _>(EventType::Unreliable, send_client_packets, dummy)
        //add framework command endpoint for use by connection controls
        .insert_resource(client_fw_command_sender)
        //message receiving
        .configure_sets(PreUpdate,
            ClientFwTickSetPrivate::FwStart
                .after(bevy_replicon::prelude::ClientSet::Receive)
        )
        //connection event handling
        .add_systems(PreUpdate,
            (
                log_just_connected.run_if(bevy_renet::client_just_connected()),
                log_just_disconnected.run_if(bevy_renet::client_just_disconnected()),
                // reinitialize when disconnected and not at game end
                // - at game end the server will shut down, we don't want to reinitialize in that case
                // - note: there should not be a race condition here because the client fw only moves to End if
                //   the server sends an End mode message, but this will only be called in a tick where we are disconnected
                //   and hence won't receive a game End mode message
                reinitialize_client
                    .run_if(bevy_renet::client_just_disconnected())
                    .run_if(not(in_state(ClientFwMode::End))),
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
        .configure_sets(PostUpdate,
            ClientFwTickSetPrivate::FwEnd
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
