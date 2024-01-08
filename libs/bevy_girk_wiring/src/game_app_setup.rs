//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_replicon::prelude::*;
use bevy_replicon_repair::*;
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

/// Set up a game app with the `bevy_girk` game framework.
pub fn prepare_game_app_framework(
    game_app            : &mut App,
    game_fw_config      : GameFWConfig,
    game_fw_initializer : GameFWInitializer,
){
    // prepare message channels
    let (game_packet_sender, game_packet_receiver)     = new_channel::<GamePacket>();
    let (client_packet_sender, client_packet_receiver) = new_channel::<ClientPacket>();

    // prepare server app
    game_app
        //setup components
        .add_plugins(GameFWPlugin)
        //game framework
        .insert_resource(game_fw_config)
        .insert_resource(game_fw_initializer)
        .insert_resource(game_packet_sender)
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_packet_receiver);
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up `bevy_replicon` in a game app.
pub fn prepare_game_app_replication(game_app: &mut App, update_timeout: Duration)
{
    // depends on game framework

    // setup server with bevy_replicon (includes bevy_renet)
    game_app
        // add bevy_replicon server
        .add_plugins(bevy::time::TimePlugin)  //required by bevy_renet
        .add_plugins(
            ReplicationPlugins
                .build()
                .disable::<ClientPlugin>()
                .set( ServerPlugin{
                    tick_policy: TickPolicy::EveryFrame,
                    update_timeout,
                })
        )
        //enable replication repair for reconnects
        //todo: add custom input-status tracking mechanism w/ custom prespawn cleanup
        .add_plugins(RepliconRepairPluginServer)
        //prepare message channels
        .add_server_event_with::<EventConfig<GamePacket, SendUnreliable>, _, _>(EventType::Unreliable, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendUnordered>, _, _>(EventType::Unordered, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendOrdered>, _, _>(EventType::Ordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnreliable>, _, _>(EventType::Unreliable, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnordered>, _, _>(EventType::Unordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendOrdered>, _, _>(EventType::Ordered, dummy, dummy)
        //message receiving
        .add_systems(PreUpdate,
            receive_client_messages
                .run_if(resource_exists::<RenetServer>())
                .after(bevy_replicon::prelude::ServerSet::Receive)
                .before(GameFWTickSetPrivate::FWStart)
        )
        // <- client logic is in Update
        //message sending
        .add_systems(PostUpdate,
            send_server_messages
                .run_if(resource_exists::<RenetServer>())
                .after(GameFWTickSetPrivate::FWEnd)
                .before(bevy_replicon::prelude::ServerSet::Send)
        )
        //log server events and errors
        //- note that these will be logged out of order, since we need to collect both receive and send events and errors
        .add_systems(Last, (log_server_events, log_transport_errors).chain());
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up a game app with renet servers.
///
/// If the backend is set up on a WASM target for local
/// single-player (`native_count` = 0, `wasm_count` = 1, target environment = "wasm"), then in-memory server and
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
    #[cfg(target_family = "wasm")]
    { panic!("todo: gen random bytes not supported on WASM"); }

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
        { panic!("wasm game app backends are only supported for single-player"); }

        tracing::error!("wasm single-player servers not yet supported");
        //todo: add in-memory server
    }

    (native_meta, wasm_meta)
}

//-------------------------------------------------------------------------------------------------------------------

/// Set up a game app to hook into the `bevy_girk` backend.
/// - Sets up the game framework.
/// - Sets up replication.
/// - Sets up renet servers based on the number of clients. If the backend is set up on a WASM target for local
///   single-player (`native_count` = 0, `wasm_count` = 1, target environment = "wasm"), then in-memory server and
///   client transports will be added to the app and the user must manually move the client transport into their client app.
///
/// Returns metadata for generating connect tokens for clients to connect to the the renet server.
//todo: 'backend' is wrong term here?
pub fn prepare_game_app_backend(
    game_app            : &mut App,
    game_fw_config      : GameFWConfig,
    game_fw_initializer : GameFWInitializer,
    game_server_config  : GameServerSetupConfig,
    native_count        : usize,
    wasm_count          : usize,
) -> (Option<ConnectMetaNative>, Option<ConnectMetaWasm>)
{
    prepare_game_app_framework(game_app, game_fw_config, game_fw_initializer);
    prepare_game_app_replication(game_app, Duration::from_secs((game_server_config.timeout_secs * 2).min(1i32) as u64));
    let (native_meta, wasm_meta) = prepare_game_app_network(game_app, game_server_config, native_count, wasm_count);

    (native_meta, wasm_meta)
}

//-------------------------------------------------------------------------------------------------------------------
