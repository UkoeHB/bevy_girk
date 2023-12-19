//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use bevy_girk_wiring::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_renet::renet::transport::{generate_random_bytes, ServerAuthentication, ServerConfig};
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare information to use when setting up the game app.
fn prepare_game_startup(
    client_init_data     : &Vec<ClientInitDataForGame>,
    game_duration_config : GameDurationConfig
) -> Result<(GameFWInitializer, ClickGameInitializer, Vec<(bevy_simplenet::EnvType, u128, ClientIdType)>), ()>
{
    // prepare each client
    let mut client_states = Vec::<ClientState>::new();
    let mut players       = HashMap::<ClientIdType, PlayerState>::new();
    let mut watchers      = HashSet::<ClientIdType>::new();
    let mut user_clients  = Vec::<(bevy_simplenet::EnvType, u128, ClientIdType)>::new();
    client_states.reserve(client_init_data.len());
    players.reserve(client_init_data.len());
    watchers.reserve(client_init_data.len());
    user_clients.reserve(client_init_data.len());

    for client_init in client_init_data.iter()
    {
        // deserialize the client init
        let Some(extracted_client_init) = deser_msg::<ClickClientInitDataForGame>(&client_init.data)
        else { tracing::error!("unable to deserialize a client's init data"); return Err(()); };

        // handle client type
        let client_id = match extracted_client_init
        {
            ClickClientInitDataForGame::Player{ client_id, player_name } =>
            {
                players.insert(client_id, 
                        PlayerState{
                                id   : PlayerId { id: client_id },
                                name : PlayerName{ name: player_name },
                                ..default()
                            });
                client_id
            },
            ClickClientInitDataForGame::Watcher{ client_id } =>
            {
                watchers.insert(client_id);
                client_id
            }
        };

        // make client state
        client_states.push(
                ClientState{
                        id            : ClientId::new(client_id),
                        access_rights :
                            InfoAccessRights{
                                    client : Some(client_id),
                                    global : true
                                }
                    }
            );

        // save user_id/client_id mapping
        user_clients.push((client_init.env, client_init.user_id, client_id));
    }

    // finalize
    let game_fw_initializer = GameFWInitializer{ clients: client_states };
    let game_context        = ClickGameContext::new(gen_rand128(), game_duration_config);
    let game_initializer    = ClickGameInitializer{ game_context, players, watchers };

    Ok((game_fw_initializer, game_initializer, user_clients))
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_get_player_initializer(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientIdType,
) -> Option<ClickPlayerInitializer>
{
    if !game_initializer.players.contains_key(&client_id) { return None; };
    Some(ClickPlayerInitializer{
            player_context:
                ClickPlayerContext::new(
                        client_id,
                        *game_initializer.game_context.duration_config(),
                    )
        })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_get_watcher_initializer(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientIdType,
) -> Option<()>
{
    if !game_initializer.watchers.contains(&client_id) { return None; };
    Some(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn click_client_initializer(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientIdType,
) -> Result<ClickClientInitializer, ()>
{
    // try to make player config
    if let Some(player_init) = try_get_player_initializer(game_initializer, client_id)
    { return Ok(ClickClientInitializer::Player(player_init)); }

    // try to make watcher config
    if let Some(()) = try_get_watcher_initializer(game_initializer, client_id)
    { return Ok(ClickClientInitializer::Watcher); }

    tracing::error!(client_id, "client is not a participant in the game");
    Err(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn prepare_client_start_pack(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientIdType,
    ticks_per_sec    : Ticks,
) -> Result<ClickClientStartPack, ()>
{
    // set up client framework
    let client_fw_config = ClientFWConfig::new(ticks_per_sec, client_id);

    // set up client config
    let click_client_initializer = click_client_initializer(game_initializer, client_id)?;

    Ok(ClickClientStartPack{ client_fw_config, click_client_initializer })
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

/// Prepare game start report for participants to use when setting up their game clients.
fn prepare_game_start_report(
    app              : &App,
    server_config    : &GameServerSetupConfig,
    auth_key         : &[u8; 32],
    user_clients     : &Vec<(bevy_simplenet::EnvType, u128, ClientIdType)>,
    server_addresses : Vec<SocketAddr>,
) -> Result<GameStartReport, ()>
{
    // extract data
    let game_initializer = app.world.resource::<ClickGameInitializer>();
    let ticks_per_sec = app.world.resource::<GameFWConfig>().ticks_per_sec();

    // make start infos for each client
    let mut native_count = 0;
    let mut wasm_count = 0;
    let mut start_infos = Vec::<GameStartInfo>::new();
    start_infos.reserve(user_clients.len());

    for (env, user_id, client_id) in user_clients.iter()
    {
        // count client types
        match env
        {
            bevy_simplenet::EnvType::Native => native_count += 1,
            bevy_simplenet::EnvType::Wasm => wasm_count += 1,
        }

        // get game start package for client
        let client_start_pack = prepare_client_start_pack(&*game_initializer, *client_id, ticks_per_sec)?;

        start_infos.push(
                GameStartInfo{
                        user_id: *user_id,
                        client_id: *client_id as u64,
                        serialized_start_data: ser_msg(&client_start_pack),
                    }
            );
    }

    // prepare connect metas based on client types
    let mut native_meta = None;
    let wasm_meta = None;

    if native_count > 0
    {
        native_meta = Some(GameServerConnectMetaNative{
            server_config    : server_config.clone(),
            server_addresses : server_addresses.clone(),
            auth_key         : auth_key.clone(),
        });
    }

    if wasm_count > 0
    {
        tracing::error!("wasm clients not yet supported");
    }

    Ok(GameStartReport{ native_meta, wasm_meta, start_infos })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Configuration for setting up a game with a game factory.
#[derive(Serialize, Deserialize)]
pub struct ClickGameFactoryConfig
{
    pub server_setup_config  : GameServerSetupConfig,
    pub game_fw_config       : GameFWConfig,
    pub game_duration_config : GameDurationConfig,
}

//-------------------------------------------------------------------------------------------------------------------

/// Client init data used when setting up a game.
#[derive(Serialize, Deserialize)]
pub enum ClickClientInitDataForGame
{
    Player{
        /// the client's in-game id
        client_id: ClientIdType,
        /// the client's player name
        player_name: String,
    },
    Watcher{
        /// the client's in-game id
        client_id: ClientIdType,
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Config used to initialize a client.
#[derive(Serialize, Deserialize)]
pub enum ClickClientInitializer
{
    Player(ClickPlayerInitializer),
    Watcher
}

//-------------------------------------------------------------------------------------------------------------------

/// Start-up pack for clients.
#[derive(Serialize, Deserialize)]
pub struct ClickClientStartPack
{
    /// Client framework config.
    pub client_fw_config: ClientFWConfig,
    /// Client initializer.
    pub click_client_initializer: ClickClientInitializer,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ClickGameFactory;

impl GameFactoryImpl for ClickGameFactory
{
    /// Make a new game.
    fn new_game(&self, app: &mut App, launch_pack: GameLaunchPack) -> Result<GameStartReport, ()>
    {
        // extract game factory config
        let Some(config) = deser_msg::<ClickGameFactoryConfig>(&launch_pack.game_init_data)
        else { tracing::error!("could not deserialize click game factory config"); return Err(()); };

        // initialize clients and game config
        let (game_fw_initializer, game_ctx, user_clients) = prepare_game_startup(
                &launch_pack.client_init_data,
                config.game_duration_config
            )?;

        // prepare game app
        prepare_game_app_framework(app, config.game_fw_config, game_fw_initializer);
        prepare_game_app_replication(app);
        prepare_game_app_core(app, game_ctx);

        // set up renet server
        // - we use a unique auth key so clients can only interact with the server created here
        //todo: wasm single player, we don't need auth key, just use in-memory transport (need server config enum)
        //todo: set up renet server transports based on client types
        #[cfg(target_family = "wasm")]
        { panic!("todo: gen random bytes not supported on WASM"); }

        let auth_key = generate_random_bytes::<32>();
        let server_config = new_server_config(launch_pack.client_init_data.len(), &config.server_setup_config, &auth_key);
        let server_addr = setup_native_renet_server(app, server_config);

        // prepare game start report
        // - must call this AFTER prepping the game app and setting up the renet server
        let game_start_report = prepare_game_start_report(
                &app,
                &config.server_setup_config,
                &auth_key,
                &user_clients,
                vec![server_addr]
            )?;

        Ok(game_start_report)
    }
}

//-------------------------------------------------------------------------------------------------------------------
