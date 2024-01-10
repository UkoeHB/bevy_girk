//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use crate::click_game_integration::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::utils::AHasher;
use bevy_replicon::prelude::*;

//standard shortcuts
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

//-------------------------------------------------------------------------------------------------------------------

pub fn get_test_protocol_id() -> u64
{
    let mut hasher = AHasher::default();
    "test_protocol_id".hash(&mut hasher);
    hasher.finish()
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_player_init_for_game(user_id: u128, client_id: ClientIdType) -> ClientInitDataForGame
{
    let client_init_data = ClickClientInitDataForGame::Player{
            client_id   : client_id,
            player_name : String::from("player") + stringify!(client_id),
        };

    ClientInitDataForGame{
            env     : bevy_simplenet::env_type(),
            user_id : user_id,
            data    : ser_msg(&client_init_data),
        }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_watcher_init_for_game(user_id: u128, client_id: ClientIdType) -> ClientInitDataForGame
{
    let client_init_data = ClickClientInitDataForGame::Watcher{
            client_id: client_id,
        };

    ClientInitDataForGame{
            env     : bevy_simplenet::env_type(),
            user_id : user_id,
            data    : ser_msg(&client_init_data),
        }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_game_initializer(
    num_players     : usize,
    duration_config : GameDurationConfig,
) -> ClickGameInitializer
{
    // make player states
    let mut players: HashMap<ClientIdType, PlayerState> = HashMap::default();

    for id in 0..num_players
    {
        players.insert(
                id as ClientIdType,
                PlayerState{
                        id: PlayerId{ id: id as ClientIdType },
                        name: PlayerName{ name: String::from("testname") },
                        ..Default::default()
                    }
            );
    }

    // make game context
    let game_context = ClickGameContext::new(0u128, duration_config);

    ClickGameInitializer { game_context, players, watchers: HashSet::default() }
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
    for packet in to_clients.drain()
    {
        packets.send(packet.event);
    }
}

//-------------------------------------------------------------------------------------------------------------------
