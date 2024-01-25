//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
#[allow(unused_imports)]
use bevy_renet::renet::transport::{generate_random_bytes, ServerAuthentication, ServerConfig};

//standard shortcuts
use std::net::SocketAddr;
use std::time::Duration;
use wasm_timer::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Dummy system, does nothing.
fn dummy() {}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_server_events(mut server_events: EventReader<bevy_renet::renet::ServerEvent>)
{
    for event in server_events.read()
    {
        tracing::debug!(?event);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_transport_errors(mut transport_errors: EventReader<renet::transport::NetcodeTransportError>)
{
    for error in transport_errors.read()
    {
        tracing::debug!(?error);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients_on_disconnect(
    mut server_events : EventReader<bevy_renet::renet::ServerEvent>,
    mut readiness     : ResMut<ClientReadiness>,
){
    for event in server_events.read()
    {
        let bevy_renet::renet::ServerEvent::ClientDisconnected{ client_id, .. } = event else { continue; };

        readiness.set(*client_id, Readiness::default());
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn new_server_config(num_clients: usize, server_setup_config: &GameServerSetupConfig, auth_key: &[u8; 32]) -> ServerConfig
{
    ServerConfig{
            current_time     : SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            max_clients      : num_clients,
            protocol_id      : server_setup_config.protocol_id,
            public_addresses : vec![SocketAddr::new(server_setup_config.server_ip.into(), 0)],
            authentication   : ServerAuthentication::Secure{ private_key: *auth_key },
        }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for setting up a `bevy_girk` server app.
#[derive(Debug)]
pub struct GirkServerConfig
{
    /// Config for the game framework.
    pub game_fw_config: GameFwConfig,
    /// Initializer for the game framework.
    pub game_fw_initializer: GameFwInitializer,
    /// Resend time for server messages within `renet`.
    pub resend_time: Duration,
    /// Config for setting up a game server.
    pub game_server_config: GameServerSetupConfig,
    /// The number of native clients that will connect.
    pub native_count: usize,
    /// The number of WASM clients that will connect.
    pub wasm_count: usize,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a game app with the `bevy_girk` game framework.
pub fn prepare_game_app_framework(
    game_app            : &mut App,
    game_fw_config      : GameFwConfig,
    game_fw_initializer : GameFwInitializer,
){
    // prepare server app
    game_app
        //setup components
        .add_plugins(GameFwPlugin)
        //game framework
        .insert_resource(game_fw_config)
        .insert_resource(game_fw_initializer);
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up `bevy_replicon` in a game app.
pub fn prepare_game_app_replication(game_app: &mut App, resend_time: Duration, update_timeout: Duration)
{
    // depends on game framework

    // prepare channels
    prepare_network_channels(game_app, resend_time);

    // setup server with bevy_replicon (includes bevy_renet)
    game_app
        // add bevy_replicon server
        .add_plugins(bevy::time::TimePlugin)  //required by bevy_renet
        .add_plugins(
            ReplicationPlugins
                .build()
                .disable::<ClientPlugin>()
                .set(ServerPlugin{
                    tick_policy: TickPolicy::EveryFrame,
                    update_timeout,
                })
        )
        //enable replication repair for client reconnects
        //todo: add custom input-status tracking mechanism w/ custom prespawn cleanup
        .add_plugins(bevy_replicon_repair::ServerPlugin)
        //prepare message channels
        //- note: the event types specified here do nothing
        .add_server_event_with::<GamePacket, _, _>(EventType::Unreliable, send_server_packets, dummy)
        .add_client_event_with::<ClientPacket, _, _>(EventType::Unreliable, dummy, receive_client_packets)

        //# PREUPDATE #
        //<-- RenetReceive {renet}: receive network packets from clients
        //<-- ServerSet::Receive {replicon}: process client acks and connection events
        //<-- ServerRepairSet {replicon repair}: repairs replicon server internal trackers for reconnected clients
        //<-- GameFwTickSetPrivate::FwStart {girk}: prepares the app for this tick
        .configure_sets(PreUpdate,
            GameFwTickSetPrivate::FwStart
                .after(bevy_replicon_repair::ServerRepairSet)
        )

        //# UPDATE #
        //<-- GameFwSet {girk}: contains user logic
        //  <-- GameFwTickSet::{Admin, Start} {girk}: ordinal sets for user logic
        //  <-- GameFwTickSetPrivate::FwHandleRequests {girk}: handle client requests; we do this in the middle of
        //      the ordinal sets so the game tick and game mode updaters (and user-defined tick initialization logic) can
        //      run first
        //  <-- GameFwTickSet::{PreLogic, Logic, PostLogic, End} {girk}: ordinal sets for user logic
        .add_systems(Update, reset_clients_on_disconnect.in_set(GameFwTickSet::Admin))

        //# POSTUPDATE
        //<-- GameFwTickSetPrivate::FwEnd {girk}: dispatch server messages to replicon
        //<-- ServerSet::Send {replicon}: dispatch replication messages and server messages to renet
        //<-- RenetSend {renet}: dispatch network packets to clients
        .configure_sets(PostUpdate,
            GameFwTickSetPrivate::FwEnd
                .before(bevy_replicon::prelude::ServerSet::Send)
        )
        //log server events and errors
        //- note that these will be logged out of order, since we need to collect both receive and send events and errors
        .add_systems(Last, (log_server_events, log_transport_errors).chain());
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a game app with renet servers.
///
/// If the game app is set up on a WASM target for local single-player
/// (`native_count` = 0, `wasm_count` = 1, target environment = "wasm"), then in-memory server and
/// client transports will be added to the app and the user must manually move the client transport into their client app.
///
/// Returns metadata for generating connect tokens for clients to connect to the the renet server.
pub fn prepare_game_app_network(
    game_app           : &mut App,
    game_server_config : GameServerSetupConfig,
    native_count       : usize,
    wasm_count         : usize,
) -> (Option<ConnectMetaNative>, Option<ConnectMetaWasm>)
{
    //todo: wasm single player, we don't need auth key, just use in-memory transport (need server config enum)
    //todo: set up renet server transports based on client types

    let mut native_meta = None;
    let wasm_meta = None;

    #[cfg(not(target_family = "wasm"))]
    {
        if native_count > 0
        {
            // set up renet server
            // - we use a unique auth key so clients can only interact with the server created here
            let auth_key = generate_random_bytes::<32>();
            let server_config = new_server_config(native_count, &game_server_config, &auth_key);
            let server_addr = setup_native_renet_server(game_app, server_config);

            native_meta = Some(ConnectMetaNative{
                server_config    : game_server_config,
                server_addresses : vec![server_addr],
                auth_key         : auth_key.clone(),
            });
        }

        if wasm_count > 0
        {
            tracing::error!("wasm clients not yet supported");
            //todo: add wasm transport
        }
    }

    #[cfg(target_family = "wasm")]
    {
        if native_count > 0 || wasm_count != 1
        { panic!("wasm game apps are only supported for single-player"); }

        tracing::error!("wasm single-player servers not yet supported");
        //todo: add in-memory server
    }

    (native_meta, wasm_meta)
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a `bevy_girk` game app.
/// - Sets up the game framework.
/// - Sets up replication.
/// - Sets up renet servers based on the number of clients. If the game app is set up on a WASM target for local
///   single-player (`native_count` = 0, `wasm_count` = 1, target environment = "wasm"), then in-memory server and
///   client transports will be added to the game app and the user must manually move the client transport into their
///   client app.
///
/// Returns metadata for generating connect tokens for clients to connect to the the renet server.
pub fn prepare_girk_game_app(
    game_app : &mut App,
    config   : GirkServerConfig
) -> (Option<ConnectMetaNative>, Option<ConnectMetaWasm>)
{
    prepare_game_app_framework(game_app, config.game_fw_config, config.game_fw_initializer);
    prepare_game_app_replication(
        game_app,
        config.resend_time,
        Duration::from_secs((config.game_server_config.timeout_secs * 2).min(1i32) as u64),
    );
    let (native_meta, wasm_meta) = prepare_game_app_network(
        game_app,
        config.game_server_config,
        config.native_count,
        config.wasm_count
    );

    (native_meta, wasm_meta)
}

//-------------------------------------------------------------------------------------------------------------------
