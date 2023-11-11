//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;
use std::time::{Duration, Instant};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct PendingGamesCacheConfig
{
    /// amount of time a game may remain in the cache before it expires
    pub expiry_duration: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

/// Games sit in this cache while waiting for their launch packs.
#[derive(Resource, Debug)]
pub struct PendingGamesCache
{
    /// config
    config: PendingGamesCacheConfig,
    /// timer
    timer: Instant,
    /// [ game id : (game start request, birth time) ]
    pending: HashMap<u64, (GameStartRequest, Duration)>,
}

impl PendingGamesCache
{
    /// make a new cache
    pub fn new(config: PendingGamesCacheConfig) -> PendingGamesCache
    {
        PendingGamesCache{
                config,
                timer   : Instant::now(),
                pending : HashMap::default(),
            }
    }

    /// add pending game
    /// - returns `Err(())` if there is already a pending game with the given game id
    pub fn add_pending_game(&mut self, start_request: GameStartRequest) -> Result<(), ()>
    {
        let game_id = start_request.game_id();
        tracing::trace!(game_id, "add pending game");

        // verify the game doesn't already exist
        if self.has_game(game_id) { return Err(()); }

        // insert the pending game
        if let Some(_) = self.pending.insert(game_id, (start_request, self.timer.elapsed()))
        { tracing::error!("pending game insertion error"); }

        Ok(())
    }

    /// try to extract the pending game
    /// - returns `None` if the pending game doesn't exist
    pub fn extract_game(&mut self, game_id: u64) -> Option<GameStartRequest>
    {
        tracing::trace!(game_id, "remove pending game");
        self.pending.remove(&game_id).map(|(req, _)| req)
    }

    /// check if cache has a pending game with the given game id
    pub fn has_game(&self, game_id: u64) -> bool
    {
        self.pending.contains_key(&game_id)
    }

    /// current number of pending games
    pub fn num_pending(&self) -> usize
    {
        self.pending.len()
    }

    /// drain expired pending games
    /// - iterates over all pending games (may be inefficient)
    pub fn drain_expired(&mut self) -> impl Iterator<Item = GameStartRequest> + '_
    {
        // min birth time = current time - expiry duration
        let elapsed         = self.timer.elapsed();
        let expiry_duration = self.config.expiry_duration;
        let min_birth_time  = elapsed.saturating_sub(expiry_duration);

        // retain pending games that have not expired
        self.pending.drain_filter(
                move | game_id, (_, birth_time) |
                {
                    // retain: game is not expired
                    if *birth_time >= min_birth_time { return false; }

                    // remove: erase the dead game
                    tracing::trace!(game_id, "removing expired pending game");
                    true
                }
            ).map(|(_, (game_start_request, _))| -> GameStartRequest { game_start_request })
    }

    /// drain all pending games
    pub fn drain_all(&mut self) -> impl Iterator<Item = GameStartRequest> + '_
    {
        self.pending.drain().map(|(_, (game_start_request, _))| -> GameStartRequest { game_start_request })
    }
}

//-------------------------------------------------------------------------------------------------------------------
