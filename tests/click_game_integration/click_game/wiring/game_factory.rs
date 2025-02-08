//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use bevy_girk_wiring_server::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::ClientId;
use renet2_setup::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct GameStartupHelper
{
    client_set     : GameFwClients,
    click_init     : ClickGameInitializer,
    start_infos    : Vec<GameStartInfo>,
    client_counts  : ClientCounts,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare information to use when setting up the game app.
fn prepare_game_startup(
    game_id         : u64,
    config          : &GameFwConfig,
    init_data       : Vec<ClickClientInitDataForGame>,
    duration_config : GameDurationConfig
) -> Result<GameStartupHelper, String>
{
    // prepare each client
    let mut client_set     = HashSet::<ClientId>::with_capacity(init_data.len());
    let mut players        = HashMap::<ClientId, PlayerState>::with_capacity(init_data.len());
    let mut watchers       = HashSet::<ClientId>::with_capacity(init_data.len());
    let mut start_infos    = Vec::with_capacity(init_data.len());
    let mut client_counts  = ClientCounts::default();

    for client_init in init_data
    {
        let client_id = client_init.client_id;

        // handle client type
        let initializer = match client_init.init
        {
            ClickClientInit::Player{ player_name } =>
            {
                players.insert(
                    client_id,
                    PlayerState{
                        id        : PlayerId { id: client_id },
                        name      : PlayerName{ name: player_name },
                        score     : Default::default(),
                        replicate : Default::default(),
                    }
                );
                ClickClientInitializer::Player(ClickPlayerInitializer{
                    player_context: ClickPlayerContext::new(
                        client_id,
                        duration_config,
                    )
                })
            },
            ClickClientInit::Watcher =>
            {
                watchers.insert(client_id);
                ClickClientInitializer::Watcher
            }
        };

        // save client id for the game
        client_set.insert(client_id);

        // count client type
        client_counts.add(client_init.connection, client_id.get());

        // Prep start info for the client.
        let client_fw_config = ClientFwConfig::new(config.ticks_per_sec(), game_id, client_id);
        let start_pack = ClickClientStartPack{ client_fw_config, initializer };
        let start_info = GameStartInfo::new(game_id, client_init.user_id, client_id.get(), start_pack);
        start_infos.push(start_info)
    }
    debug_assert_eq!(client_set.len(), start_infos.len());

    // finalize
    let game_context = ClickGameContext::new(gen_rand128(), duration_config);

    Ok(GameStartupHelper{
        client_set : GameFwClients::new(client_set),
        click_init : ClickGameInitializer{ game_context, players, watchers },
        start_infos,
        client_counts,
    })
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
        /// the client's player name
        player_name: String,
    },
    Watcher
}

//-------------------------------------------------------------------------------------------------------------------

/// Used by a game factory to initialize a client in the game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickClientInitDataForGame
{
    /// Indicates the type of connection the client wants to have with the server.
    pub connection: ConnectionType,
    /// The client's server-side user id.
    pub user_id: u128,
    /// The client's in-game id.
    pub client_id: ClientId,
    /// Client-type-specific init data for use in initializing a game.
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
    pub initializer: ClickClientInitializer,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ClickGameFactory;

impl GameFactoryImpl for ClickGameFactory
{
    type Launch = ClickLaunchPackData;

    /// Make a new game.
    fn new_game(
        &self,
        app: &mut App,
        game_id: u64,
        data: ClickLaunchPackData
    ) -> Result<GameStartReport, String>
    {
        // initialize clients and game config
        let config = data.config;
        let clients = data.clients;
        let startup = prepare_game_startup(game_id, &config.game_fw_config, clients, config.game_duration_config)?;

        // girk server config
        let server_config = GirkServerConfig{
            clients            : startup.client_set,
            config             : config.game_fw_config,
            game_server_config : config.server_setup_config,
            resend_time        : std::time::Duration::from_millis(100),
            client_counts      : startup.client_counts,
        };

        // prepare game app
        let metas = prepare_girk_game_app(app, server_config).unwrap();
        prepare_game_app_core(app, startup.click_init);

        Ok(GameStartReport{ metas, start_infos: startup.start_infos })
    }
}

//-------------------------------------------------------------------------------------------------------------------
