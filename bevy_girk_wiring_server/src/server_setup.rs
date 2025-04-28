//local shortcuts
use crate::ServerEventHandlingPlugin;
use bevy_girk_game_fw::{
    ClientReadiness, GameFwClients, GameFwConfig, GameFwPlugin, GameFwSet,
    GameInitProgress, Readiness
};
use bevy_girk_wiring_common::prepare_network_channels;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::{prelude::{
    AppRuleExt, ClientEventPlugin, ClientPlugin, RepliconChannels, RepliconPlugins, ServerPlugin, TickPolicy, VisibilityPolicy
}, shared::backend::connected_client::NetworkId};
use bevy_replicon_attributes::{ReconnectPolicy, VisibilityAttributesPlugin};
use bevy_replicon_renet2::{RenetChannelsExt, RepliconRenetServerPlugin};
use renet2::ConnectionConfig;
use renet2_setup::{setup_combo_renet2_server_in_bevy, ClientCounts, ConnectMetas, GameServerSetupConfig};

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//todo: use bevy_replicon events once they implement Debug
fn log_server_events(mut server_events: EventReader<renet2::ServerEvent>)
{
    for event in server_events.read()
    {
        tracing::debug!(?event);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn log_transport_errors(mut transport_errors: EventReader<renet2_netcode::NetcodeTransportError>)
{
    for error in transport_errors.read()
    {
        tracing::debug!(?error);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_client_on_disconnect(
    event: Trigger<OnRemove, NetworkId>,
    ids: Query<&NetworkId>,
    mut readiness: ResMut<ClientReadiness>,
){
    let client_entity = event.target();
    let Ok(id) = ids.get(client_entity) else { return };
    readiness.set(id.get(), Readiness::default());
}

//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for setting up a `bevy_girk` server app.
///
/// See [`prepare_girk_game_app`].
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
    /// The number of clients that will connect.
    pub client_counts: ClientCounts,
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
pub fn prepare_game_app_replication(game_app: &mut App, mutations_timeout: Duration)
{
    // depends on game framework
    if !game_app.is_plugin_added::<bevy::time::TimePlugin>() {
        game_app.add_plugins(bevy::time::TimePlugin);
    }
    if !game_app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        game_app.add_plugins(bevy::state::app::StatesPlugin);
    }

    // setup server with bevy_replicon (includes bevy_renet)
    game_app
        //add bevy_replicon server
        .add_plugins(
            RepliconPlugins
                .build()
                .disable::<ClientPlugin>()
                .disable::<ClientEventPlugin>()
                .set(ServerPlugin{
                    tick_policy: TickPolicy::EveryFrame,
                    visibility_policy: VisibilityPolicy::Whitelist,
                    mutations_timeout,
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
        //<-- ServerSet::TriggerConnectionEvents: send connection events as observer events
        //   <-- reset_clients_on_disconnect
        //<-- ServerSet::Receive {replicon}: process client acks and connection events
        //<-- GameFwSet::Start {girk}: prepares the app for this tick
        .configure_sets(PreUpdate,
            GameFwSet::Start
                .after(bevy_replicon::prelude::ServerSet::Receive)
        )
        .add_observer(reset_client_on_disconnect)

        //# POSTUPDATE
        //<-- GameFwSet::End {girk}: dispatch server messages to replicon
        //<-- ServerSet::Send {replicon}: dispatch replication messages and server messages to renet
        //<-- RenetSend {renet}: dispatch network packets to clients
        .configure_sets(PostUpdate, GameFwSet::End)
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
    resend_time: Duration,
    game_server_config: GameServerSetupConfig,
    client_counts: ClientCounts,
) -> Result<ConnectMetas, String>
{
    let replicon_channels = game_app.world().resource::<RepliconChannels>();
    let mut server_channels = replicon_channels.server_configs();
    let mut client_channels = replicon_channels.client_configs();
    prepare_network_channels(game_app.world_mut(), &mut server_channels, &mut client_channels, resend_time);

    let connection_config = ConnectionConfig::from_channels(server_channels, client_channels);
    setup_combo_renet2_server_in_bevy(game_app.world_mut(), game_server_config, client_counts, connection_config)
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a `bevy_girk` game app.
/// - Sets up the game framework.
/// - Sets up replication.
/// - Sets up renet servers based on the requested transports.
///
/// Returns metadata for generating connect tokens for clients to connect to the the renet server.
pub fn prepare_girk_game_app(game_app: &mut App, config: GirkServerConfig) -> Result<ConnectMetas, String>
{
    prepare_game_app_framework(game_app, config.clients, config.config);
    prepare_game_app_replication(
        game_app,
        Duration::from_secs((config.game_server_config.timeout_secs * 2).min(1i32) as u64),
    );
    prepare_game_app_network(game_app, config.resend_time, config.game_server_config, config.client_counts)
}

//-------------------------------------------------------------------------------------------------------------------
