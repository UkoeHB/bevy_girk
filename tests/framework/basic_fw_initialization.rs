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
    let (_client_fw_comand_sender, client_fw_comand_reader) = new_channel::<ClientFwCommand>();

    // make the client ready
    client_packet_sender.send(
            ClientPacket{
                    client_id   : 0 as ClientIdType,
                    send_policy : SendOrdered.into(),
                    request     : bytes::Bytes::from(ser_msg(&ClientRequest{
                            req: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                        }))
                }
        ).unwrap();

    App::new()
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        //setup app
        .set_runner(make_test_runner(2))
        //setup game framework
        .insert_resource(GameFwConfig::new( ticks_per_sec, Ticks(1), Ticks(0) ))
        .insert_resource(prepare_player_client_contexts(num_players))
        .insert_resource(client_packet_receiver)
        .insert_resource(game_packet_sender)
        .insert_resource(GameMessageBuffer::new::<()>())
        //setup client framework
        .insert_resource(ClientFwConfig::new( ticks_per_sec, 0 as ClientIdType ))
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_fw_comand_reader)
        .insert_resource(ClientRequestBuffer::new::<()>())
        //setup game core
        .insert_resource(DummyGameDurationConfig{ max_ticks: Ticks(1) })
        //add game framework
        .add_plugins(GameFwPlugin)
        //add client framework
        .add_plugins(ClientFwPlugin)
        //add game
        .add_plugins(DummyGameCorePlugin)
        //add client
        .add_plugins(DummyClientCorePlugin)
        //configure execution flow
        .configure_sets(Update, (GameFwSet, ClientFwSet).chain())
        .run();

    //todo: validate initialization worked?
}

//-------------------------------------------------------------------------------------------------------------------
