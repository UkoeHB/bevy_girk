//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::*;

//third-party shortcuts
use bevy::utils::AHasher;

//standard shortcuts
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

//-------------------------------------------------------------------------------------------------------------------

pub fn get_test_protocol_id() -> u64
{
    let mut hasher = AHasher::default();
    "test_protocol_id".hash(&mut hasher);
    hasher.finish()
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_player_init_for_game(user_id: u128, client_id: ClientId) -> ClickClientInitDataForGame
{
    let init = ClickClientInit::Player{
            client_id,
            player_name : String::from("player") + stringify!(?client_id),
        };

        ClickClientInitDataForGame{
            env: bevy_simplenet::env_type(),
            user_id,
            init,
        }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_watcher_init_for_game(user_id: u128, client_id: ClientId) -> ClickClientInitDataForGame
{
    let init = ClickClientInit::Watcher{ client_id };

        ClickClientInitDataForGame{
            env: bevy_simplenet::env_type(),
            user_id,
            init,
        }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_game_initializer(
    num_players     : usize,
    duration_config : GameDurationConfig,
) -> ClickGameInitializer
{
    // make player states
    let mut players: HashMap<ClientId, PlayerState> = HashMap::default();

    for id in 0..num_players
    {
        players.insert(
                ClientId::from_raw(id as u64),
                PlayerState{
                        id: PlayerId{ id: ClientId::from_raw(id as u64) },
                        name: PlayerName{ name: String::from("testname") },
                        score: Default::default(),
                        replicate: Default::default(),
                    }
            );
    }

    // make game context
    let game_context = ClickGameContext::new(0u128, duration_config);

    ClickGameInitializer { game_context, players, watchers: HashSet::default() }
}

//-------------------------------------------------------------------------------------------------------------------
