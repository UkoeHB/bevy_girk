//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;
use bevy_girk_wiring_common::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy_replicon::prelude::ClientId;

//standard shortcuts
#[cfg(not(target_family = "wasm"))]
use rand::thread_rng;
#[cfg(not(target_family = "wasm"))]
use rand::seq::SliceRandom;
use std::collections::VecDeque;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// Use this method in the crate that instantiates a launch pack source.
/*
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_protocol_id() -> u64
{
    let mut hasher = AHasher::default();
    PACKAGE_VERSION.hash(&mut hasher);
    hasher.finish()
}
*/

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_player_init_data(connection: ConnectionType, user_id: u128, client_id: ClientId) -> ClickClientInitDataForGame
{
    ClickClientInitDataForGame{
        connection,
        user_id,
        client_id,
        init: ClickClientInit::Player{
            player_name : String::from("player") + stringify!(?client_id),
        },
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_watcher_init_data(connection: ConnectionType, user_id: u128, client_id: ClientId) -> ClickClientInitDataForGame
{
    ClickClientInitDataForGame{
        connection,
        user_id,
        client_id,
        init: ClickClientInit::Watcher,
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_launch_pack(game_factory_config: &ClickGameFactoryConfig, start_request: &GameStartRequest) -> Result<GameLaunchPack, ()>
{
    // extract players/watchers from lobby data
    let Ok((mut players, mut watchers)) = ClickLobbyChecker::collect_members(&start_request.lobby_data)
    else { tracing::error!("unable to collect lobby members"); return Err(()); };

    let num_players  = players.len();
    let num_watchers = watchers.len();

    // shuffle the game participants
    #[cfg(not(target_family = "wasm"))]
    {
        players.shuffle(&mut thread_rng());
        watchers.shuffle(&mut thread_rng());
    }

    // make init data for the clients
    let mut client_init_data = Vec::<ClickClientInitDataForGame>::with_capacity(num_players + num_watchers);

    for (idx, (connection, player_user_id)) in players.iter().enumerate()
    {
        client_init_data.push(make_player_init_data(*connection, *player_user_id, ClientId::new(1 + idx as u64)));
    }

    for (idx, (connection, watcher_user_id)) in watchers.iter().enumerate()
    {
        let client_id = idx + num_players;
        client_init_data.push(make_watcher_init_data(*connection, *watcher_user_id, ClientId::new(1 + client_id as u64)));
    }

    // click launch pack
    let launch_pack = ClickLaunchPackData{ config: game_factory_config.clone(), clients: client_init_data };

    // launch pack
    let launch_pack = GameLaunchPack::new(start_request.game_id(), launch_pack);

    Ok(launch_pack)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub struct ClickGameLaunchPackSource
{
    /// Serialized config needed by game factory to start a game.
    config: ClickGameFactoryConfig,

    /// Queue of reports.
    queue: VecDeque<GameLaunchPackReport>,
}

impl ClickGameLaunchPackSource
{
    pub fn new(game_factory_config: &ClickGameFactoryConfig) -> ClickGameLaunchPackSource
    {
        ClickGameLaunchPackSource{ config: game_factory_config.clone(), queue: VecDeque::default() }
    }
}

impl GameLaunchPackSourceImpl for ClickGameLaunchPackSource
{
    /// Request a launch pack for a specified game.
    fn request_launch_pack(&mut self, start_request: &GameStartRequest)
    {
        match get_launch_pack(&self.config, start_request)
        {
            Ok(launch_pack) => self.queue.push_back(GameLaunchPackReport::Pack(launch_pack)),
            Err(_)          => self.queue.push_back(GameLaunchPackReport::Failure(start_request.game_id())),
        }
    }

    /// Get the next available report.
    fn try_next(&mut self) -> Option<GameLaunchPackReport>
    {
        self.queue.pop_front()
    }
}

//-------------------------------------------------------------------------------------------------------------------
