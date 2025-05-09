//local shortcuts
use crate::test_helpers::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Basic initialization of game/client frameworks.
#[test]
fn basic_fw_initialization()
{
    // misc.
    let num_players = 1;
    let ticks_per_sec = 1;

    // prepare message channels
    let mut app = App::new();
    app.add_event::<ClientPacket>();
    app.add_event::<bevy_replicon::prelude::FromClient<ClientPacket>>();
    app.add_event::<bevy_replicon::prelude::ToClients<GamePacket>>();
    app.add_event::<GamePacket>();

    // make the client ready
    app.world_mut().resource_mut::<Events<FromClient<ClientPacket>>>().send(FromClient{
            client_entity: SERVER,
            event: ClientPacket{
                    send_policy : Channel::Ordered,
                    request     : bytes::Bytes::from(ser_msg(&ClientRequestData{
                            req: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                        }))
                }
        });

    app
        //bevy plugins
        .add_plugins(bevy::time::TimePlugin)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(
            RepliconPlugins
                .build()
                .set(ServerPlugin{
                    tick_policy: TickPolicy::EveryFrame,
                    visibility_policy: VisibilityPolicy::Whitelist,
                    ..Default::default()
                })
        )
        .add_plugins(VisibilityAttributesPlugin{
            server_id: Some(0),
            reconnect_policy: ReconnectPolicy::Reset
        })
        //setup app
        .set_runner(make_test_runner(2))
        //setup game framework
        .insert_resource(GameFwConfig::new( ticks_per_sec, 1, 0 ))
        .insert_resource(prepare_player_client_contexts(num_players))
        .insert_resource(GameMessageType::new::<()>())
        //setup client framework
        .insert_resource(ClientFwConfig::new( ticks_per_sec, 0, 0 ))
        .insert_resource(ClientRequestType::new::<()>())
        //setup game core
        .insert_resource(DummyGameDurationConfig{ max_ticks: 1 })
        //add game framework
        .add_plugins(GameFwPlugin)
        //add client framework
        .add_plugins(ClientFwPlugin)
        //add game
        .add_plugins(DummyGameCorePlugin)
        //add client
        .add_plugins(DummyClientCorePlugin)
        .run();

    //todo: validate initialization worked?
}

//-------------------------------------------------------------------------------------------------------------------
