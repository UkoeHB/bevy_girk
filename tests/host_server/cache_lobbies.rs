//local shortcuts
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_lobbies_basic()
{
    // make a cache
    let max_request_size      = 0;
    let max_lobby_players     = 2;
    let max_lobby_watchers    = 0;
    let min_players_to_launch = max_lobby_players;
    let mut cache = LobbiesCache::new(
            LobbiesCacheConfig{
                    max_request_size,
                    lobby_checker: Box::new(BasicLobbyChecker{
                        max_lobby_players,
                        max_lobby_watchers,
                        min_players_to_launch,
                    })
                }
        );

    // make a new lobby
    let owner_id = 1000u128;
    let lobby_id = cache.new_lobby(
            owner_id,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            String::from("test"),
            Vec::default()
        ).unwrap();

    let lobby_ref = cache.lobby_ref(lobby_id).expect("should have lobby");
    let None = cache.lobby_ref(lobby_id + 1) else { panic!("should be None"); };
    assert!(lobby_ref.is_owner(owner_id));
    assert!(lobby_ref.get_password() == &String::from("test"));
    assert!(lobby_ref.get_password() != &String::from("bad_pass"));
    assert_eq!(lobby_ref.num_members(), 1);

    // add a member to the lobby
    let None = cache.lobby_ref_mut(lobby_id + 1) else { panic!("should be None"); };
    assert!(cache.try_add_member(
            lobby_id,
            1u128,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            &String::from("test")
        ));
    assert!(!cache.try_add_member(
            lobby_id,
            2u128,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            &String::from("bad_pass")
        ));
    let lobby_ref_mut = cache.lobby_ref_mut(lobby_id).expect("should have mut lobby");
    assert_eq!(lobby_ref_mut.num_members(), 2);

    // remove the lobby
    let lobby = cache.extract_lobby(lobby_id).expect("extract should work");

    let None = cache.lobby_ref(lobby_id) else { panic!("there should not be a lobby any more"); };
    let None = cache.lobby_ref_mut(lobby_id) else { panic!("there should not be a lobby any more"); };

    // put the lobby back
    cache.insert_lobby(lobby).expect("insert lobby should work");

    let _ = cache.lobby_ref(lobby_id).expect("should have lobby");
    let _ = cache.lobby_ref_mut(lobby_id).expect("should have mut lobby");

    // try to insert a lobby with the same id
    let duplicate_lobby = Lobby::new(lobby_id, owner_id, String::from("test"), Vec::default());
    let Err(_) = cache.insert_lobby(duplicate_lobby) else { panic!("duplicate lobby id insertion should fail"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_lobbies_owner_member_type()
{
    // make a cache
    let max_request_size      = 0;
    let max_lobby_players     = 1;
    let max_lobby_watchers    = 0;
    let min_players_to_launch = max_lobby_players;
    let mut cache = LobbiesCache::new(
            LobbiesCacheConfig{
                    max_request_size,
                    lobby_checker: Box::new(BasicLobbyChecker{
                        max_lobby_players,
                        max_lobby_watchers,
                        min_players_to_launch,
                    })
                }
        );

    // make a new lobby w/ player owner type (valid)
    let owner_id = 1000u128;
    let _lobby_id = cache.new_lobby(
            owner_id,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            String::from("test"),
            Vec::default()
        ).unwrap();

    // make a new lobby w/ watcher owner type (invalid)
    let owner_id = 1001u128;
    let Err(_) = cache.new_lobby(
            owner_id,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Watcher.into() },
            String::from("test"),
            Vec::default()
        )
    else { panic!("watcher owner is invalid"); };
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_lobbies_added_member_type()
{
    // make a cache
    let max_request_size      = 0;
    let max_lobby_players     = 1;
    let max_lobby_watchers    = 1;
    let min_players_to_launch = max_lobby_players;
    let mut cache = LobbiesCache::new(
            LobbiesCacheConfig{
                    max_request_size,
                    lobby_checker: Box::new(BasicLobbyChecker{
                        max_lobby_players,
                        max_lobby_watchers,
                        min_players_to_launch,
                    })
                }
        );

    // make a new lobby w/ player owner type (valid)
    let owner_id = 1000u128;
    let lobby_id = cache.new_lobby(
            owner_id,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            String::from("test"), Vec::default()
        ).unwrap();
    let lobby_ref = cache.lobby_ref(lobby_id).expect("should have lobby");
    assert_eq!(lobby_ref.num_members(), 1);
    assert!(lobby_ref.has_member(1000u128));

    // add a member to the lobby w/ watcher type (valid)
    assert!(cache.try_add_member(
            lobby_id,
            1u128,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Watcher.into() },
            &String::from("test")
        ));
    assert!(!cache.try_add_member(
            lobby_id,
            1u128,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            &String::from("test")
        ));
    let lobby_ref = cache.lobby_ref(lobby_id).expect("should have lobby");
    assert_eq!(lobby_ref.num_members(), 2);
    assert!(lobby_ref.has_member(1u128));

    // add a member to the lobby w/ any type (invalid, all slots taken)
    assert!(!cache.try_add_member(
            lobby_id,
            2u128,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            &String::from("test")
        ));
    assert!(!cache.try_add_member(
            lobby_id,
            2u128,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Watcher.into() },
            &String::from("test")
        ));
    let lobby_ref = cache.lobby_ref(lobby_id).expect("should have lobby");
    assert_eq!(lobby_ref.num_members(), 2);
    assert!(!lobby_ref.has_member(2u128));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_lobbies_search()
{
    // make a cache
    let max_request_size      = 2;
    let max_lobby_players     = 2;
    let max_lobby_watchers    = 0;
    let min_players_to_launch = max_lobby_players;
    let mut cache = LobbiesCache::new(
            LobbiesCacheConfig{
                    max_request_size,
                    lobby_checker: Box::new(BasicLobbyChecker{
                        max_lobby_players,
                        max_lobby_watchers,
                        min_players_to_launch,
                    })
                }
        );

    // search the cache
    let lobbies = get_searched_lobbies(&cache, LobbySearchRequest::LobbyId(0u64));
    assert_eq!(lobbies.len(), 0);

    let lobbies = get_searched_lobbies(&cache, LobbySearchRequest::PageOlder{ youngest_id: 0u64, num: 10u16 });
    assert_eq!(lobbies.len(), 0);

    let lobbies = get_searched_lobbies(&cache, LobbySearchRequest::PageNewer{ oldest_id: 0u64, num: 10u16 });
    assert_eq!(lobbies.len(), 0);

    // add one lobby
    let owner_id = 11u128;
    let first_lobby_id = cache.new_lobby(
            owner_id,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            String::from("test"),
            Vec::default()
        ).unwrap();

    // search the cache
    let lobbies = get_searched_lobbies(&cache, LobbySearchRequest::LobbyId(first_lobby_id));
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id);

    let lobbies = get_searched_lobbies(&cache, LobbySearchRequest::PageOlder{ youngest_id: first_lobby_id, num: 10u16 });
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id);

    let lobbies = get_searched_lobbies(&cache, LobbySearchRequest::PageNewer{ oldest_id: first_lobby_id, num: 10u16 });
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id);

    // add two lobbies
    let owner_id = 11u128;
    let second_lobby_id = cache.new_lobby(
            owner_id + 1,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            String::from("test"),
            Vec::default()
        ).unwrap();
    let third_lobby_id = cache.new_lobby(
            owner_id + 2,
            LobbyMemberData{ env: bevy_simplenet::env_type(), color: BasicLobbyMemberType::Player.into() },
            String::from("test"),
            Vec::default()
        ).unwrap();

    // search page (request more than max request size)
    let lobbies = get_searched_lobbies(&cache,
            LobbySearchRequest::PageOlder{
                    youngest_id : third_lobby_id,
                    num         : max_request_size + 1
                }
        );
    assert_eq!(lobbies.len(), 2);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id + 2);
    assert_eq!(lobbies.get(1).unwrap().owner_id, owner_id + 1);

    let lobbies = get_searched_lobbies(&cache,
            LobbySearchRequest::PageNewer{
                    oldest_id : first_lobby_id,
                    num       : max_request_size + 1
                }
        );
    assert_eq!(lobbies.len(), 2);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id);
    assert_eq!(lobbies.get(1).unwrap().owner_id, owner_id + 1);

    // search page (request 1)
    let lobbies = get_searched_lobbies(&cache,
            LobbySearchRequest::PageOlder{
                    youngest_id : second_lobby_id,
                    num         : 1u16
                }
        );
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id + 1);

    let lobbies = get_searched_lobbies(&cache,
            LobbySearchRequest::PageNewer{
                    oldest_id : second_lobby_id,
                    num       : 1u16
                }
        );
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id + 1);

    // search 'now' (most recent)
    let lobbies = get_searched_lobbies(&cache,
            LobbySearchRequest::PageOlder{
                    youngest_id : u64::MAX,
                    num         : 1u16
                }
        );
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id + 2);

    // search 'oldest'
    let lobbies = get_searched_lobbies(&cache,
            LobbySearchRequest::PageNewer{
                    oldest_id : 0u64,
                    num       : 1u16
                }
        );
    assert_eq!(lobbies.len(), 1);
    assert_eq!(lobbies.get(0).unwrap().owner_id, owner_id);
}

//-------------------------------------------------------------------------------------------------------------------
