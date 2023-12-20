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
    fw_init      : GameFWInitializer,
    click_init   : ClickGameInitializer,
    clients      : Vec<(u128, ClientIdType)>,
    native_count : usize,
    wasm_count   : usize,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare information to use when setting up the game app.
fn prepare_game_startup(
    client_init_data     : &Vec<ClientInitDataForGame>,
    game_duration_config : GameDurationConfig
) -> Result<GameStartupHelper, ()>
{
    // prepare each client
    let mut client_states = Vec::<ClientState>::new();
    let mut players       = HashMap::<ClientIdType, PlayerState>::new();
    let mut watchers      = HashSet::<ClientIdType>::new();
    let mut clients       = Vec::<(u128, ClientIdType)>::new();
    let mut native_count  = 0;
    let mut wasm_count    = 0;
    client_states.reserve(client_init_data.len());
    players.reserve(client_init_data.len());
    watchers.reserve(client_init_data.len());
    clients.reserve(client_init_data.len());

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

        // count client type
        match client_init.env
        {
            bevy_simplenet::EnvType::Native => native_count += 1,
            bevy_simplenet::EnvType::Wasm => wasm_count += 1,
        }

        // save user_id/client_id mapping
        clients.push((client_init.user_id, client_id));
    }

    // finalize
    let game_context = ClickGameContext::new(gen_rand128(), game_duration_config);

    Ok(GameStartupHelper{
        fw_init    : GameFWInitializer{ clients: client_states },
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

/// Get game start infos for participants to use when setting up their game clients.
fn get_game_start_infos(
    app          : &App,
    user_clients : &Vec<(u128, ClientIdType)>,
) -> Result<Vec<GameStartInfo>, ()>
{
    // extract data
    let game_initializer = app.world.resource::<ClickGameInitializer>();
    let ticks_per_sec = app.world.resource::<GameFWConfig>().ticks_per_sec();

    // make start infos for each client
    let mut start_infos = Vec::<GameStartInfo>::new();
    start_infos.reserve(user_clients.len());

    for (user_id, client_id) in user_clients.iter()
    {
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

    Ok(start_infos)
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
        let startup = prepare_game_startup(&launch_pack.client_init_data, config.game_duration_config)?;

        // prepare game app
        let (native_meta, wasm_meta) = prepare_game_app_backend(
                app,
                config.game_fw_config,
                startup.fw_init,
                config.server_setup_config,
                startup.native_count,
                startup.wasm_count
            );
        prepare_game_app_core(app, startup.click_init);

        // game start info
        // - must call this AFTER prepping the game app and setting up the renet server
        let start_infos = get_game_start_infos(&app, &startup.clients)?;

        Ok(GameStartReport{ native_meta, wasm_meta, start_infos })
    }
}

//-------------------------------------------------------------------------------------------------------------------
