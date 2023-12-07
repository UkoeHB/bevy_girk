//local shortcuts
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_ongoing_games_basic()
{
    // make a cache
    let mut cache = OngoingGamesCache::new(OngoingGamesCacheConfig{ expiry_duration: Duration::from_secs(1) });

    // add game
    let game_id = 0u64;
    let game_hub_id = 0u128;
    let user_id_1 = 1u128;
    let user_id_2 = 2u128;
    let _ = cache.add_ongoing_game(
            OngoingGame{
                    game_id,
                    game_hub_id,
                    connect_infos : vec![GameConnectInfo::new_from_id(user_id_1), GameConnectInfo::new_from_id(user_id_2)],
                }
        ).expect("add should work");

    assert_eq!(cache.get_connect_infos(game_id).expect("should have game").len(), 2);
    let None = cache.get_connect_infos(game_id + 1) else { panic!("game should be unknown"); };
    let (query_game_id, _) = cache.get_user_connect_info(user_id_1).expect("user should have connect info");
    assert_eq!(query_game_id, game_id);
    let (query_game_id, _) = cache.get_user_connect_info(user_id_2).expect("user should have connect info");
    assert_eq!(query_game_id, game_id);
    let None = cache.get_user_connect_info(user_id_2 + 1) else { panic!("unknown user should not have connect info"); };

    // try to add the game again
    let Err(_) = cache.add_ongoing_game(
            OngoingGame{
                    game_id,
                    game_hub_id,
                    connect_infos : vec![GameConnectInfo::new_from_id(user_id_1), GameConnectInfo::new_from_id(user_id_2)],
                }
        ) else { panic!("adding the same game should fail"); };

    // try to add a new game using same users
    let game_id_2 = game_id + 1;
    let Err(_) = cache.add_ongoing_game(
            OngoingGame{
                    game_id: game_id_2,
                    game_hub_id,
                    connect_infos : vec![GameConnectInfo::new_from_id(user_id_1), GameConnectInfo::new_from_id(user_id_2)],
                }
        ) else { panic!("adding a game with users that are in-game should fail"); };

    // remove game
    let _ = cache.remove_ongoing_game(game_id).expect("removing ongoing game should succeed");
    let Err(_) = cache.remove_ongoing_game(game_id) else { panic!("removing duplicate ongoing game should fail"); };

    let None = cache.get_connect_infos(game_id) else { panic!("game should be unknown"); };
    let None = cache.get_user_connect_info(user_id_1) else { panic!("removed user should not have connect info"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_ongoing_games_expiration()
{
    // make a cache
    let one_third_duration = Duration::from_millis(10);
    let cache_config = OngoingGamesCacheConfig{
            expiry_duration: one_third_duration + one_third_duration + one_third_duration
        };
    let mut cache = OngoingGamesCache::new(cache_config);

    // add game
    let game_id_1 = 1u64;
    let user_id_1 = 1u128;
    let _ = cache.add_ongoing_game(
            OngoingGame{
                    game_id       : game_id_1,
                    game_hub_id   : 0u128,
                    connect_infos : vec![GameConnectInfo::new_from_id(user_id_1)],
                }
        ).expect("add should work");

    // wait for part of expiry period
    std::thread::sleep(one_third_duration + one_third_duration);

    // add another game
    let game_id_2 = 2u64;
    let user_id_2 = 2u128;
    let _ = cache.add_ongoing_game(
            OngoingGame{
                    game_id       : game_id_2,
                    game_hub_id   : 0u128,
                    connect_infos : vec![GameConnectInfo::new_from_id(user_id_2)],
                }
        ).expect("add should work");

    // remove expired (should do nothing)
    for _ in cache.drain_expired() {}

    assert_eq!(cache.get_connect_infos(game_id_1).unwrap().len(), 1);
    assert_eq!(cache.get_connect_infos(game_id_2).unwrap().len(), 1);

    // wait for expiration of first game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove expired (should remove first game)
    let mut count = 0;
    for _ in cache.drain_expired() { count += 1; }
    assert_eq!(count, 1);

    let None = cache.get_connect_infos(game_id_1) else { panic!("game 1 should be removed"); };
    assert_eq!(cache.get_connect_infos(game_id_2).unwrap().len(), 1);

    // wait a bit
    std::thread::sleep(one_third_duration);        

    // add new game with user 1
    let game_id_3 = 3u64;
    let _ = cache.add_ongoing_game(
            OngoingGame{
                    game_id       : game_id_3,
                    game_hub_id   : 0u128,
                    connect_infos : vec![GameConnectInfo::new_from_id(user_id_1)],
                }
        ).expect("add should work");

    // wait for expiration of second game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove expired (should remove second game)
    let mut count = 0;
    for _ in cache.drain_expired() { count += 1; }
    assert_eq!(count, 1);

    let None = cache.get_connect_infos(game_id_1) else { panic!("game 1 should be removed"); };
    let None = cache.get_connect_infos(game_id_2) else { panic!("game 1 should be removed"); };
    assert_eq!(cache.get_connect_infos(game_id_3).unwrap().len(), 1);
    let Some((query_game_id, _)) = cache.get_user_connect_info(user_id_1) else { panic!("user 1 should have game") };
    assert_eq!(query_game_id, game_id_3);
}

//-------------------------------------------------------------------------------------------------------------------
