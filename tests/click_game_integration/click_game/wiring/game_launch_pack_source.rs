//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts

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

fn make_player_init_data(env: bevy_simplenet::EnvType, user_id: u128, client_id: ClientIdType) -> ClientInitDataForGame
{
    let client_init_data = ClickClientInitDataForGame::Player{
            client_id   : client_id,
            player_name : String::from("player") + stringify!(client_id),
        };

    ClientInitDataForGame{
            env,
            user_id,
            data: ser_msg(&client_init_data),
        }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn make_watcher_init_data(env: bevy_simplenet::EnvType, user_id: u128, client_id: ClientIdType) -> ClientInitDataForGame
{
    let client_init_data = ClickClientInitDataForGame::Watcher{
            client_id: client_id,
        };

    ClientInitDataForGame{
            env,
            user_id,
            data: ser_msg(&client_init_data),
        }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_launch_pack(game_factory_config_ser: &Vec<u8>, start_request: &GameStartRequest) -> Result<GameLaunchPack, ()>
{
    // extract players/watchers from lobby data
    let Ok((mut players, mut watchers)) = ClickLobbyChecker::collect_members(&start_request.lobby_data)
    else { tracing::error!("unable to collect lobby members"); return Err(()); };

    let num_players  = players.len();
    let num_watchers = watchers.len();

    // shuffle the game participants
    //todo: assert there is only one player/watcher on WASM
    #[cfg(not(target_family = "wasm"))]
    {
        players.shuffle(&mut thread_rng());
        watchers.shuffle(&mut thread_rng());
    }

    // make init data for the clients
    let mut client_init_data = Vec::<ClientInitDataForGame>::new();
    client_init_data.reserve(num_players + num_watchers);

    for (idx, (env, player_user_id)) in players.iter().enumerate()
    {
        client_init_data.push(make_player_init_data(*env, *player_user_id, idx as ClientIdType));
    }

    for (idx, (env, watcher_user_id)) in watchers.iter().enumerate()
    {
        let client_id = idx + num_players;
        client_init_data.push(make_watcher_init_data(*env, *watcher_user_id, client_id as ClientIdType));
    }

    // launch pack
    let launch_pack = GameLaunchPack::new(start_request.game_id(), game_factory_config_ser.clone(), client_init_data);

    Ok(launch_pack)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct ClickGameLaunchPackSource
{
    /// Serialized config needed by game factory to start a game.
    game_factory_config_ser: Vec<u8>,

    /// Queue of reports.
    queue: VecDeque<GameLaunchPackReport>,
}

impl ClickGameLaunchPackSource
{
    pub fn new(game_factory_config: &ClickGameFactoryConfig) -> ClickGameLaunchPackSource
    {
        ClickGameLaunchPackSource{ game_factory_config_ser: ser_msg(&game_factory_config), queue: VecDeque::default() }
    }
}

impl GameLaunchPackSourceImpl for ClickGameLaunchPackSource
{
    /// Request a launch pack for a specified game.
    fn request_launch_pack(&mut self, start_request: &GameStartRequest)
    {
        match get_launch_pack(&self.game_factory_config_ser, start_request)
        {
            Ok(launch_pack) => self.queue.push_back(GameLaunchPackReport::Pack(launch_pack)),
            Err(_)          => self.queue.push_back(GameLaunchPackReport::Failure(start_request.game_id())),
        }
    }

    /// Get the next available report.
    fn try_get_next(&mut self) -> Option<GameLaunchPackReport>
    {
        self.queue.pop_front()
    }
}

//-------------------------------------------------------------------------------------------------------------------
