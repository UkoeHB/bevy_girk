//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;

//standard shortcuts
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct GameOverFlag;

pub fn add_game_over_flag(world: &mut World)
{
    world.insert_resource(GameOverFlag);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_player_client_contexts(num_players: usize) -> GameFwClients
{
    let mut clients = HashSet::default();

    for client_id in 0..num_players
    {
        clients.insert(client_id as u64);
    }

    GameFwClients::new(clients)
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_test_runner(num_ticks: u32) -> impl Fn(App) -> AppExit + Send + 'static
{
    move | mut app: App |
    {
        app.add_systems(OnEnter(GameFwState::End), add_game_over_flag);

        for i in 0..num_ticks
        {
            if app.world().contains_resource::<GameOverFlag>()
            {
                panic!("test runner failed: game over flag found at tick {} (expected: {num_ticks}) \
                    (should appear in the last tick)!", i);
            }

            app.update();
        }

        if !app.world().contains_resource::<GameOverFlag>()
        { panic!("test runner failed: game over flag not found!"); }

        AppExit::Success
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
        from_client.send(FromClient{ client_entity: SERVER, event: packet });
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn forward_game_packets(
    mut to_clients : ResMut<Events<ToClients<GamePacket>>>,
    mut packets    : EventWriter<GamePacket>,
){
    for packet in to_clients.drain()
    {
        let packet = GamePacket{ send_policy: packet.event.send_policy, message: packet.event.message };
        packets.send(packet);
    }
}

//-------------------------------------------------------------------------------------------------------------------
