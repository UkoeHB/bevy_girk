//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_client_instance::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use bevy_girk_wiring::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::ClientId;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare the core of a click game client.
fn prepare_client_app_core(client_app: &mut App)
{
    // depends on client framework

    // prep client app
    client_app.add_plugins(ClientPlugins);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Prepare the core of a click game client.
fn setup_client_game_core(world: &mut World, player_initializer: ClickPlayerInitializer) -> Sender<PlayerInput>
{
    // depends on client framework

    // player input channel
    let (player_input_sender, player_input_receiver) = new_channel::<PlayerInput>();

    // make client app
    world
        .insert_resource(player_initializer)
        .insert_resource(player_input_receiver);

    player_input_sender
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Client factory for testing purposes.
///
/// If a player client is produced, the player input sender and id will be stored in the factory.
#[derive(Debug)]
pub struct ClickClientFactory
{
    expected_protocol_id: u64,
    pub player_id: Option<ClientId>,
    pub player_input: Option<Sender<PlayerInput>>,
}

impl ClickClientFactory
{
    pub fn new(expected_protocol_id: u64) -> Self
    {
        Self{ expected_protocol_id, player_id: None, player_input: None }
    }
}

impl ClientFactoryImpl for ClickClientFactory
{
    fn add_plugins(&mut self, client_app: &mut App)
    {
        // girk client config
        let config = GirkClientStartupConfig{
            resend_time: std::time::Duration::from_millis(100),
        };

        // set up client app
        client_app
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default());
        prepare_girk_client_app(client_app, config);
        prepare_client_app_core(client_app);
    }

    fn setup_game(&mut self, world: &mut World, token: ServerConnectToken, start_info: GameStartInfo)
    {
        // extract client startup pack
        let client_start_pack = deser_msg::<ClickClientStartPack>(&start_info.serialized_start_data).unwrap();

        // new connect pack
        let connect_pack = ClientConnectPack::new(self.expected_protocol_id, token).unwrap();

        // girk client config
        let config = GirkClientConfig{
            client_fw_config: client_start_pack.client_fw_config,
            connect_pack,
        };

        // set up client app
        setup_girk_client_game(world, config);

        match client_start_pack.click_client_initializer
        {
            // player
            ClickClientInitializer::Player(player_initializer) =>
            {
                self.player_id    = Some(player_initializer.player_context.id());
                self.player_input = Some(setup_client_game_core(world, player_initializer));
            }
            // watcher
            ClickClientInitializer::Watcher => ()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
