//local shortcuts
use crate::test_helpers::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn basic_game()
{
    // prepare game instance layncher
    let (report_sender, mut report_receiver) = new_io_message_channel::<GameInstanceReport>();
    let factory = GameFactory::new(DummyGameFactory{});
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(factory));

    // game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : Ticks(100),
            game_duration_ticks : Ticks(2),
        };
    let game_config_ser = ser_msg(&game_config);


    // make game instance
    let game_id = 1u64;
    let launch_pack = GameLaunchPack::new(game_id, game_config_ser, Vec::default());
    let mut instance = game_launcher.launch(launch_pack, report_sender);
    assert!(instance.is_running());
    assert!(instance.try_get().is_none());
    std::thread::sleep(Duration::from_millis(5));

    // - game start report
    let Some(GameInstanceReport::GameStart(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id);


    // wait until game should be done
    std::thread::sleep(Duration::from_millis(50));
    assert!(!instance.is_running());
    assert!(instance.try_get().unwrap());
    assert!(instance.try_get().unwrap());

    // - game over report
    let Some(GameInstanceReport::GameOver(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn two_games()
{
    // prepare game instance factory
    let (report_sender, mut report_receiver) = new_io_message_channel::<GameInstanceReport>();
    let factory = GameFactory::new(DummyGameFactory{});
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(factory));

    // game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : Ticks(100),
            game_duration_ticks : Ticks(6),
        };
    let game_config_ser = ser_msg(&game_config);


    // make game instance 1
    let game_id1 = 1u64;
    let launch_pack1 = GameLaunchPack::new(game_id1, game_config_ser.clone(), Vec::default());
    let mut instance1 = game_launcher.launch(launch_pack1, report_sender.clone());
    assert!(instance1.is_running());


    // wait for instance 1 to tick a little
    std::thread::sleep(Duration::from_millis(40));


    // make game instance 2
    let game_id2 = 2u64;
    let launch_pack2 = GameLaunchPack::new(game_id2, game_config_ser, Vec::default());
    let mut instance2 = game_launcher.launch(launch_pack2, report_sender);
    assert!(instance2.is_running());
    std::thread::sleep(Duration::from_millis(5));

    // - game start report for game 1
    let Some(GameInstanceReport::GameStart(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id1);

    // - game start report for game 2
    let Some(GameInstanceReport::GameStart(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id2);


    // wait until game 1 should be done
    std::thread::sleep(Duration::from_millis(25));
    assert!(instance1.try_get().unwrap());
    assert!(instance2.try_get().is_none());

    // - game over report for game 1
    let Some(GameInstanceReport::GameOver(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id1);


    // wait until game 2 should be done
    std::thread::sleep(Duration::from_millis(40));
    assert!(instance2.try_get().unwrap());
    assert!(instance1.try_get().unwrap());

    // - game over report for game 2
    let Some(GameInstanceReport::GameOver(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id2);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn abort_game()
{
    // prepare game instance factory
    let (report_sender, mut report_receiver) = new_io_message_channel::<GameInstanceReport>();
    let factory = GameFactory::new(DummyGameFactory{});
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherLocal::new(factory));

    // game config
    let game_config = DummyGameConfig{
            ticks_per_sec       : Ticks(100),
            game_duration_ticks : Ticks(10),
        };
    let game_config_ser = ser_msg(&game_config);


    // make game instance
    let game_id = 1u64;
    let launch_pack = GameLaunchPack::new(game_id, game_config_ser, Vec::default());
    let mut instance = game_launcher.launch(launch_pack, report_sender);
    assert!(instance.is_running());
    std::thread::sleep(Duration::from_millis(5));

    // - game start report
    let Some(GameInstanceReport::GameStart(id, _)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id);


    // abort the game
    instance.send_command(GameInstanceCommand::AbortGame).expect("send instance command should succeed");
    std::thread::sleep(Duration::from_millis(15));
    assert!(instance.try_get().unwrap());

    // - game aborted report
    let Some(GameInstanceReport::GameAborted(id)) = report_receiver.try_get_next()
    else { panic!("did not receive game instance report"); };
    assert_eq!(id, game_id);
}

//-------------------------------------------------------------------------------------------------------------------
