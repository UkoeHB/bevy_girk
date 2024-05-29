//local shortcuts
use crate::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_renet::{client_disconnected, client_just_connected, client_just_disconnected};
use bevy_replicon::client::ServerInitTick;
use bevy_replicon::prelude::{
    ClientSet, RepliconPlugins, ServerPlugin
};
use bevy_replicon_renet::RepliconRenetClientPlugin;
use bevy_replicon_repair::AppReplicationRepairExt;
use iyes_progress::*;
use renet::{transport::NetcodeTransportError, RenetClient};

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_connection_state(client: Res<RenetClient>) -> Progress
{
    if client.is_disconnected() { return Progress{ done: 0, total: 2 }; }
    if client.is_connecting()   { return Progress{ done: 1, total: 2 }; }
    if client.is_connected()    { return Progress{ done: 2, total: 2 }; }

    Progress{ done: 0, total: 2 }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_initialization_state(mode: Res<State<ClientFwMode>>) -> Progress
{
    match **mode
    {
        ClientFwMode::Connecting => Progress{ done: 0, total: 2 },
        ClientFwMode::Syncing    => Progress{ done: 1, total: 2 },
        ClientFwMode::Init       => Progress{ done: 2, total: 2 },
        _                        => Progress{ done: 2, total: 2 },
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_replication_initialized(
    In(just_connected) : In<bool>,
    mut initialized    : Local<bool>,
    client             : Res<RenetClient>,
    tick               : Res<ServerInitTick>
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
    // - note: does nothing if the tick changes multiple times after connecting
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

fn reinitialize_client(command_sender: Res<Sender<ClientFwCommand>>)
{
    if let Err(_) = command_sender.send(ClientFwCommand::ReInitialize)
    { tracing::error!("failed commanding client framework to reinitialize"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn set_client_syncing(mut client_fw_mode: ResMut<NextState<ClientFwMode>>)
{
    tracing::info!("synchronizing client");
    client_fw_mode.set(ClientFwMode::Syncing);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn set_client_init(mut client_fw_mode: ResMut<NextState<ClientFwMode>>)
{
    tracing::info!("initializing client");
    client_fw_mode.set(ClientFwMode::Init);
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

fn log_transport_errors(mut errors: EventReader<NetcodeTransportError>)
{
    for error in errors.read()
    {
        tracing::warn!(?error);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for setting up a `bevy_girk` client app.
#[derive(Debug)]
pub struct GirkClientConfig
{
    /// Config for the client framework.
    pub client_fw_config: ClientFwConfig,
    /// Resend time for client messages within `renet`.
    pub resend_time: Duration,
    /// Client pack for the initial `renet` connection attempt.
    pub connect_pack: RenetClientConnectPack,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a client app with the `bevy_girk` client framework.
///
/// REQUIREMENTS:
/// - `bevy::time::TimePlugin`.
pub fn prepare_client_app_framework(client_app: &mut App, client_fw_config: ClientFwConfig) -> Sender<ClientFwCommand>
{
    // prepare message channels
    let (command_sender, command_receiver) = new_channel::<ClientFwCommand>();

    // prepare client app framework
    client_app
        //setup components
        .add_plugins(ClientFwPlugin)
        //client framework
        .insert_resource(client_fw_config)
        .insert_resource(command_receiver);

    command_sender
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up `bevy_replicon` in a client app.
pub fn prepare_client_app_replication(
    client_app     : &mut App,
    command_sender : Sender<ClientFwCommand>,
    resend_time    : Duration,
){
    // depends on client framework

    // prepare channels
    prepare_network_channels(client_app, resend_time);

    // setup client with bevy_replicon
    client_app
        //add framework command endpoint for use by connection controls
        .insert_resource(command_sender)
        //add bevy_replicon client
        .add_plugins(RepliconPlugins
            .build()
            .disable::<ServerPlugin>())
        .add_plugins(RepliconRenetClientPlugin)
        //enable replication repair for reconnects
        //todo: add custom input-status tracking mechanism w/ custom prespawn cleanup
        .add_plugins(bevy_replicon_repair::ClientPlugin{ cleanup_prespawns: true })
        //prepare event handling
        .add_plugins(ClientEventHandlingPlugin)
        //register GameInitProgress for replication
        .replicate_repair::<GameInitProgress>()

        //# PREUPDATE #
        //<-- RenetReceive {renet}: collects network packets
        //<-- ClientSet::ReceivePackets {replicon}: collects renet packets
        //<-- ClientSet::ResetEvents (if client just connected) {replicon}: ensures client and server messages
        //    don't leak across a reconnect
        //<-- ClientSet::Receive {replicon}: collects replication messages
        //<-- ClientRepairSet (after first disconnect) {replicon_repair}: repairs replication state following a
        //    disconnect
        //TODO: ClientSet::SyncHierarchy
        //<-- ClientFwSetPrivate::FwStart {girk}: handles client fw commands and network messages, prepares the
        //    client for this tick; we do this before ClientFwSet because server messages can control the current tick's
        //    game mode (and in general determine the contents of the current tick - e.g. replicated state is applied
        //    before user logic)
        .configure_sets(PreUpdate,
            // Ordering explanation:
            // - We want `ClientFwMode::Syncing` to run for at least one tick before handling the first replication
            //   message. So we block the client set in the range [Connecting, Syncing first tick]
            ClientSet::Receive
                .run_if(not(in_state(ClientFwMode::Connecting)))
                .run_if(not(client_just_connected))
        )
        .configure_sets(PreUpdate,
            ClientFwSetPrivate::FwStart
                .after(bevy_replicon_repair::ClientRepairSet)
        )
        .add_systems(PreUpdate,
            (
                log_just_connected.run_if(client_just_connected),
                log_just_disconnected.run_if(client_just_disconnected),
                // reinitialize when disconnected and not at game end
                // - at game end the server will shut down, we don't want to reinitialize in that case
                // - note: there should not be a race condition here because the client fw only moves to End if
                //   the server sends an End mode message, but this will only be called in a tick where we are disconnected
                //   and hence won't receive a game End mode message in `ClientFwSetPrivate::FwStart` after this
                reinitialize_client
                    .run_if(client_just_disconnected)
                    .run_if(not(in_state(ClientFwMode::End))),
                // set syncing when just connected
                // - note: this will not run in the first tick of `ClientFwMode::Connecting` because we disable
                //   `setup_renet_client` for that tick (it actually takes at least 3 ticks to connect once disconnected)
                set_client_syncing
                    .run_if(client_just_connected)
                    .run_if(in_state(ClientFwMode::Connecting)),
                // set initialized when just synchronized
                // - note: this will not run in the first tick of `ClientFwMode::Syncing` because we disabled
                //   `ClientSet::Receive` for that tick, so ServerInitTick will not change (unless the user manually changes
                //   it)
                set_client_init
                    .run_if(resource_changed::<ServerInitTick>)
                    .run_if(in_state(ClientFwMode::Syncing)),
            )
                .chain()
                .after(bevy_replicon_repair::ClientRepairSet)
                .before(ClientFwSetPrivate::FwStart),
        )

        //# UPDATE #
        //<-- ClientFwSet::{Admin, Start, PreLogic, Logic, PostLogic, End} {girk}: ordinal sets for user logic
        //<-- ClientFwLoadingSet (in state ClientInitializationState::InProgress) {girk}: should contain all user
        //    loading systems (systems with `.track_progress()`)
        //<-- AssetsTrackProgress {iyes progress}: tracks progress of assets during initialization
        .add_systems(Update,
            (
                //track connection status
                track_connection_state.track_progress(),
                //track whether the first replication message post-connect has been received
                //- note: we spawn an empty replicated entity in the game framework to ensure an init message is always sent
                //        when a client connects (for reconnects we assume the user did not despawn that entity, or spawned
                //        some other entity/entities)
                client_just_connected
                    .pipe(track_replication_initialized)
                    .track_progress(),
                //track whether the client is initialized
                //- note: this leverages the fact that `iyes_progress` collects progress in PostUpdate to ensure
                //        `ClientInitializationState::Done` will not be entered until `ClientFwMode::Init` has run
                //        for at least one tick (because the client fw will not try to leave `ClientFwMode::Init` until
                //        `ClientInitializationState::Done` has been entered, which can happen in the second Init tick
                //        at earliest)
                track_initialization_state
                    .track_progress()
            )
                .in_set(ClientFwLoadingSet)
        )
        .configure_sets(Update, ClientFwSet::End.before(iyes_progress::prelude::AssetsTrackProgress))

        //# POSTUPDATE #
        //<-- ClientFwSetPrivate::FwEnd {girk}: dispatches messages to replicon, performs final tick cleanup
        //<-- ClientSet::Send (if connected) {replicon}: dispatches messages to renet (`send_client_packets`)
        //<-- ClientSet::SendPackets {replicon}: forwards packets to renet
        //<-- RenetSend {renet}: dispatches packets to the server
        .configure_sets(PostUpdate,
            ClientFwSetPrivate::FwEnd
                .before(ClientSet::Send)
        )

        //log transport errors
        //- note that these will be logged out of order, since we need to collect both receive and send errors
        .add_systems(Last, log_transport_errors)
        ;
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a renet client and enables renet reconnects.
///
/// Note that this function simply waits for a new connect pack to appear, then sets up a renet client.
/// For automatically requesting a new connect pack when disconnected, see the `bevy_girk_client_instance` crate.
pub fn prepare_client_app_network(client_app: &mut App, connect_pack: RenetClientConnectPack)
{
    client_app.insert_resource(connect_pack)
        .add_systems(PreUpdate,
            // Ordering explanation:
            // - We want `ClientFwMode::Connecting` to run for at least one tick.
            // - We want `client_just_disconnected` to return true for the first tick of `ClientFwMode::Connecting`.
            // - We don't put this in Last in case the client manually disconnects halfway through Update.
            setup_renet_client.map(Result::unwrap)
                .after(bevy_renet::RenetReceive)  //detect disconnected
                .before(ClientSet::ReceivePackets)       //add ordering constraint
                .run_if(not(client_just_disconnected))  //ignore for first full tick while disconnected
                .run_if(client_disconnected)            //poll for connect packs while disconnected
                .run_if(resource_exists::<RenetClientConnectPack>)  //check for connect pack
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a `bevy_girk` client app compatible with `bevy_girk` game apps.
/// - Sets up the client framework.
/// - Sets up replication.
/// - Sets up the renet client.
pub fn prepare_girk_client_app(client_app: &mut App, config: GirkClientConfig)
{
    let command_sender = prepare_client_app_framework(client_app, config.client_fw_config);
    prepare_client_app_replication(client_app, command_sender, config.resend_time);
    prepare_client_app_network(client_app, config.connect_pack);
}

//-------------------------------------------------------------------------------------------------------------------
