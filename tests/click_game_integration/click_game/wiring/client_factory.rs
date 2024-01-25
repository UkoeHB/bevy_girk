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
use bevy_fn_plugin::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
fn DummyClientCorePlugin(app: &mut App)
{
    app.insert_resource(GameMessageHandler::new(
            | _: &mut World, packet: &GamePacket | -> Result<(), Option<(Tick, GameFwMsg)>>
            {
                deserialize_game_message::<()>(packet)?;
                Ok(())
            }
        ));
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
    fn new_client(&mut self, token: ServerConnectToken, start_info: GameStartInfo) -> Result<(App, u64), ()>
    {
        // extract client startup pack
        let client_start_pack = deser_msg::<ClickClientStartPack>(&start_info.serialized_start_data).unwrap();

        // new connect pack
        let connect_pack = RenetClientConnectPack::new(self.expected_protocol_id, token).unwrap();

        // girk client config
        let config = GirkClientConfig{
            client_fw_config: client_start_pack.client_fw_config,
            resend_time: std::time::Duration::from_millis(100),
            connect_pack,
        };

        // set up client app
        let mut client_app = App::new();

        client_app.add_plugins(bevy::time::TimePlugin);
        prepare_girk_client_app(&mut client_app, config);

        match client_start_pack.click_client_initializer
        {
            // player
            ClickClientInitializer::Player(player_initializer) =>
            {
                self.player_id    = Some(player_initializer.player_context.id());
                self.player_input = Some(prepare_client_app_core(&mut client_app, player_initializer));
            }
            // watcher
            ClickClientInitializer::Watcher =>
            {
                client_app
                    .add_plugins(DummyClientCorePlugin)
                    .insert_resource(ClientRequestType::new::<GameRequest>())
                    .add_plugins(GameReplicationPlugin);
            }
        }

        Ok((client_app, self.expected_protocol_id))
    }
}

//-------------------------------------------------------------------------------------------------------------------
