//local shortcuts
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;
use crate::test_helpers::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_ping_tracker(ping_tracker: Res<PingTracker>)
{
    let (estimated_game_ticks_elapsed, _) = ping_tracker.estimate_game_ticks(0u64);
    assert_eq!(estimated_game_ticks_elapsed, Ticks(1));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn basic_game_and_client()
{
    // misc.
    let num_players = 1;
    let ticks_per_sec = Ticks(1);

    // prepare message channels
    let (client_packet_sender, client_packet_receiver)        = new_channel::<ClientPacket>();
    let (game_packet_sender, game_packet_receiver)            = new_channel::<GamePacket>();
    let (_client_fw_command_sender, client_fw_command_reader) = new_channel::<ClientFwCommand>();
    let (_player_input_sender, player_input_reader)           = new_channel::<PlayerInput>();

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

    // prepare game initializer
    let game_initializer = test_utils::prepare_game_initializer(
            num_players,
            GameDurationConfig::new(Ticks(0), Ticks(0)),
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
        .insert_resource(GameFwConfig::new(ticks_per_sec, Ticks(1), Ticks(0) ))
        .insert_resource(prepare_player_client_contexts(num_players))
        //setup components
        .set_runner(make_test_runner(3))
        .add_plugins(GameFwPlugin)
        .add_plugins(ClientFwPlugin)
        .add_plugins(GamePlugins)
        .add_plugins(ClientPlugins.build().disable::<GameReplicationPlugin>())
        .configure_sets(PreUpdate,
            (
                GameFwTickSetPrivate::FwStart,
                ClientFwTickSetPrivate::FwStart
            ).chain()
        )
        .configure_sets(Update, (GameFwSet, ClientFwSet).chain())
        .configure_sets(PostUpdate,
            (
                GameFwTickSetPrivate::FwEnd,
                ClientFwTickSetPrivate::FwEnd,
            ).chain()
        )
        //game framework
        .insert_resource(client_packet_receiver)
        .insert_resource(game_packet_sender)
        //client framework
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_fw_command_reader)
        .insert_resource(ClientFwConfig::new( ticks_per_sec, 0 as ClientIdType ))
        //game
        .insert_resource(game_initializer)
        //client core
        .insert_resource(player_initializer)
        .insert_resource(player_input_reader)
        // TEST: validate ping (in second tick because client fw needs an extra tick to collect ping response)
        .add_systems(Last, check_client_ping_tracker.run_if(
                |game_fw_ticks: Res<GameFwTicksElapsed>|
                game_fw_ticks.elapsed.ticks().0 > 1
            )
        )
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
