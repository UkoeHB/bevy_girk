//local shortcuts
use crate::click_game_integration::*;

//third-party shortcuts
use bevy::utils::AHasher;
use renet2::ClientId;
use renet2_setup::ConnectionType;

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
        player_name : String::from("player") + stringify!(?client_id),
    };

    ClickClientInitDataForGame{
        client_id,
        connection: ConnectionType::inferred(),
        user_id,
        init,
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_watcher_init_for_game(user_id: u128, client_id: ClientId) -> ClickClientInitDataForGame
{
    let init = ClickClientInit::Watcher;

    ClickClientInitDataForGame{
        client_id,
        connection: ConnectionType::inferred(),
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
            id as u64,
            PlayerState{
                    id: PlayerId{ id: id as u64 },
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
