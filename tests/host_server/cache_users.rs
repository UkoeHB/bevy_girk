//local shortcuts
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_users()
{
    // make a cache
    let mut cache = UsersCache::default();

    // add a user
    let user_id_1 = 1u128;
    let _ = cache.add_user(user_id_1, UserInfo::test()).expect("adding user should succeed");
    assert_eq!(cache.get_user_state(user_id_1).expect("user should exist"), UserState::Idle);

    // try to add the user again
    let Err(_) = cache.add_user(user_id_1, UserInfo::test()) else { panic!("readding user should fail"); };

    // try to remove a user that doesn't exist
    let Err(_) = cache.remove_user(5782u128) else { panic!("removing unknown user should fail"); };

    // try to update user that doesn't exist
    let Err(_) = cache.update_user_state(3672u128, UserState::InLobby(0u64))
    else { panic!("updating unknown user should fail"); };

    // try to access state of a user that doesn't exist
    assert!(cache.get_user_state(67656u128).is_none());

    // add another user
    let user_id_2 = 2u128;
    let _ = cache.add_user(user_id_2, UserInfo::test()).expect("adding user should succeed");

    // update user
    let new_state = UserState::InLobby(0u64);
    let _ = cache.update_user_state(user_id_1, new_state).expect("updating user should succeed");
    assert_eq!(cache.get_user_state(user_id_1).expect("user should exist"), new_state);

    // remove user
    let _ = cache.remove_user(user_id_1).expect("removing user should succeed");
}

//-------------------------------------------------------------------------------------------------------------------
