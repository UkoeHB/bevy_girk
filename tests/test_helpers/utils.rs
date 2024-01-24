//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use bevy_replicon::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct GameOverFlag;

pub fn add_game_over_flag(world: &mut World)
{
    world.insert_resource(GameOverFlag);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn add_client(client_id: ClientIdType, game_fw_initializer: &mut GameFwInitializer)
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

pub fn append_client(game_fw_initializer: &mut GameFwInitializer)
{
    add_client(game_fw_initializer.clients.len() as ClientIdType, game_fw_initializer);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_player_client_contexts(num_players: usize) -> GameFwInitializer
{
    let mut game_fw_initializer = GameFwInitializer::default();

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
    // prepare message channels
    app.add_event::<ClientPacket>();
    app.add_event::<bevy_replicon::prelude::FromClient<ClientPacket>>();
    app.add_event::<bevy_replicon::prelude::ToClients<GamePacket>>();
    app.add_event::<GamePacket>();

    // kludge: enable first 10 clients
    for client_id in 0..10
    {
        app.world.resource_mut::<Events<FromClient<ClientPacket>>>().send(FromClient{
                client_id: renet::ClientId::from_raw(client_id as u64),
                event: ClientPacket{
                        send_policy : SendOrdered.into(),
                        request     : bytes::Bytes::from(ser_msg(&ClientRequestData{
                                req: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                            }))
                    }
            });
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_runner(num_ticks: u32) -> impl Fn(App) + Send + 'static
{
    move | mut app: App |
    {
        app.add_systems(OnEnter(GameFwMode::End), add_game_over_flag);

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

#[derive(Default, Resource)]
pub struct PanicOnDrop(bool);

impl PanicOnDrop
{
    pub fn take(&mut self)
    {
        self.0 = true;
    }
}

impl Drop for PanicOnDrop
{
    fn drop(&mut self)
    {
        if !self.0 { panic!("failed to reach test end condition"); }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn forward_client_packets(
    mut packets     : ResMut<Events<ClientPacket>>,
    mut from_client : EventWriter<FromClient<ClientPacket>>,
){
    for packet in packets.drain()
    {
        from_client.send(FromClient{ client_id: renet::ClientId::from_raw(0), event: packet });
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn forward_game_packets(
    mut to_clients : ResMut<Events<ToClients<GamePacket>>>,
    mut packets    : EventWriter<GamePacket>,
){
    for mut packet in to_clients.drain()
    {
        // remove the layered-in replicon change tick
        let _ = deser_bytes_partial::<RepliconTick>(&mut packet.event.message).expect("failed deserializing change tick");
        let packet = GamePacket{ send_policy: packet.event.send_policy, message: packet.event.message };

        packets.send(packet);
    }
}

//-------------------------------------------------------------------------------------------------------------------
