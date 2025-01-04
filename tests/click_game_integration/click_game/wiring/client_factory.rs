use std::any::type_name;

//local shortcuts
use bevy_girk_client_instance::*;
use bevy_girk_utils::*;
use bevy_girk_wiring_client::{prepare_girk_client_app, setup_girk_client_game, ClientConnectPack, GirkClientConfig, GirkClientStartupConfig};
use bevy_girk_wiring_common::ServerConnectToken;
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
    world.insert_resource(player_initializer);
    world.insert_resource(player_input_receiver);

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
    /// Stored for testing
    pub player_id: Option<ClientId>,
    /// Stored for testing
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
    type Data = ClickClientStartPack;

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

    fn setup_game(
        &mut self,
        world: &mut World,
        token: ServerConnectToken,
        start_info: ClientStartInfo<ClickClientStartPack>
    )
    {
        let connect_pack = match ClientConnectPack::new(self.expected_protocol_id, token) {
            Ok(connect) => connect,
            Err(err) => {
                tracing::error!("failed obtaining ClientConnectPack for {}: {err:?}", type_name::<Self>());
                return;
            }
        };

        // girk client config
        let config = GirkClientConfig{
            client_fw_config: start_info.data.client_fw_config,
            connect_pack,
        };

        // set up client app
        setup_girk_client_game(world, config);

        match start_info.data.initializer
        {
            ClickClientInitializer::Player(player_initializer) =>
            {
                self.player_id    = Some(player_initializer.player_context.id());
                self.player_input = Some(setup_client_game_core(world, player_initializer));
            }
            ClickClientInitializer::Watcher => ()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
