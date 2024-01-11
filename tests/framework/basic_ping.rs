//local shortcuts
use crate::test_helpers::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Ping the game framework.
#[test]
fn basic_ping()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // misc.
    let num_players = 1;

    // prepare message channels
    let mut app = App::new();
    app.add_event::<ClientPacket>();
    app.add_event::<bevy_replicon::prelude::FromClient<ClientPacket>>();
    app.add_event::<bevy_replicon::prelude::ToClients<GamePacket>>();
    app.add_event::<GamePacket>();

    // make the client ready
    app.world.resource_mut::<Events<FromClient<ClientPacket>>>().send(FromClient{
            client_id: renet::ClientId::from_raw(0u64),
            event: ClientPacket{
                    send_policy : SendOrdered.into(),
                    request     : bytes::Bytes::from(ser_msg(&ClientRequestData{
                                req: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                        }))
                }
        });

    // send ping request
    app.world.resource_mut::<Events<FromClient<ClientPacket>>>().send(FromClient{
            client_id: renet::ClientId::from_raw(0u64),
            event: ClientPacket{
                    send_policy : SendUnordered.into(),
                    request     : bytes::Bytes::from(ser_msg(&ClientRequestData{
                        req: AimedMsg::<_, ()>::Fw(ClientFwRequest::GetPing(
                            PingRequest{
                                    timestamp_ns: 0u64
                                })
                    )}))
                }
        });

    app
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        .init_resource::<bevy_replicon::prelude::LastChangeTick>()
        //setup game framework
        .insert_resource(GameFwConfig::new( Ticks(1), Ticks(1), Ticks(0) ))
        .insert_resource(GameMessageBuffer::new::<()>())
        //setup client framework
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup game core
        .insert_resource(DummyGameDurationConfig{ max_ticks: Ticks(1) })
        //add game framework
        .add_plugins(GameFwPlugin)
        //add game
        .add_plugins(DummyGameCorePlugin)
        .add_systems(Update, forward_game_packets);
    app.update();
    app.update();

    // expect ping response
    let mut found_ping_response: bool = false;

    for game_packet in app.world.resource_mut::<Events<GamePacket>>().drain()
    {
        // deserialize ping response
        let Some(message) = deser_msg::<GameMessageData::<()>>(&game_packet.message[..])
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
