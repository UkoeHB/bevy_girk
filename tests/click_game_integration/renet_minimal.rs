//local shortcuts
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_renet::{*, transport::*};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[test]
fn renet_minimal()
{
    /*
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env().unwrap()
        .add_directive("bevy_simplenet=trace".parse().unwrap())
        .add_directive("renet=trace".parse().unwrap())
        .add_directive("renetcode=trace".parse().unwrap())
        .add_directive("bevy_replicon=trace".parse().unwrap())
        .add_directive("bevy_girk_host_server=trace".parse().unwrap())
        .add_directive("bevy_girk_game_hub_server=trace".parse().unwrap())
        .add_directive("bevy_girk_wiring=trace".parse().unwrap())
        .add_directive("bevy_girk_demo_game_core=trace".parse().unwrap())
        .add_directive("bevy_girk_game_fw=trace".parse().unwrap());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
    */

    let mut server_app = App::new();
    let mut client_app = vec![App::new()];
    server_app.add_plugins(RepliconCorePlugin)
        .add_plugins((RenetServerPlugin, NetcodeServerPlugin))
        .add_plugins(bevy::time::TimePlugin);
    client_app[0].add_plugins(RepliconCorePlugin)
        .add_plugins((RenetClientPlugin, NetcodeClientPlugin))
        .add_plugins(bevy::time::TimePlugin);

    setup_local_test_renet_network(&mut server_app, &mut client_app);

    while !client_app[0].world.resource::<RenetClient>().is_connected()
    {
        std::thread::sleep(std::time::Duration::from_millis(20));
        server_app.update();
        client_app[0].update();
    }

    assert!(client_app[0].world.resource::<RenetClient>().is_connected());
    client_app[0].world.resource_mut::<RenetClient>().disconnect();
    assert!(client_app[0].world.resource::<RenetClient>().is_disconnected());
    std::thread::sleep(std::time::Duration::from_millis(20));
    server_app.update();
    client_app[0].update();
    assert!(client_app[0].world.resource::<RenetClient>().is_disconnected());
}

//-------------------------------------------------------------------------------------------------------------------
