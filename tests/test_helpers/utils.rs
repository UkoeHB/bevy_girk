//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct GameOverFlag;

pub fn add_game_over_flag(world: &mut World)
{
    world.insert_resource(GameOverFlag);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn add_client(client_id: ClientIdType, game_fw_initializer: &mut GameFWInitializer)
{
    // make client state
    let dummy_client_state =
        ClientState{
                id            : ClientId::new(client_id),
                access_rights :
                    InfoAccessRights{
                            client : Some(client_id),
                            global : true
                        }
            };

    game_fw_initializer.clients.push(dummy_client_state);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn append_client(game_fw_initializer: &mut GameFWInitializer)
{
    add_client(game_fw_initializer.clients.len() as ClientIdType, game_fw_initializer);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_player_client_contexts(num_players: usize) -> GameFWInitializer
{
    let mut game_fw_initializer = GameFWInitializer::default();

    for client_id in 0..num_players
    {
        add_client(client_id as ClientIdType, &mut game_fw_initializer);
    }

    game_fw_initializer
}

//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
pub fn AddMockMessageChannelsPlugin(app: &mut App)
{
    let (game_packet_sender, game_packet_receiver)     = new_channel::<GamePacket>();
    let (client_packet_sender, client_packet_receiver) = new_channel::<ClientPacket>();

    // kludge: enable first 10 clients
    for client_id in 0..10
    {
        client_packet_sender.send(
                ClientPacket{
                        client_id   : client_id as ClientIdType,
                        send_policy : SendOrdered.into(),
                        message     : bytes::Bytes::from(ser_msg(&ClientMessage{
                                message: AimedMsg::<_, ()>::Fw(GameFWRequest::ClientInitProgress(1.0))
                            }))
                    }
            ).unwrap();
    }

    app.insert_resource(game_packet_sender)
        // save the output reader so the channel will stay open
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_packet_receiver);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_runner(num_ticks: u32) -> impl Fn(App) + Send + 'static
{
    move | mut app: App |
    {
        app.add_systems(OnEnter(GameFWMode::End), add_game_over_flag);

        for _ in 0..num_ticks
        {
            if app.world.contains_resource::<GameOverFlag>()
                { panic!("test runner failed: game over flag found too early (should appear in the last tick)!"); }

            app.update();
        }

        if !app.world.contains_resource::<GameOverFlag>()
            { panic!("test runner failed: game over flag not found!"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------
