//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_game_hub_server::*;

//third-party shortcuts

//standard shortcuts
use std::default::Default;
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_pending_games_basic()
{
    // make a cache
    let cache_config = PendingGamesCacheConfig{ expiry_duration: Duration::from_secs(1) };
    let mut cache = PendingGamesCache::new(cache_config);

    // empty cache test
    assert!(cache.extract_game(0u64).is_none());
    assert_eq!(cache.num_pending(), 0);

    // add one game
    let game_id = 0u64;
    let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id, ..Default::default() } };
    cache.add_pending_game(start_request).expect("adding pending game should succeed");
    assert!(cache.has_game(game_id));
    let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id, ..Default::default() } };
    if let Ok(_) = cache.add_pending_game(start_request) { panic!("inserting duplicate pending game should fail"); }

    assert_eq!(cache.num_pending(), 1);

    // remove game
    assert!(cache.extract_game(game_id).is_some());
    assert!(cache.extract_game(game_id).is_none());
    assert_eq!(cache.num_pending(), 0);

    // add more games
    let total = 10u64;
    for game_id in 1u64..=total
    {
        let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id, ..Default::default() } };
        cache.add_pending_game(start_request).expect("adding pending game should succeed");
        assert!(cache.has_game(game_id));
        let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id, ..Default::default() } };
        if let Ok(_) = cache.add_pending_game(start_request) { panic!("inserting duplicate pending game should fail"); }
    }
    assert_eq!(cache.num_pending(), total as usize);

    // drain the games
    let mut count = 0u64;
    for _ in cache.drain_all()
    {
        count += 1;
    }
    assert_eq!(count, total);
    assert_eq!(cache.num_pending(), 0);
    assert!(cache.extract_game(1u64).is_none());
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_pending_games_expiration()
{
    // make a cache
    let one_third_duration = Duration::from_millis(5);
    let cache_config = PendingGamesCacheConfig{
            expiry_duration: one_third_duration + one_third_duration + one_third_duration
        };
    let mut cache = PendingGamesCache::new(cache_config);

    // add game
    let game_id_1 = 1u64;
    let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id_1, ..Default::default() } };
    cache.add_pending_game(start_request).expect("adding pending game should succeed");

    // wait for part of expiry period
    std::thread::sleep(one_third_duration + one_third_duration);

    // add another game
    let game_id_2 = 2u64;
    let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id_2, ..Default::default() } };
    cache.add_pending_game(start_request).expect("adding pending game should succeed");

    // remove expired (should do nothing)
    for _ in cache.drain_expired() {}

    assert_eq!(cache.num_pending(), 2);

    // wait for expiration of first game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove expired (should remove first game)
    let mut count = 0;
    for _ in cache.drain_expired() { count += 1; }
    assert_eq!(count, 1);

    assert_eq!(cache.num_pending(), 1);
    assert!(!cache.has_game(game_id_1));
    assert!(cache.has_game(game_id_2));

    // wait a bit
    std::thread::sleep(one_third_duration);        

    // add new game with user 1
    let game_id_3 = 3u64;
    let start_request = GameStartRequest{ lobby_data: LobbyData { id: game_id_3, ..Default::default() } };
    cache.add_pending_game(start_request).expect("adding pending game should succeed");

    // wait for expiration of second game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove expired (should remove second game)
    let mut count = 0;
    for _ in cache.drain_expired() { count += 1; }
    assert_eq!(count, 1);

    assert_eq!(cache.num_pending(), 1);
    assert!(!cache.has_game(game_id_2));
    assert!(cache.has_game(game_id_3));
}

//-------------------------------------------------------------------------------------------------------------------
