//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_player_click(player_input_sender: Res<MessageSender<PlayerInput>>)
{
    player_input_sender.send(PlayerInput::Play(PlayerInputPlay::ClickButton)).expect("player input receiver is missing");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_player_score(players: Query<&PlayerScore, With<PlayerId>>)
{
    assert_eq!(players.get_single().unwrap().score(), 1);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Test player clicks: PlayerInput -> GameMsg -> reaction
#[test]
fn player_clicks()
{
    // misc.
    let num_players = 1;
    let ticks_per_sec = Ticks(1);

    // prepare message channels
    let (client_packet_sender, client_packet_receiver)        = new_message_channel::<ClientPacket>();
    let (game_packet_sender, game_packet_receiver)            = new_message_channel::<GamePacket>();
    let (_client_fw_command_sender, client_fw_command_reader) = new_message_channel::<ClientFWCommand>();
    let (player_input_sender, player_input_reader)            = new_message_channel::<PlayerInput>();

    // make the client ready
    client_packet_sender.send(
            ClientPacket{
                    client_id   : 0 as ClientIdType,
                    send_policy : SendOrdered.into(),
                    message     : ClientMessage{
                            message: AimedMsg::Fw{ bytes: ser_msg(&GameFWRequest::ClientInitProgress(1.0)) }
                        }
                }
        ).unwrap();

    // prepare game initializer
    let game_initializer = test_utils::prepare_game_initializer(
        num_players,
        GameDurationConfig::new(Ticks(2), Ticks(3)),
    );

    // prepare client initializer
    let player_context = ClickPlayerContext::new(
            0 as ClientIdType,
            *game_initializer.game_context.duration_config()
        );
    let player_initializer = ClickPlayerInitializer{ player_context };

    App::new()
        //third-party plugins
        .add_plugins(bevy::time::TimePlugin)
        .add_plugins(bevy_replicon::prelude::RepliconCorePlugin)
        //setup game framework
        .insert_resource(GameFWConfig::new( ticks_per_sec, Ticks(1) ))
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup components
        .set_runner(make_test_runner(8))
        .add_plugins(GameFWPlugin)
        .add_plugins(ClientFWPlugin)
        .add_plugins(GamePlugins)
        .add_plugins(ClientPlugins.build().disable::<GameReplicationPlugin>())
        .configure_sets(PreUpdate,
            (
                GameFWTickSetPrivate::FWStart,
                ClientFWTickSetPrivate::FWStart
            ).chain()
        )
        .configure_sets(Update, (GameFWSet, ClientFWSet).chain())
        .configure_sets(PostUpdate,
            (
                GameFWTickSetPrivate::FWEnd,
                ClientFWTickSetPrivate::FWEnd,
            ).chain()
        )
        //game framework
        .insert_resource(client_packet_receiver)
        .insert_resource(game_packet_sender)
        //client framework
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_fw_command_reader)
        .insert_resource(ClientFWConfig::new( ticks_per_sec, 0 as ClientIdType ))
        //game
        .insert_resource(game_initializer)
        //client core
        .insert_resource(player_initializer)
        .insert_resource(player_input_reader)
        // TEST: player clicks
        .insert_resource(player_input_sender)
        .add_systems(OnEnter(ClientCoreMode::Prep), send_player_click)  //this should fail
        .add_systems(OnEnter(ClientCoreMode::Play), send_player_click)  //this should succeed
        .add_systems(OnEnter(ClientCoreMode::GameOver), check_player_score)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
