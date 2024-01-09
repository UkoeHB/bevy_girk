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
                    request     : bytes::Bytes::from(ser_msg(&ClientRequest{
                                req: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                        }))
                }
        ).unwrap();

    // send ping request
    client_packet_sender.send(
            ClientPacket{
                    client_id   : 0 as ClientIdType,
                    send_policy : SendOrdered.into(),
                    request     : bytes::Bytes::from(ser_msg(&ClientRequest{
                        req: AimedMsg::<_, ()>::Fw(ClientFwRequest::GetPing(
                            PingRequest{
                                    timestamp_ns: 0u64
                                })
                    )}))
                }
        ).unwrap();

    App::new()
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        //setup app
        .set_runner(make_test_runner(2))
        //setup game framework
        .insert_resource(GameFwConfig::new( Ticks(1), Ticks(1), Ticks(0) ))
        .insert_resource(client_packet_receiver)
        .insert_resource(game_packet_sender)
        .insert_resource(GameMessageBuffer::new::<()>())
        //setup client framework
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup game core
        .insert_resource(DummyGameDurationConfig{ max_ticks: Ticks(1) })
        //add game framework
        .add_plugins(GameFwPlugin)
        //add game
        .add_plugins(DummyGameCorePlugin)
        .run();

    // expect ping response
    assert!(game_packet_receiver.len() >= 1);

    let mut found_ping_response: bool = false;

    while let Some(game_packet) = game_packet_receiver.try_recv()
    {
        // deserialize ping response
        let Some(message) = deser_msg::<GameMessage::<()>>(&game_packet.message[..])
        else { panic!("failed to deserialize game fw message"); };
        let AimedMsg::Fw(msg) = message.msg else { panic!("did not receive fw message") };

        // try to extract ping response
        let GameFwMsg::PingResponse(_) = msg else { continue; };

        // success
        found_ping_response = true;
    }

    if !found_ping_response { panic!("Did not find a ping response!"); }
}

//-------------------------------------------------------------------------------------------------------------------
