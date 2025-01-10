//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_ping_tracker(ping_tracker: Res<PingTracker>, mut flag: ResMut<PanicOnDrop>)
{
    let (estimated_game_tick, _) = ping_tracker.estimate_game_tick(0u64);
    assert_eq!(estimated_game_tick, Tick(2));
    flag.take();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_test(mut next: ResMut<NextState<ClientAppState>>)
{
    next.set(ClientAppState::Game);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn request_ping(mut sender: ClientSender)
{
    sender.fw_request(ClientFwRequest::GetPing(PingRequest{
        timestamp_ns: 0u64
    }));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn basic_game_and_client()
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
    let (_player_input_sender, player_input_reader)           = new_channel::<PlayerInput>();

    // make the client ready
    app.world_mut().resource_mut::<Events<FromClient<ClientPacket>>>().send(FromClient{
        client_id: ClientId::SERVER,
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
        GameDurationConfig::new(2),
    );

    // prepare client initializer
    let player_context = ClickPlayerContext::new(
        ClientId::SERVER,
        *game_initializer.game_context.duration_config()
    );
    let player_initializer = ClickPlayerInitializer{ player_context };

    app
        //third-party plugins
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
            server_id: Some(ClientId::SERVER),
            reconnect_policy: ReconnectPolicy::Reset
        })
        //setup game framework
        .insert_resource(GameFwConfig::new(ticks_per_sec, 1, 0 ))
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup components
        .set_runner(make_test_runner(4))
        .add_plugins(GameFwPlugin)
        .add_plugins(ClientFwPlugin)
        .add_plugins(GamePlugins)
        .add_plugins(ClientPlugins.build().disable::<GameReplicationPlugin>())
        .configure_sets(PreUpdate,
            (
                GameFwSet::Start,
                ClientFwSet::Start
            )
                .chain()
                .after(bevy_replicon::prelude::ClientSet::Receive)
        )
        .configure_sets(PostUpdate,
            (
                GameFwSet::End,
                ClientFwSet::End,
            )
                .chain()
                .before(bevy_replicon::prelude::ClientSet::Send)
        )
        .add_systems(PreUpdate, forward_client_packets.before(GameFwSet::Start))
        .add_systems(PostUpdate, forward_game_packets.after(GameFwSet::End))
        //game framework
        //client framework
        .insert_resource(ClientFwConfig::new( ticks_per_sec, 0, ClientId::SERVER ))
        //game
        .insert_resource(game_initializer)
        //client core
        .insert_resource(player_initializer)
        .insert_resource(player_input_reader)
        // TEST: validate ping (in second tick because client fw needs an extra tick to collect ping response)
        .insert_resource(PanicOnDrop::default())
        .add_systems(Startup, setup_test)
        .add_systems(OnEnter(ClientFwState::Game), request_ping)
        .add_systems(Last, check_client_ping_tracker.run_if(
                |game_fw_tick: Res<GameFwTick>|
                {
                    tracing::debug!("{game_fw_tick:?}");
                    ***game_fw_tick >= 4
                }
            )
        )
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
