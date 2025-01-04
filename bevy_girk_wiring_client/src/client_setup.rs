//local shortcuts
use crate::{setup_renet_client, ClientConnectPack, ClientEventHandlingPlugin, ReceiveServerEventsSet};
use bevy_girk_client_fw::{
    ClientFwConfig, ClientFwLoadingSet, ClientFwPlugin, ClientFwSet, ClientFwState,
    ClientInitState, ClientInstanceState
};
use bevy_girk_client_instance::ClientInstanceCommand;
use bevy_girk_game_fw::GameInitProgress;

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::{just_entered_state, set_and_apply_state};
use bevy_girk_wiring_common::prepare_network_channels;
use bevy_renet2::prelude::{
    client_connected, client_disconnected, client_just_connected, client_just_disconnected, RenetClient
};
use bevy_renet2::netcode::{NetcodeClientTransport, NetcodeTransportError};
use bevy_replicon::client::ServerUpdateTick;
use bevy_replicon::prelude::{
    AppRuleExt, ClientSet, RepliconPlugins, ServerEventsPlugin, ServerPlugin
};
use bevy_replicon_renet2::RepliconRenetClientPlugin;
use iyes_progress::{Progress, ProgressReturningSystem};

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_connection_state(client: Option<Res<RenetClient>>) -> Progress
{
    let Some(client) = client else {
        return Progress{ done: 0, total: 1 };
    };
    
    if client.is_disconnected() { return Progress{ done: 0, total: 2 }; }
    if client.is_connecting()   { return Progress{ done: 1, total: 2 }; }
    if client.is_connected()    { return Progress{ done: 2, total: 2 }; }

    Progress{ done: 0, total: 2 }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_initialization_state(state: Res<State<ClientFwState>>) -> Progress
{
    match **state
    {
        ClientFwState::Setup      => Progress{ done: 0, total: 3 },
        ClientFwState::Connecting => Progress{ done: 1, total: 3 },
        ClientFwState::Syncing    => Progress{ done: 2, total: 3 },
        ClientFwState::Init       => Progress{ done: 3, total: 3 },
        _                         => Progress{ done: 3, total: 3 },
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn track_replication_initialized(
    In(just_connected) : In<bool>,
    mut initialized    : Local<bool>,
    client             : Res<RenetClient>,
    tick               : Res<ServerUpdateTick>
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

fn reinitialize_client(mut c: Commands)
{
    c.queue(ClientInstanceCommand::RequestConnectToken);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn set_client_connecting(w: &mut World)
{
    tracing::info!("connecting client");
    set_and_apply_state(w, ClientFwState::Connecting);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn set_client_syncing(w: &mut World)
{
    tracing::info!("synchronizing client");
    set_and_apply_state(w, ClientFwState::Syncing);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn set_client_init(w: &mut World)
{
    tracing::info!("initializing client");
    set_and_apply_state(w, ClientFwState::Init);
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

fn cleanup_client_resources(w: &mut World)
{
    if let Some(mut transport) = w.get_resource_mut::<NetcodeClientTransport>() {
        transport.disconnect();
    }
    w.remove_resource::<NetcodeClientTransport>();
    w.remove_resource::<RenetClient>();
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
pub struct GirkClientStartupConfig
{
    /// Resend time for client messages within `renet`.
    pub resend_time: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for initializing a `bevy_girk` client.
#[derive(Debug)]
pub struct GirkClientConfig
{
    /// Config for the client framework.
    pub client_fw_config: ClientFwConfig,
    /// Client pack for the initial `renet` connection attempt.
    pub connect_pack: ClientConnectPack,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a client app with the `bevy_girk` client framework.
///
/// Adds the following if missing:
/// - `bevy::time::TimePlugin`.
/// - `bevy::asset::AssetPlugin`.
///
/// Note: `ClientFwConfig` must be inserted separately (e.g. by the `ClientFactory`).
pub fn prepare_client_app_framework(client_app: &mut App)
{
    if !client_app.is_plugin_added::<bevy::time::TimePlugin>() {
        client_app.add_plugins(bevy::time::TimePlugin);
    }
    if !client_app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        client_app.add_plugins(bevy::state::app::StatesPlugin);
    }

    // prepare client app framework
    client_app
        //setup components
        .add_plugins(ClientFwPlugin);
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up `bevy_replicon` in a client app.
pub fn prepare_client_app_replication(
    client_app  : &mut App,
    resend_time : Duration,
){
    // depends on client framework
    if !client_app.is_plugin_added::<bevy::time::TimePlugin>() {
        client_app.add_plugins(bevy::time::TimePlugin);
    }
    if !client_app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        client_app.add_plugins(bevy::state::app::StatesPlugin);
    }

    // prepare channels
    prepare_network_channels(client_app, resend_time);

    // setup client with bevy_replicon
    client_app
        //add bevy_replicon client
        .add_plugins(RepliconPlugins
            .build()
            .disable::<ServerPlugin>()
            .disable::<ServerEventsPlugin>())
        .add_plugins(RepliconRenetClientPlugin)
        //prepare event handling
        .add_plugins(ClientEventHandlingPlugin)
        //register GameInitProgress for replication
        .replicate::<GameInitProgress>()

        //# PREUPDATE #
        //<-- girk renet client setup
        //<-- RenetReceive {renet}: collects network packets
        //<-- ClientSet::ReceivePackets {replicon}: collects renet packets
        //<-- ClientSet::ResetEvents (if client just connected) {replicon}: ensures client and server messages
        //    don't leak across a reconnect
        //<-- ClientSet::Receive {replicon}: processes replication messages
        //<-- ClientSet::SyncHierarchy {replicon}: synchronizes replicated entity hierarchies
        //<-- ReceiveServerEventsSet {girk}: collects GamePacket messages
        //<-- girk connection initialization management
        //<-- ClientFwSet::Start {girk}: handles client fw commands and network messages, prepares the
        //    client for this tick; we do this before ClientFwSet because server messages can control the current tick's
        //    game state (and in general determine the contents of the current tick - e.g. replicated state is applied
        //    before user logic)
        //# OnExit(ClientInstanceState::Game)
        //<-- cleanup client resources
        .configure_sets(PreUpdate,
            (
                // Ordering explanation:
                // - We want `ClientFwState::Syncing` to run for at least one tick before handling the first replication
                //   message. So we block the client set in the range [Setup, Syncing first tick]
                ClientSet::Receive
                    .run_if(not(in_state(ClientFwState::Setup)))
                    .run_if(not(in_state(ClientFwState::Connecting)))
                    .run_if(not(client_just_connected)),
                ClientSet::SyncHierarchy,
                ReceiveServerEventsSet,
                ClientFwSet::Start,
            )
                .chain()
        )
        .add_systems(PreUpdate,
            (
                cleanup_client_resources.run_if(client_just_disconnected),
                log_just_connected.run_if(client_just_connected),
                log_just_disconnected.run_if(client_just_disconnected),
                // reinitialize when disconnected and not at game end
                // - at game end the server will shut down, we don't want to reinitialize in that case
                // - note: there should not be a race condition here because the client fw only moves to End if
                //   the server sends an End state message, but this will only be called in a tick where we are disconnected
                //   and hence won't receive a game End state message in `ClientFwSet::Start` after this
                reinitialize_client
                    .run_if(client_disconnected)
                    .run_if(not(in_state(ClientFwState::Setup)))
                    .run_if(not(in_state(ClientFwState::End))),
                // set syncing when just connected
                // - note: this will not run in the first tick of `ClientFwState::Setup` because we disable
                //   `setup_renet_client` for that tick (it actually takes at least 3 ticks to connect once disconnected)
                set_client_connecting
                    .run_if(not(client_disconnected))
                    .run_if(not(just_entered_state(ClientFwState::Setup)))
                    .run_if(in_state(ClientFwState::Setup)),
                // set syncing when just connected
                set_client_syncing
                    .run_if(client_connected)
                    .run_if(not(just_entered_state(ClientFwState::Connecting)))
                    .run_if(in_state(ClientFwState::Connecting)),
                // set initialized when just synchronized
                // - note: this will not run in the first tick of `ClientFwState::Syncing` because we disabled
                //   `ClientSet::Receive` for that tick, so ServerUpdateTick will not change (unless the user manually changes
                //   it)
                set_client_init
                    .run_if(resource_changed::<ServerUpdateTick>)
                    .run_if(in_state(ClientFwState::Syncing)),
            )
                .chain()
                .after(ReceiveServerEventsSet)
                .before(ClientFwSet::Start)
                .run_if(in_state(ClientInstanceState::Game)),
        )
        .add_systems(OnExit(ClientInstanceState::Game), cleanup_client_resources)

        //# UPDATE #
        //<-- ClientFwSet::Update {girk}: user logic
        //<-- ClientFwLoadingSet (in state ClientInitState::InProgress) {girk}: should contain all user
        //    loading systems for setting up a game (systems with `.track_progress()`), but NOT app-setup systems
        //    which need to run on startup in ClientInstanceState::Client
        //<-- AssetsTrackProgress {iyes progress}: tracks progress of assets during initialization
        .add_systems(Update,
            (
                //track connection status
                track_connection_state.track_progress::<ClientInitState>(),
                //track whether the first replication message post-connect has been received
                //- note: we spawn an empty replicated entity in the game framework to ensure an init message is always sent
                //        when a client connects
                client_just_connected
                    .pipe(track_replication_initialized)
                    .track_progress::<ClientInitState>()
                    .run_if(resource_exists::<RenetClient>),
                //track whether the client is initialized
                //- note: this leverages the fact that `iyes_progress` collects progress in PostUpdate to ensure
                //        `ClientInitState::Done` will not be entered until `ClientFwState::Init` has run
                //        for at least one tick (because the client fw will not try to leave `ClientFwState::Init` until
                //        `ClientInitState::Done` has been entered, which can happen in the second Init tick
                //        at earliest)
                track_initialization_state
                    .track_progress::<ClientInitState>()
            )
                .in_set(ClientFwLoadingSet)
                .run_if(in_state(ClientInstanceState::Game))
        )
        .configure_sets(Update, ClientFwSet::Update.before(iyes_progress::prelude::AssetsTrackProgress))

        //# POSTUPDATE #
        //<-- CheckProgressSet {iyes_progress}: checks initialization progress
        //<-- ClientFwSet::End {girk}: dispatches messages to replicon, performs final tick cleanup
        //<-- ClientSet::Send (if connected) {replicon}: dispatches messages to renet (`send_client_packets`)
        //<-- ClientSet::SendPackets {replicon}: forwards packets to renet
        //<-- RenetSend {renet}: dispatches packets to the server
        .configure_sets(PostUpdate,
            ClientFwSet::End
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
/// Note that here we just wait for a new connect pack to appear, then set up a renet client.
/// For automatically requesting a new connect pack when disconnected, see the `bevy_girk_client_instance` crate.
///
/// Note: The `ClientConnectPack` needs to be inserted separately (e.g. by the `ClientFactory`).
pub fn prepare_client_app_network(client_app: &mut App)
{
    client_app.add_systems(PreUpdate,
            // Ordering explanation:
            // - We want `ClientFwState::Setup` to run for at least one tick.
            // - We want `client_disconnected` to return true for the first tick of `ClientFwState::Setup`.
            setup_renet_client.map(Result::unwrap)
                // add ordering constraint
                .before(bevy_renet2::prelude::RenetReceive)
                // ignore for first full tick after entering the game
                .run_if(not(just_entered_state(ClientInstanceState::Game)))
                // only try to set up the client while in game
                .run_if(in_state(ClientInstanceState::Game))
                // poll for connect packs while disconnected
                .run_if(client_disconnected)
                // check for connect pack
                .run_if(resource_exists::<ClientConnectPack>)
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a `bevy_girk` client app compatible with `bevy_girk` game apps.
/// - Sets up the client framework.
/// - Sets up replication.
/// - Sets up the renet client.
pub fn prepare_girk_client_app(client_app: &mut App, config: GirkClientStartupConfig)
{
    prepare_client_app_framework(client_app);
    prepare_client_app_replication(client_app, config.resend_time);
    prepare_client_app_network(client_app);
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a `bevy_girk` client app to run a game.
///
/// This should be called every time the same client app starts a new game.
pub fn setup_girk_client_game(world: &mut World, config: GirkClientConfig)
{
    world.insert_resource(config.client_fw_config);
    world.insert_resource(config.connect_pack);
}

//-------------------------------------------------------------------------------------------------------------------
