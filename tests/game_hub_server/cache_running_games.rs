//local shortcuts
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_hub_server::*;
use bevy_girk_game_instance::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_running_games_basic()
{
    // make a cache
    let cache_config = RunningGamesCacheConfig{ expiry_duration: Duration::from_secs(1) };
    let factory = GameFactory::new(DummyGameFactory{});
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(factory));
    let mut cache = RunningGamesCache::new(cache_config, game_launcher);

    // prep game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : 100000,
            game_duration_ticks : 1,
        };


    // empty cache test
    assert!(cache.extract_instance(0u64).is_none());
    assert_eq!(cache.num_running(), 0);

    // add one game
    let game_id = 0u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack = GameLaunchPack::new(game_id, dummy_pack.clone());
    let start_request = GameStartRequest{ lobby_data: LobbyData{ id: game_id, ..Default::default() } };
    cache.make_instance(start_request.clone(), launch_pack).expect("making game instance should succeed");
    assert!(cache.has_game(game_id));
    let launch_pack = GameLaunchPack::new(game_id, dummy_pack);
    if let Ok(_) = cache.make_instance(start_request, launch_pack) { panic!("making duplicate instance should fail"); }

    assert_eq!(cache.num_running(), 1);

    // remove game
    assert!(cache.extract_instance(game_id).is_some());
    assert!(cache.extract_instance(game_id).is_none());
    assert_eq!(cache.num_running(), 0);

    // add more games
    let total = 10u64;
    for game_id in 1u64..=total
    {
        let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
        let launch_pack = GameLaunchPack::new(game_id, dummy_pack.clone());
        let start_request = GameStartRequest{ lobby_data: LobbyData{ id: game_id, ..Default::default() } };
        cache.make_instance(start_request.clone(), launch_pack).expect("making game instance should succeed");
        assert!(cache.has_game(game_id));
        let launch_pack = GameLaunchPack::new(game_id, dummy_pack);
        if let Ok(_) = cache.make_instance(start_request, launch_pack) { panic!("making duplicate instance should fail"); }
    }
    assert_eq!(cache.num_running(), total as usize);

    // drain the games
    let mut count = 0u64;
    for _ in cache.drain_all()
    {
        count += 1;
    }
    assert_eq!(count, total);
    assert_eq!(cache.num_running(), 0);
    assert!(cache.extract_instance(1u64).is_none());
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_running_games_expiration()
{
    // make a cache
    let one_third_duration = Duration::from_millis(15);
    let cache_config = RunningGamesCacheConfig{ expiry_duration: one_third_duration + one_third_duration + one_third_duration };
    let factory = GameFactory::new(DummyGameFactory{});
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(factory));
    let mut cache = RunningGamesCache::new(cache_config, game_launcher);

    // prep game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : 100,
            game_duration_ticks : 100,  //long game time
        };


    // add game
    let game_id_1 = 0u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack_1 = GameLaunchPack::new(game_id_1, dummy_pack.clone());
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    cache.make_instance(start_request_1, launch_pack_1).expect("making game instance should succeed");

    // wait for part of expiry period
    std::thread::sleep(one_third_duration + one_third_duration);

    // add another game
    let game_id_2 = 2u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack_2 = GameLaunchPack::new(game_id_2, dummy_pack);
    let start_request_2 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_2, ..Default::default() } };
    cache.make_instance(start_request_2, launch_pack_2).expect("making game instance should succeed");

    // remove expired (should do nothing)
    for mut instance in cache.drain_invalid()
    {
        let None = instance.try_get() else { panic!("instance should be running"); };
        instance.send_command(GameInstanceCommand::Abort).unwrap();
    }

    assert_eq!(cache.num_running(), 2);

    // wait for expiration of first game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove expired (should remove first game)
    let mut count = 0;
    for mut instance in cache.drain_invalid()
    {
        count += 1;
        let None = instance.try_get() else { panic!("instance should be running"); };
        instance.send_command(GameInstanceCommand::Abort).unwrap();
    }
    assert_eq!(count, 1);

    assert_eq!(cache.num_running(), 1);
    assert!(!cache.has_game(game_id_1));
    assert!(cache.has_game(game_id_2));

    // wait a bit
    std::thread::sleep(one_third_duration);        

    // add new game with user 1
    let game_id_3 = 3u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack_3 = GameLaunchPack::new(game_id_3, dummy_pack);
    let start_request_3 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_3, ..Default::default() } };
    cache.make_instance(start_request_3, launch_pack_3).expect("making game instance should succeed");

    // wait for expiration of second game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove expired (should remove second game)
    let mut count = 0;
    for mut instance in cache.drain_invalid()
    {
        count += 1;
        let None = instance.try_get() else { panic!("instance should be running"); };
        instance.send_command(GameInstanceCommand::Abort).unwrap();
    }
    assert_eq!(count, 1);

    assert_eq!(cache.num_running(), 1);
    assert!(!cache.has_game(game_id_2));
    assert!(cache.has_game(game_id_3));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_running_games_termination()
{
    // make a cache
    let one_third_duration = Duration::from_millis(20);
    let cache_config = RunningGamesCacheConfig{ expiry_duration: Duration::from_secs(1) };  //long expiry duration
    let factory = GameFactory::new(DummyGameFactory{});
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(factory));
    let mut cache = RunningGamesCache::new(cache_config, game_launcher);

    // prep game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : 100,
            game_duration_ticks : 6,  //60ms game time
        };


    // add game
    let game_id_1 = 0u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack_1 = GameLaunchPack::new(game_id_1, dummy_pack.clone());
    let start_request_1 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_1, ..Default::default() } };
    cache.make_instance(start_request_1, launch_pack_1).expect("making game instance should succeed");

    // wait for part of game 1 duration
    std::thread::sleep(one_third_duration + one_third_duration);

    // add another game
    let game_id_2 = 2u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack_2 = GameLaunchPack::new(game_id_2, dummy_pack);
    let start_request_2 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_2, ..Default::default() } };
    cache.make_instance(start_request_2, launch_pack_2).expect("making game instance should succeed");

    // remove terminated (should do nothing)
    for mut instance in cache.drain_invalid()
    {
        let Some(true) = instance.try_get() else { panic!("instance should be terminated successfully"); };
        instance.send_command(GameInstanceCommand::Abort).unwrap();
    }

    assert_eq!(cache.num_running(), 2);

    // wait for termination of first game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove terminated (should remove first game)
    let mut count = 0;
    for mut instance in cache.drain_invalid()
    {
        count += 1;
        let Some(true) = instance.try_get() else { panic!("instance should be terminated successfully"); };
        instance.send_command(GameInstanceCommand::Abort).unwrap();
    }
    assert_eq!(count, 1);

    assert_eq!(cache.num_running(), 1);
    assert!(!cache.has_game(game_id_1));
    assert!(cache.has_game(game_id_2));

    // wait a bit
    std::thread::sleep(one_third_duration);        

    // add new game with user 1
    let game_id_3 = 3u64;
    let dummy_pack = DummyLaunchPack{ config: game_config, clients: Vec::default() };
    let launch_pack_3 = GameLaunchPack::new(game_id_3, dummy_pack);
    let start_request_3 = GameStartRequest{ lobby_data: LobbyData{ id: game_id_3, ..Default::default() } };
    cache.make_instance(start_request_3, launch_pack_3).expect("making game instance should succeed");

    // wait for expiration of second game
    std::thread::sleep(one_third_duration + one_third_duration);

    // remove terminated (should remove second game)
    let mut count = 0;
    for mut instance in cache.drain_invalid()
    {
        count += 1;
        let Some(true) = instance.try_get() else { panic!("instance should be terminated successfully"); };
        instance.send_command(GameInstanceCommand::Abort).unwrap();
    }
    assert_eq!(count, 1);

    assert_eq!(cache.num_running(), 1);
    assert!(!cache.has_game(game_id_2));
    assert!(cache.has_game(game_id_3));
}

//-------------------------------------------------------------------------------------------------------------------
