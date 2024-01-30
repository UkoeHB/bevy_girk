//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_player_click(player_input_sender: Res<Sender<PlayerInput>>)
{
    player_input_sender.send(PlayerInput::Play(PlayerInputPlay::ClickButton)).expect("player input receiver is missing");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_player_score(players: Query<&PlayerScore, With<PlayerId>>, mut flag: ResMut<PanicOnDrop>)
{
    assert_eq!(players.get_single().unwrap().score(), 1);
    flag.take();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Test player clicks: PlayerInput -> GameMsg -> reaction
#[test]
fn player_clicks()
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
    let ticks_per_sec = 1;

    // prepare message channels
    let mut app = App::new();
    app.add_event::<ClientPacket>();
    app.add_event::<bevy_replicon::prelude::FromClient<ClientPacket>>();
    app.add_event::<bevy_replicon::prelude::ToClients<GamePacket>>();
    app.add_event::<GamePacket>();
    let (_client_fw_command_sender, client_fw_command_reader) = new_channel::<ClientFwCommand>();
    let (player_input_sender, player_input_reader)            = new_channel::<PlayerInput>();

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

    // prepare game initializer
    let game_initializer = test_utils::prepare_game_initializer(
        num_players,
        GameDurationConfig::new(2, 3),
    );

    // prepare client initializer
    let player_context = ClickPlayerContext::new(
            SERVER_ID,
            *game_initializer.game_context.duration_config()
        );
    let player_initializer = ClickPlayerInitializer{ player_context };

    app
        //third-party plugins
        .add_plugins(bevy::time::TimePlugin)
        .add_plugins(bevy_replicon::prelude::RepliconCorePlugin)
        .init_resource::<bevy_replicon::prelude::ClientCache>()
        .add_plugins(VisibilityAttributesPlugin{ server_id: Some(SERVER_ID), reconnect_policy: ReconnectPolicy::Reset })
        .add_event::<renet::ServerEvent>()
        //setup game framework
        .insert_resource(GameFwConfig::new( ticks_per_sec, 1, 0 ))
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup components
        .set_runner(make_test_runner(8))
        .add_plugins(GameFwPlugin)
        .add_plugins(ClientFwPlugin)
        .add_plugins(GamePlugins)
        .add_plugins(ClientPlugins.build().disable::<GameReplicationPlugin>())
        .configure_sets(PreUpdate,
            (
                GameFwTickSetPrivate::FwStart,
                ClientFwSetPrivate::FwStart
            ).chain()
        )
        .configure_sets(Update, (GameFwSet, ClientFwSet::Admin).chain())
        .configure_sets(PostUpdate,
            (
                GameFwTickSetPrivate::FwEnd,
                ClientFwSetPrivate::FwEnd,
            ).chain()
        )
        .add_systems(PreUpdate, forward_client_packets.before(GameFwTickSetPrivate::FwStart))
        .add_systems(PostUpdate, forward_game_packets.after(GameFwTickSetPrivate::FwEnd))
        //game framework
        //client framework
        .insert_resource(client_fw_command_reader)
        .insert_resource(ClientFwConfig::new( ticks_per_sec, ClientId::from_raw(0u64) ))
        //game
        .insert_resource(game_initializer)
        //client core
        .insert_resource(player_initializer)
        .insert_resource(player_input_reader)
        // TEST: player clicks
        .insert_resource(PanicOnDrop::default())
        .insert_resource(player_input_sender)
        .add_systems(OnEnter(ClientCoreMode::Prep), send_player_click)  //this should fail
        .add_systems(OnEnter(ClientCoreMode::Play), send_player_click)  //this should succeed
        .add_systems(OnEnter(ClientCoreMode::GameOver), check_player_score)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
