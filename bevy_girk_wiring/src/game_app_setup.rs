//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::{ReconnectPolicy, VisibilityAttributesPlugin};
use bevy_replicon_renet2::RepliconRenetServerPlugin;
#[allow(unused_imports)]
use bevy_renet2::renet2::transport::{ServerAuthentication, ServerSetupConfig};

//standard shortcuts
use std::net::SocketAddr;
use std::time::Duration;
use wasm_timer::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//todo: use bevy_replicon events once they implement Debug
fn log_server_events(mut server_events: EventReader<bevy_renet2::renet2::ServerEvent>)
{
    for event in server_events.read()
    {
        tracing::debug!(?event);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_transport_errors(mut transport_errors: EventReader<renet2::transport::NetcodeTransportError>)
{
    for error in transport_errors.read()
    {
        tracing::debug!(?error);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients_on_disconnect(
    mut server_events : EventReader<bevy_replicon::prelude::ServerEvent>,
    mut readiness     : ResMut<ClientReadiness>,
){
    for event in server_events.read()
    {
        let bevy_replicon::prelude::ServerEvent::ClientDisconnected{ client_id, .. } = event else { continue; };

        readiness.set(*client_id, Readiness::default());
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn new_server_config(
    num_clients: usize,
    config: &GameServerSetupConfig,
    auth_key: &[u8; 32]
) -> ServerSetupConfig
{
    ServerSetupConfig{
        current_time     : SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default(),
        max_clients      : num_clients,
        protocol_id      : config.protocol_id,
        socket_addresses : vec![vec![SocketAddr::new(config.server_ip.into(), 0)]],
        authentication   : ServerAuthentication::Secure{ private_key: *auth_key },
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for setting up a `bevy_girk` server app.
#[derive(Debug)]
pub struct GirkServerConfig
{
    /// Client list for the game.
    pub clients: GameFwClients,
    /// Config for the game framework.
    pub config: GameFwConfig,
    /// Resend time for server messages within `renet`.
    pub resend_time: Duration,
    /// Config for setting up a game server.
    pub game_server_config: GameServerSetupConfig,
    /// The number of in-memory clients that will connect.
    pub memory_count: usize,
    /// The number of native clients that will connect.
    pub native_count: usize,
    /// The number of WASM clients that will connect.
    pub wasm_count: usize,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a game app with the `bevy_girk` game framework.
pub fn prepare_game_app_framework(game_app: &mut App, clients: GameFwClients, config: GameFwConfig)
{
    if !game_app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        game_app.add_plugins(bevy::state::app::StatesPlugin);
    }

    // prepare server app
    game_app
        //setup components
        .add_plugins(GameFwPlugin)
        //game framework
        .insert_resource(clients)
        .insert_resource(config);
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up `bevy_replicon` in a game app.
pub fn prepare_game_app_replication(game_app: &mut App, resend_time: Duration, update_timeout: Duration)
{
    // depends on game framework
    if !game_app.is_plugin_added::<bevy::time::TimePlugin>() {
        game_app.add_plugins(bevy::time::TimePlugin);
    }
    if !game_app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        game_app.add_plugins(bevy::state::app::StatesPlugin);
    }

    // prepare channels
    prepare_network_channels(game_app, resend_time);

    // setup server with bevy_replicon (includes bevy_renet)
    game_app
        //add bevy_replicon server
        .add_plugins(
            RepliconPlugins
                .build()
                .disable::<ClientPlugin>()
                .disable::<ClientEventsPlugin>()
                .set(ServerPlugin{
                    tick_policy: TickPolicy::EveryFrame,
                    visibility_policy: VisibilityPolicy::Whitelist,
                    update_timeout,
                    replicate_after_connect: true,
                })
        )
        // add renet backend
        .add_plugins(RepliconRenetServerPlugin)
        //enable visibility attributes
        .add_plugins(VisibilityAttributesPlugin{ server_id: None, reconnect_policy: ReconnectPolicy::Reset })
        //prepare event handling
        .add_plugins(ServerEventHandlingPlugin)
        //register GameInitProgress for replication
        .replicate::<GameInitProgress>()

        //# PREUPDATE #
        //<-- RenetReceive {renet}: receive network packets from clients
        //<-- ServerSet::ReceivePackets {replicon}: collect renet packets
        //<-- ServerSet::Receive {replicon}: process client acks and connection events
        //<-- GameFwSetPrivate::FwStart {girk}: prepares the app for this tick
        .configure_sets(PreUpdate,
            GameFwSetPrivate::FwStart
                .after(bevy_replicon::prelude::ServerSet::Receive)
        )

        //# UPDATE #
        //<-- GameFwSet::{Admin, Start} {girk}: ordinal sets for user logic
        //<-- GameFwSetPrivate::FwHandleRequests {girk}: handle client requests; we do this in the middle of
        //      the ordinal sets so the game tick and game state updaters (and user-defined tick initialization logic) can
        //      run first
        //<-- GameFwSet::{PreLogic, Logic, PostLogic, End} {girk}: ordinal sets for user logic
        .add_systems(Update, reset_clients_on_disconnect.in_set(GameFwSet::Admin))

        //# POSTUPDATE
        //<-- GameFwSetPrivate::FwEnd {girk}: dispatch server messages to replicon
        //<-- ServerSet::StoreHierarchy {replicon}: store hierarchy information that needs to be replicated
        //<-- ServerSet::Send {replicon}: dispatch replication messages and server messages to renet
        //<-- RenetSend {renet}: dispatch network packets to clients
        .configure_sets(PostUpdate,
            GameFwSetPrivate::FwEnd
                .before(bevy_replicon::prelude::ServerSet::StoreHierarchy)
        )
        //log server events and errors
        //- note that these will be logged out of order, since we need to collect both receive and send events and errors
        .add_systems(Last, (log_server_events, log_transport_errors).chain());
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a game app with renet servers.
///
/// Returns metadata for generating connect tokens for clients to connect to the renet server.
pub fn prepare_game_app_network(
    game_app: &mut App,
    game_server_config: GameServerSetupConfig,
    memory_count: usize,
    native_count: usize,
    wasm_count: usize,
) -> (Option<ConnectMetaMemory>, Option<ConnectMetaNative>, Option<ConnectMetaWasm>)
{
    // set up renet server
    // - we use a unique auth key so clients can only interact with the server created here
    let auth_key = {
        // We assume this is only used for local-player on web.
        #[cfg(target_family = "wasm")]
        {
            if memory_count == 0 || native_count > 0 || wasm_count > 0
            {
                panic!("aborting game app networking construction; target family is WASM where only in-memory \
                    transports are permitted, but found other transport requests (memory: {}, native: {}, wasm: {})",
                    native_count, wasm_count, memory_count);
            }

            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
        }

        #[cfg(not(target_family = "wasm"))]
        bevy_renet2::renet2::transport::generate_random_bytes::<32>()
    };

    setup_combo_renet_server(
        game_app,
        game_server_config,
        memory_count,
        native_count,
        wasm_count,
        auth_key,
    )


/*
    #[cfg(not(target_family = "wasm"))]
    {
        if native_transport && !wasm_transport
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

        if wasm_transport
        {
            tracing::error!("wasm clients not yet supported");
            
            #[cfg(any())]
            {
                // set up renet server
                // - we use a unique auth key so clients can only interact with the server created here
                let auth_key = generate_random_bytes::<32>();
                let server_config = new_server_config(native_count, &game_server_config, &auth_key);
                let (native_addr, wasm_addr, cert_hashes) = setup_native_wasm_renet_server(game_app, server_config);

                native_meta = Some(ConnectMetaNative{
                    server_config    : game_server_config.clone(),
                    server_addresses : vec![native_addr],
                    socket_id        : 0,
                    auth_key         : auth_key.clone(),
                });
                wasm_meta = Some(ConnectMetaWasm{
                    server_config    : game_server_config,
                    server_addresses : vec![wasm_addr],
                    socket_id        : 1,
                    auth_key         : auth_key.clone(),
                    cert_hashes
                });
            }
        }
    }

    #[cfg(target_family = "wasm")]
    {
        panic!("wasm game apps not yet supported");

        tracing::error!("wasm single-player servers not yet supported");
        //todo: add in-memory server
    }
*/
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a `bevy_girk` game app.
/// - Sets up the game framework.
/// - Sets up replication.
/// - Sets up renet servers based on the requested transports.
///
/// Returns metadata for generating connect tokens for clients to connect to the the renet server.
pub fn prepare_girk_game_app(
    game_app : &mut App,
    config   : GirkServerConfig
) -> (Option<ConnectMetaMemory>, Option<ConnectMetaNative>, Option<ConnectMetaWasm>)
{
    prepare_game_app_framework(game_app, config.clients, config.config);
    prepare_game_app_replication(
        game_app,
        config.resend_time,
        Duration::from_secs((config.game_server_config.timeout_secs * 2).min(1i32) as u64),
    );
    prepare_game_app_network(
        game_app,
        config.game_server_config,
        config.memory_transport,
        config.native_transport,
        config.wasm_transport
    )
}

//-------------------------------------------------------------------------------------------------------------------
