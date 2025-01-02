//local shortcuts
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_pending_lobbies_ack()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // make a new cache
    let mut cache = PendingLobbiesCache::new(
            PendingLobbiesConfig{
                    ack_timeout  : Duration::from_secs(10),
                    start_buffer : Duration::from_secs(5),
                }
        );

    // add pending lobby with 2 users
    let lobby_id = 0u64;
    let owner_user_id = 0u128;
    let password = String::from("test");
    let mut lobby = Lobby::new(lobby_id, owner_user_id, password.clone(), Vec::default());
    assert!(lobby.add_member(
            owner_user_id,
            LobbyMemberData{ connection: ConnectionType::inferred(),  color: BasicLobbyMemberType::Player.into() }
        ));
    let user_id_1 = 1u128;
    assert!(lobby.add_member(
            user_id_1,
            LobbyMemberData{ connection: ConnectionType::inferred(),  color: BasicLobbyMemberType::Player.into() }
        ));

    let _ = cache.add_lobby(lobby.clone()).expect("add lobby should succeed");

    // try to add the lobby again
    let Err(_) = cache.add_lobby(lobby.clone()) else { panic!("add duplicate lobby should fail"); };

    // add user ack
    cache.add_user_ack(lobby_id, owner_user_id).expect("adding ack should succeed");

    // try to ack again
    let Err(_) = cache.add_user_ack(lobby_id, 84758u128) else { panic!("ack again should fail"); };

    // try to ack with a user that doesn't exist
    let Err(_) = cache.add_user_ack(lobby_id, 84758u128) else { panic!("ack from unknown user should fail"); };

    // try to ack a lobby that doesn't exist
    let Err(_) = cache.add_user_ack(54598u64, user_id_1) else { panic!("ack for unknown lobby should fail"); };

    // add final user ack
    cache.add_user_ack(lobby_id, user_id_1).expect("adding final ack should succeed");
    assert!(cache.try_get_full_acked_lobby(lobby_id).is_some());

    // remove lobby should succeed
    let _ = cache.remove_lobby(lobby_id).expect("lobby should exist");
    let Err(_) = cache.remove_lobby(lobby_id) else { panic!("lobby should be removed"); };
    assert!(cache.try_get_full_acked_lobby(lobby_id).is_none());
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_pending_lobbies_nack()
{
    // make a new cache
    let mut cache = PendingLobbiesCache::new(
            PendingLobbiesConfig{
                    ack_timeout  : Duration::from_secs(10),
                    start_buffer : Duration::from_secs(5),
                }
        );

    // add pending lobby with 2 users
    let lobby_id = 0u64;
    let owner_user_id = 0u128;
    let password = String::from("test");
    let mut lobby = Lobby::new(lobby_id, owner_user_id, password.clone(), Vec::default());
    assert!(lobby.add_member(
            owner_user_id,
            LobbyMemberData{ connection: ConnectionType::inferred(),  color: BasicLobbyMemberType::Player.into() }
        ));
    let user_id_1 = 1u128;
    assert!(lobby.add_member(
            user_id_1,
            LobbyMemberData{ connection: ConnectionType::inferred(),  color: BasicLobbyMemberType::Player.into() }
        ));

    let _ = cache.add_lobby(lobby.clone()).expect("add lobby should succeed");

    // add user ack
    cache.add_user_ack(lobby_id, owner_user_id).expect("adding ack should succeed");

    // try to nack with a user that doesn't exist
    let Err(_) = cache.remove_nacked_lobby(lobby_id, 6786u128) else { panic!("nack with unknown user should fail"); };

    // try to nack a lobby that doesn't exist
    let Err(_) = cache.remove_nacked_lobby(89490u64, user_id_1) else { panic!("nack for unknown lobby should fail"); };

    // nack the owner (after ack)
    let _: Lobby = cache.remove_nacked_lobby(lobby_id, owner_user_id).expect("nack should remove user");
    assert!(cache.try_get_full_acked_lobby(lobby_id).is_none());

    // remove lobby should fail
    let Err(_) = cache.remove_lobby(lobby_id) else { panic!("lobby should be removed"); };
    assert!(cache.try_get_full_acked_lobby(lobby_id).is_none());
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_pending_lobbies_cleanup()
{
    // lobby should be removed after ack timeout if it has insufficient acks
    // lobby should be removed after start buffer if it is fully acked and not manually removed

    // make a new cache
    let one_half_duration = Duration::from_millis(5);
    let mut cache = PendingLobbiesCache::new(
            PendingLobbiesConfig{
                    ack_timeout  : one_half_duration + one_half_duration,
                    start_buffer : one_half_duration + one_half_duration,
                }
        );

    // add pending lobby 1 with 1 user (1 ack)
    let lobby_id_1 = 1u64;
    let owner_user_id_1 = 0u128;
    let password = String::from("test");
    let mut lobby_1 = Lobby::new(lobby_id_1, owner_user_id_1, password.clone(), Vec::default());
    assert!(lobby_1.add_member(
            owner_user_id_1,
            LobbyMemberData{ connection: ConnectionType::inferred(),  color: BasicLobbyMemberType::Player.into() }
        ));

    let _ = cache.add_lobby(lobby_1).expect("add lobby should succeed");
    cache.add_user_ack(lobby_id_1, owner_user_id_1).expect("adding ack should succeed");

    // add second lobby 2 with 1 user (no acks)
    let lobby_id_2 = 2u64;
    let owner_user_id_2 = 100u128;
    let password = String::from("test");
    let mut lobby_2 = Lobby::new(lobby_id_2, owner_user_id_2, password.clone(), Vec::default());
    assert!(lobby_2.add_member(
            owner_user_id_2,
            LobbyMemberData{ connection: ConnectionType::inferred(),  color: BasicLobbyMemberType::Player.into() }
        ));

    let _ = cache.add_lobby(lobby_2).expect("add lobby should succeed");

    // drain expired should yield no lobbies
    let mut lobbies_count = 0;
    for _ in cache.drain_expired()
    {
        lobbies_count += 1;
    }
    assert_eq!(lobbies_count, 0);

    // wait for ack timeout
    std::thread::sleep(one_half_duration + one_half_duration + one_half_duration);

    // drain expired should yield lobby 2
    let mut lobbies_count = 0;
    for lobby in cache.drain_expired()
    {
        lobbies_count += 1;
        assert_eq!(lobby.id(), lobby_id_2);
    }
    assert_eq!(lobbies_count, 1);

    // wait for buffer timeout
    std::thread::sleep(one_half_duration + one_half_duration);

    // drain expired should yield lobby 1
    let mut lobbies_count = 0;
    for lobby in cache.drain_expired()
    {
        lobbies_count += 1;
        assert_eq!(lobby.id(), lobby_id_1);
    }
    assert_eq!(lobbies_count, 1);
}

//-------------------------------------------------------------------------------------------------------------------
