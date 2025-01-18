//local shortcuts
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_hub_dc_buffer_basic()
{
    // make a buffer
    let config = GameHubDisconnectBufferConfig{
            expiry_duration: std::time::Duration::from_secs(0),
        };
    let mut buffer = GameHubDisconnectBuffer::new(config);

    // empty buffer test
    assert_eq!(buffer.num_buffered(), 0);

    // add one hub
    buffer.add_game_hub(0u128).expect("inserting hub should succeed");
    if let Ok(_) = buffer.add_game_hub(0u128) { panic!("inserting duplicate hub should fail"); }

    assert_eq!(buffer.num_buffered(), 1);
    assert!(buffer.has_game_hub(0u128));

    // add second hub
    buffer.add_game_hub(1u128).expect("inserting hub should succeed");

    assert_eq!(buffer.num_buffered(), 2);
    assert!(buffer.has_game_hub(1u128));

    // remove hub
    buffer.remove_game_hub(1u128).expect("removing hub should succeed");
    let Err(_) = buffer.remove_game_hub(1u128) else { panic!("removing duplicate hub should fail"); };

    assert_eq!(buffer.num_buffered(), 1);
    assert!(buffer.has_game_hub(0u128));
    assert!(!buffer.has_game_hub(1u128));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_hub_dc_buffer_expired()
{
    // make a buffer
    let config = GameHubDisconnectBufferConfig{
            expiry_duration: std::time::Duration::from_secs(0),
        };
    let mut buffer = GameHubDisconnectBuffer::new(config);

    // add one hub
    buffer.add_game_hub(0u128).expect("inserting hub should succeed");
    if let Ok(_) = buffer.add_game_hub(0u128) { panic!("inserting duplicate hub should fail"); }

    assert_eq!(buffer.num_buffered(), 1);
    assert!(buffer.has_game_hub(0u128));

    // drain expired hub
    let expired: Vec<u128> = buffer.drain_expired().into_iter().collect();
    assert_eq!(expired.len(), 1);
    assert_eq!(*expired.get(0).unwrap(), 0u128);
    assert!(!buffer.has_game_hub(0u128));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn game_hub_dc_buffer_unexpired()
{
    // make a buffer
    let config = GameHubDisconnectBufferConfig{
            expiry_duration: std::time::Duration::from_secs(1),
        };
    let mut buffer = GameHubDisconnectBuffer::new(config);

    // add one hub
    buffer.add_game_hub(0u128).expect("inserting hub should succeed");
    if let Ok(_) = buffer.add_game_hub(0u128) { panic!("inserting duplicate hub should fail"); }

    assert_eq!(buffer.num_buffered(), 1);
    assert!(buffer.has_game_hub(0u128));

    // drain expired hubs (none have expired)
    let expired: Vec<u128> = buffer.drain_expired().into_iter().collect();
    assert_eq!(expired.len(), 0);
    assert!(buffer.has_game_hub(0u128));
}

//-------------------------------------------------------------------------------------------------------------------
