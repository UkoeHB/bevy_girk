//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use bevy_girk_wiring::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use bevy_kot_utils::*;
use bevy_renet::renet::transport::ClientAuthentication;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
fn DummyClientCorePlugin(app: &mut App)
{
    app.insert_resource(GameMessageHandler::new( | _: &mut World, _: Vec<u8>, _: Ticks | -> bool { false } ));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare the core of a click game client.
pub fn prepare_client_app_core(client_app: &mut App, player_initializer: ClickPlayerInitializer) -> Sender<PlayerInput>
{
    // depends on client framework

    // player input channel
    let (player_input_sender, player_input_receiver) = new_channel::<PlayerInput>();

    // make client app
    client_app
        .add_plugins(ClientPlugins)
        .insert_resource(player_initializer)
        .insert_resource(player_input_receiver);

    player_input_sender
}

//-------------------------------------------------------------------------------------------------------------------

/// Make the core of a click game client.
///
/// Note: If the connection type is 'InMemory', then you must manually insert the in-memory client transport into the
///       client app.
pub fn make_game_client_core(
    expected_protocol_id : u64,
    connect_token        : ServerConnectToken,
    start_info           : GameStartInfo
) -> (App, Option<Sender<PlayerInput>>, Option<ClientIdType>)
{
    // extract connect token and validate protocol version
    let ServerConnectToken::Native{ bytes: serialized_connect_token } = connect_token;
    //else { panic!("only native connect tokens currently supported"); };

    let connect_token = connect_token_from_bytes(&serialized_connect_token).unwrap();
    if connect_token.protocol_id != expected_protocol_id { panic!("protocol id mismatch"); }

    // extract client startup pack
    let client_start_pack = deser_msg::<ClickClientStartPack>(&start_info.serialized_start_data).unwrap();

    // get client address based on server address
    let server_addr = connect_token.server_addresses[0].expect("only one server address is currently supported");
    let client_address = client_address_from_server_address(&server_addr);

    // set up client app
    let mut client_app = App::new();
    let mut player_input_sender : Option<Sender<PlayerInput>> = None;
    let mut player_id           : Option<ClientIdType>               = None;

    client_app.add_plugins(bevy::time::TimePlugin);
    let client_fw_command_sender = prepare_client_app_framework(&mut client_app, client_start_pack.client_fw_config);
    prepare_client_app_replication(&mut client_app, client_fw_command_sender);
    prepare_client_app_network(
            &mut client_app,
            RenetClientConnectPack::Native(ClientAuthentication::Secure{ connect_token }, client_address),
        );

    match client_start_pack.click_client_initializer
    {
        // player
        ClickClientInitializer::Player(player_initializer) =>
        {
            player_id           = Some(player_initializer.player_context.id());
            player_input_sender = Some(prepare_client_app_core(&mut client_app, player_initializer));
        }
        // watcher
        ClickClientInitializer::Watcher =>
        {
            client_app
                .add_plugins(DummyClientCorePlugin)
                .add_plugins(GameReplicationPlugin);
        }
    }

    (client_app, player_input_sender, player_id)
}

//-------------------------------------------------------------------------------------------------------------------
