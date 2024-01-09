//local shortcuts
use crate::test_helpers::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Basic initialization of game/client frameworks.
#[test]
fn basic_fw_initialization()
{
    // misc.
    let num_players = 1;
    let ticks_per_sec = Ticks(1);

    // prepare message channels
    let (client_packet_sender, client_packet_receiver)      = new_channel::<ClientPacket>();
    let (game_packet_sender, game_packet_receiver)          = new_channel::<GamePacket>();
    let (_client_fw_comand_sender, client_fw_comand_reader) = new_channel::<ClientFWCommand>();

    // make the client ready
    client_packet_sender.send(
            ClientPacket{
                    client_id   : 0 as ClientIdType,
                    send_policy : SendOrdered.into(),
                    message     : bytes::Bytes::from(ser_msg(&ClientMessage{
                            message: AimedMsg::<_, ()>::Fw(GameFWRequest::ClientInitProgress(1.0))
                        }))
                }
        ).unwrap();

    App::new()
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        //setup app
        .set_runner(make_test_runner(2))
        //setup game framework
        .insert_resource(GameFWConfig::new( ticks_per_sec, Ticks(1), Ticks(0) ))
        .insert_resource(prepare_player_client_contexts(num_players))
        .insert_resource(client_packet_receiver)
        .insert_resource(game_packet_sender)
        //setup client framework
        .insert_resource(ClientFWConfig::new( ticks_per_sec, 0 as ClientIdType ))
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_fw_comand_reader)
        //setup game core
        .insert_resource(DummyGameDurationConfig{ max_ticks: Ticks(1) })
        //add game framework
        .add_plugins(GameFWPlugin)
        //add client framework
        .add_plugins(ClientFWPlugin)
        //add game
        .add_plugins(DummyGameCorePlugin)
        //add client
        .add_plugins(DummyClientCorePlugin)
        //configure execution flow
        .configure_sets(Update, (GameFWSet, ClientFWSet).chain())
        .run();

    //todo: validate initialization worked?
}

//-------------------------------------------------------------------------------------------------------------------
