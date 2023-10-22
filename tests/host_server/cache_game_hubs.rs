//local shortcuts
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[test]
fn cache_game_hubs()
{
    // make a cache
    let mut cache = GameHubsCache::default();

    // empty cache test
    assert_eq!(cache.num_hubs(), 0);
    assert_eq!(cache.highest_capacity_hub(), None);
    assert_eq!(cache.highest_nonzero_capacity_hub(), None);

    // add one hub
    cache.insert_hub(0).expect("inserting hub should succeed");
    if let Ok(_) = cache.insert_hub(0) { panic!("inserting duplicate hub should fail"); }

    assert_eq!(cache.num_hubs(), 1);
    assert_eq!(cache.highest_capacity_hub(), Some(0));
    assert_eq!(cache.highest_nonzero_capacity_hub(), None);

    // add second hub
    cache.insert_hub(1).expect("inserting hub should succeed");
    cache.set_hub_capacity(1, GameHubCapacity(1)).expect("setting capacity should succeed");

    assert_eq!(cache.num_hubs(), 2);
    assert_eq!(cache.highest_capacity_hub(), Some(1));
    assert_eq!(cache.highest_nonzero_capacity_hub(), Some(1));

    // reduce second hub's capacity
    cache.set_hub_capacity(1, GameHubCapacity(0)).expect("setting capacity should succeed");

    assert_eq!(cache.num_hubs(), 2);
    assert_eq!(cache.highest_capacity_hub(), Some(1));
    assert_eq!(cache.highest_nonzero_capacity_hub(), None);  //no hubs

    // remove hub
    cache.remove_hub(1).expect("removing hub should succeed");
    let Err(_) = cache.remove_hub(1) else { panic!("removing duplicate hub should fail"); };

    assert_eq!(cache.num_hubs(), 1);
    assert_eq!(cache.highest_capacity_hub(), Some(0));
    assert_eq!(cache.highest_nonzero_capacity_hub(), None);

    // add pending game
    cache.add_pending_game(0, 0).expect("inserting pending game should succeed");
    let Err(_) = cache.add_pending_game(0, 0) else { panic!("inserting duplicate pending game should fail"); };

    // try to remove game
    if let Ok(_) = cache.remove_game(0, 0) { panic!("removing game should fail"); }

    // remove pending game
    cache.remove_pending_game(0, 0).expect("removing pending game should succeed");
    let Err(_) = cache.remove_pending_game(0, 0) else { panic!("removing unknown pending game should fail"); };

    // add pending game
    cache.add_pending_game(0, 0).expect("inserting pending game should succeed");
    let Err(_) = cache.add_pending_game(0, 0) else { panic!("inserting duplicate pending game should fail"); };

    // upgrade to game
    cache.upgrade_pending_game(0, 0).expect("upgrading pending game should succeed");
    let Err(_) = cache.upgrade_pending_game(0, 0) else { panic!("upgrading duplicate pending game should fail"); };

    // try to remove pending game
    if let Ok(_) = cache.remove_pending_game(0, 0) { panic!("removing pending game should fail"); }

    // remove game
    cache.remove_game(0, 0).expect("removing game should succeed");
    let Err(_) = cache.remove_game(0, 0) else { panic!("removing unknown game should fail"); };
}

//-------------------------------------------------------------------------------------------------------------------
