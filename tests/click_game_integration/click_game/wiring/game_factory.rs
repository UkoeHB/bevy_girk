//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use bevy_girk_wiring::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct GameStartupHelper
{
    client_set   : GameFwClients,
    click_init   : ClickGameInitializer,
    clients      : Vec<(u128, ClientId)>,
    native_count : usize,
    wasm_count   : usize,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare information to use when setting up the game app.
fn prepare_game_startup(
    client_init_data     : Vec<ClickClientInitDataForGame>,
    game_duration_config : GameDurationConfig
) -> Result<GameStartupHelper, ()>
{
    // prepare each client
    let mut client_set    = HashSet::<ClientId>::new();
    let mut players       = HashMap::<ClientId, PlayerState>::new();
    let mut watchers      = HashSet::<ClientId>::new();
    let mut clients       = Vec::<(u128, ClientId)>::new();
    let mut native_count  = 0;
    let mut wasm_count    = 0;
    client_set.reserve(client_init_data.len());
    players.reserve(client_init_data.len());
    watchers.reserve(client_init_data.len());
    clients.reserve(client_init_data.len());

    for client_init in client_init_data
    {
        // handle client type
        let client_id = match client_init.init
        {
            ClickClientInit::Player{ client_id, player_name } =>
            {
                players.insert(client_id, 
                        PlayerState{
                                id        : PlayerId { id: client_id },
                                name      : PlayerName{ name: player_name },
                                score     : Default::default(),
                                replicate : Default::default(),
                            });
                client_id
            },
            ClickClientInit::Watcher{ client_id } =>
            {
                watchers.insert(client_id);
                client_id
            }
        };

        // save client id for the game
        client_set.insert(client_id);

        // count client type
        match client_init.env
        {
            bevy_simplenet::EnvType::Native => native_count += 1,
            bevy_simplenet::EnvType::Wasm => wasm_count += 1,
        }

        // save user_id/client_id mapping
        clients.push((client_init.user_id, client_id));
    }
    debug_assert_eq(client_set.len(), clients.len());

    // finalize
    let game_context = ClickGameContext::new(gen_rand128(), game_duration_config);

    Ok(GameStartupHelper{
        client_set,
        click_init : ClickGameInitializer{ game_context, players, watchers },
        clients,
        native_count,
        wasm_count,
    })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_get_player_initializer(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientId,
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
    client_id        : ClientId,
) -> Option<()>
{
    if !game_initializer.watchers.contains(&client_id) { return None; };
    Some(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn click_client_initializer(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientId,
) -> Result<ClickClientInitializer, ()>
{
    // try to make player config
    if let Some(player_init) = try_get_player_initializer(game_initializer, client_id)
    { return Ok(ClickClientInitializer::Player(player_init)); }

    // try to make watcher config
    if let Some(()) = try_get_watcher_initializer(game_initializer, client_id)
    { return Ok(ClickClientInitializer::Watcher); }

    tracing::error!(?client_id, "client is not a participant in the game");
    Err(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn prepare_client_start_pack(
    game_initializer : &ClickGameInitializer,
    client_id        : ClientId,
    ticks_per_sec    : u32,
) -> Result<ClickClientStartPack, ()>
{
    // set up client framework
    let client_fw_config = ClientFwConfig::new(ticks_per_sec, client_id);

    // set up client config
    let click_client_initializer = click_client_initializer(game_initializer, client_id)?;

    Ok(ClickClientStartPack{ client_fw_config, click_client_initializer })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Get game start infos for participants to use when setting up their game clients.
fn get_game_start_infos(
    app          : &App,
    game_id      : u64,
    user_clients : &Vec<(u128, ClientId)>,
) -> Result<Vec<GameStartInfo>, ()>
{
    // extract data
    let game_initializer = app.world.resource::<ClickGameInitializer>();
    let ticks_per_sec = app.world.resource::<GameFwConfig>().ticks_per_sec();

    // make start infos for each client
    let mut start_infos = Vec::<GameStartInfo>::new();
    start_infos.reserve(user_clients.len());

    for (user_id, client_id) in user_clients.iter()
    {
        // get game start package for client
        let client_start_pack = prepare_client_start_pack(&*game_initializer, *client_id, ticks_per_sec)?;

        start_infos.push(
                GameStartInfo{
                    game_id,
                    user_id: *user_id,
                    client_id: client_id.raw(),
                    serialized_start_data: ser_msg(&client_start_pack),
                }
            );
    }

    Ok(start_infos)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Configuration for setting up a game with a game factory.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickGameFactoryConfig
{
    pub server_setup_config  : GameServerSetupConfig,
    pub game_fw_config       : GameFwConfig,
    pub game_duration_config : GameDurationConfig,
}

//-------------------------------------------------------------------------------------------------------------------

/// Client init data used when setting up a game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClickClientInit
{
    Player{
        /// the client's in-game id
        client_id: ClientId,
        /// the client's player name
        player_name: String,
    },
    Watcher{
        /// the client's in-game id
        client_id: ClientId,
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Used by a game factory to initialize a client in the game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickClientInitDataForGame
{
    /// The client's environment type.
    pub env: bevy_simplenet::EnvType,

    /// The client's server-side user id.
    pub user_id: u128,

    /// Client init data for use in initializing a game.
    pub init: ClickClientInit,
}

//-------------------------------------------------------------------------------------------------------------------

/// Used by a game factory to initialize a game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickLaunchPackData
{
    /// Game config.
    pub config: ClickGameFactoryConfig,

    /// Client init data for use in initializing a game.
    pub clients: Vec<ClickClientInitDataForGame>,
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
    pub client_fw_config: ClientFwConfig,
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
        let Some(data) = deser_msg::<ClickLaunchPackData>(&launch_pack.game_launch_data)
        else { tracing::error!("could not deserialize click game factory config"); return Err(()); };

        // initialize clients and game config
        let config = data.config;
        let clients = data.clients;
        let startup = prepare_game_startup(clients, config.game_duration_config)?;

        // girk server config
        let server_config = GirkServerConfig{
            clients            : startup.client_set,
            config             : config.game_fw_config,
            game_server_config : config.server_setup_config,
            resend_time        : std::time::Duration::from_millis(100),
            native_count       : startup.native_count,
            wasm_count         : startup.wasm_count,
        };

        // prepare game app
        let (native_meta, wasm_meta) = prepare_girk_game_app(app, server_config);
        prepare_game_app_core(app, startup.click_init);

        // game start info
        // - must call this AFTER prepping the game app and setting up the renet server
        let start_infos = get_game_start_infos(&app, launch_pack.game_id, &startup.clients)?;

        Ok(GameStartReport{ native_meta, wasm_meta, start_infos })
    }
}

//-------------------------------------------------------------------------------------------------------------------
