//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;
use std::time::{Duration, Instant};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct GameHubDisconnectBufferConfig
{
    /// Amount of time a game hub may remain in the buffer before it expires.
    pub expiry_duration: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

/// Buffer of recently-disconnected game hubs.
/// We buffer game hub dcs in case they are temporary, to avoid disruptions to players in on-going games.
#[derive(Resource, Debug)]
pub struct GameHubDisconnectBuffer
{
    /// config
    config: GameHubDisconnectBufferConfig,
    /// timer
    timer: Instant,
    /// [ game hub id : birth time ]
    buffer: HashMap<u128, Duration>,
}

impl GameHubDisconnectBuffer
{
    /// Make a new cache.
    pub fn new(config: GameHubDisconnectBufferConfig) -> GameHubDisconnectBuffer
    {
        GameHubDisconnectBuffer{
                config,
                timer   : Instant::now(),
                buffer : HashMap::default(),
            }
    }

    /// Buffer a disconnected game hub.
    /// - returns `Err(())` if there is already a game hub with the given id
    pub fn add_game_hub(&mut self, game_hub_id: u128) -> Result<(), ()>
    {
        tracing::trace!(game_hub_id, "buffer a disconnected game hub");

        // verify the hub doesn't already exist
        if self.has_game_hub(game_hub_id) { return Err(()); }

        // insert the buffer game
        if let Some(_) = self.buffer.insert(game_hub_id, self.timer.elapsed())
        { tracing::error!("buffer insertion error"); }

        Ok(())
    }

    /// Try to extract the buffered game hub.
    /// - returns `Err(())` if the hub doesn't exist
    pub fn remove_game_hub(&mut self, game_hub_id: u128) -> Result<(), ()>
    {
        tracing::trace!(game_hub_id, "remove buffered game hub");
        let Some(_) = self.buffer.remove(&game_hub_id) else { return Err(()); };

        Ok(())
    }

    /// Check if cache has a buffer hub with the given id.
    pub fn has_game_hub(&self, game_hub_id: u128) -> bool
    {
        self.buffer.contains_key(&game_hub_id)
    }

    /// Current number of buffered hubs.
    pub fn num_buffered(&self) -> usize
    {
        self.buffer.len()
    }

    /// Drain expired buffered hubs.
    /// - iterates over all buffered hubs (may be inefficient)
    pub fn drain_expired(&mut self) -> impl Iterator<Item = u128> + '_
    {
        // min birth time = current time - expiry duration
        let elapsed         = self.timer.elapsed();
        let expiry_duration = self.config.expiry_duration;
        let min_birth_time  = elapsed.saturating_sub(expiry_duration);

        // retain buffered hubs that have not expired
        self.buffer.drain_filter(
                move | game_hub_id, birth_time |
                {
                    // retain: hub is not expired
                    if *birth_time >= min_birth_time { return false; }

                    // remove: erase the dead hub
                    tracing::trace!(game_hub_id, "removing expired buffered game hub");
                    true
                }
            ).map(|(game_hub_id, _)| -> u128 { game_hub_id })
    }

    /// Drain all buffered hubs.
    //todo: use this for server shutdown procedure
    pub fn drain_all(&mut self) -> impl Iterator<Item = u128> + '_
    {
        self.buffer.drain().map(|(game_hub_id, _)| -> u128 { game_hub_id })
    }
}

//-------------------------------------------------------------------------------------------------------------------
