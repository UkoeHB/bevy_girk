//local shortcuts
use crate::test_helpers::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Ping the game framework.
#[test]
fn basic_ping()
{
    // misc.
    let num_players = 1;

    // prepare message channels
    let (client_packet_sender, client_packet_receiver) = new_channel::<ClientPacket>();
    let (game_packet_sender, game_packet_receiver)     = new_channel::<GamePacket>();

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

    // send ping request
    client_packet_sender.send(
            ClientPacket{
                    client_id   : 0 as ClientIdType,
                    send_policy : SendOrdered.into(),
                    message     :
                        ClientMessage{
                                message: AimedMsg::Fw{ bytes: ser_msg(&GameFWRequest::PingRequest(
                                                PingRequest{
                                                        timestamp_ns: 0u64
                                                    }
                                            )) }
                            }
                }
        ).unwrap();

    App::new()
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        //setup app
        .set_runner(make_test_runner(2))
        //setup game framework
        .insert_resource(GameFWConfig::new( Ticks(1), Ticks(1), Ticks(0) ))
        .insert_resource(client_packet_receiver)
        .insert_resource(game_packet_sender)
        //setup client framework
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup game core
        .insert_resource(DummyGameDurationConfig{ max_ticks: Ticks(1) })
        //add game framework
        .add_plugins(GameFWPlugin)
        //add game
        .add_plugins(DummyGameCorePlugin)
        .run();

    // expect ping response
    assert!(game_packet_receiver.len() >= 1);

    let mut found_ping_response: bool = false;

    while let Some(response) = game_packet_receiver.try_recv()
    {
        // deserialize ping response
        let AimedMsg::Fw{ bytes: serialized_message } = &response.message.message
        else { panic!("Message was not aimed at the framework!"); };
        let Some(deserialized_message) = deser_msg::<GameFWMsg>(&serialized_message[..])
        else { panic!("Deserializing game framework message failed!"); };

        // try to extract ping response
        let GameFWMsg::PingResponse(_) = deserialized_message else { continue; };

        // success
        found_ping_response = true;
    }

    if !found_ping_response
        { panic!("Did not find a ping response!"); }
}

//-------------------------------------------------------------------------------------------------------------------
