//local shortcuts
use bevy_girk_game_instance::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OngoingGame
{
    /// this game's id
    pub game_id: u64,
    /// id of game hub hosting this game
    pub game_hub_id: u128,
    /// connection information for users (cached in case of reconnections)
    pub connect_infos: Vec<GameConnectInfo>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct OngoingGamesCacheConfig
{
    /// amount of time a game may remain in the cache before it expires
    pub expiry_duration: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

/// Tracks ongoing games that are waiting for game over reports.
#[derive(Resource)]
pub struct OngoingGamesCache
{
    /// cache config
    config: OngoingGamesCacheConfig,
    /// cache timer
    timer: Instant,
    /// [ game id : (ongoing game, registration timestamp) ]
    games: HashMap<u64, (OngoingGame, Duration)>,
    /// [ user id : game id ]
    /// note: we keep a map of user ids for efficient lookups of game connect info when users reconnect
    users: HashMap<u128, u64>
}

impl OngoingGamesCache
{
    /// make a new cache
    pub fn new(config: OngoingGamesCacheConfig) -> OngoingGamesCache
    {
        OngoingGamesCache{
                config,
                timer : Instant::now(),
                games : HashMap::default(),
                users : HashMap::default()
            }
    }

    /// add an ongoing game
    /// - returns Err if the game id is already registered, or if users are already playing a game
    pub fn add_ongoing_game(&mut self, ongoing_game: OngoingGame) -> Result<(), ()>
    {
        tracing::trace!(ongoing_game.game_id, "add ongoing game");

        // check if game already exists
        if self.games.contains_key(&ongoing_game.game_id)
        { tracing::error!(ongoing_game.game_id, "game already exists in cache"); return Err(()); }

        // add users
        for (idx, connect_info) in ongoing_game.connect_infos.iter().enumerate()
        {
            // add the user and continue if they were not already in a game
            let user_id = connect_info.user_id;
            let Some(prev_game_id) = self.users.insert(user_id, ongoing_game.game_id) else { continue; };

            // we found a user already playing a game, so we must remove all users just added
            // - this should not happen, but needs to be handled for robustness
            tracing::error!(ongoing_game.game_id, user_id, prev_game_id,
                "user is already playing a game");

            for (re_idx, re_connect_info) in ongoing_game.connect_infos.iter().enumerate()
            {
                let _ = self.users.remove(&re_connect_info.user_id);
                if re_idx >= idx { break; }
            }

            // put back the user already playing a game
            if let Some(_) = self.users.insert(user_id, prev_game_id)
            { tracing::error!("users insertion error"); }

            return Err(());
        }

        // insert the game
        if let Some(_) = self.games.insert(ongoing_game.game_id, (ongoing_game, self.timer.elapsed()))
        { tracing::error!("games insertion error"); }

        Ok(())
    }

    /// remove an ongoing game
    /// - returns `Err(())` if the game doesn't exist
    /// - returns `Ok(ongoing_game)` containing the removed game
    pub fn remove_ongoing_game(&mut self, game_id: u64) -> Result<OngoingGame, ()>
    {
        tracing::trace!(game_id, "remove ongoing game");

        // remove the game
        let Some((ongoing_game, _)) = self.games.remove(&game_id)
        else { tracing::warn!(game_id, "tried to remove game that doesn't exit"); return Err(()); };

        // remove the registered users
        for connect_info in ongoing_game.connect_infos.iter()
        {
            let user_id = connect_info.user_id;
            if let None = self.users.remove(&user_id)
            { tracing::warn!(game_id, user_id, "tried to remove user that doesn't exit"); }
        }

        Ok(ongoing_game)
    }

    /// get game id and game connect info for a specific user (if possible)
    pub fn get_user_connect_info(&self, user_id: u128) -> Option<(u64, &GameConnectInfo)>
    {
        // get the game the user is in
        let Some(game_id) = self.users.get(&user_id)
        else { tracing::trace!(user_id, "tried to get connect info for unknown user"); return None; };

        // get the game
        let Some((ongoing_game, _)) = self.games.get(game_id)
        else { tracing::error!(game_id, "tried to get connect info for missing game"); return None; };

        // find this user in the game
        for connect_info in ongoing_game.connect_infos.iter()
        {
            if connect_info.user_id != user_id { continue; }

            return Some((*game_id, connect_info));
        }

        tracing::error!(game_id, user_id, "tried to get user connect info for missing user");
        None
    }

    /// get connect infos associated with a game
    /// - returns `None` if the game doesn't exist
    pub fn get_connect_infos(&self, game_id: u64) -> Option<&Vec<GameConnectInfo>>
    {
        // try to get the game
        let Some((ongoing_game, _)) = self.games.get(&game_id)
        else { tracing::trace!(game_id, "tried to get users for unknown game"); return None; };

        Some(&ongoing_game.connect_infos)
    }

    /// drain expired games
    /// - iterates over all ongoing games (may be inefficient)
    pub fn drain_expired(&mut self) -> impl Iterator<Item = OngoingGame> + '_
    {
        // min birth time = current time - expiry duration
        let elapsed         = self.timer.elapsed();
        let expiry_duration = self.config.expiry_duration;
        let min_birth_time  = elapsed.saturating_sub(expiry_duration);

        // ref the users so we can remove the ones in dead games
        let users_ref = &mut self.users;

        // retain games that have not expired
        self.games.extract_if(
                move | _, (ongoing_game, birth_time) |
                {
                    // retain: game was born after min birth time
                    if *birth_time >= min_birth_time { return false; }

                    // remove: erase the dead game's users
                    for connect_info in ongoing_game.connect_infos.iter()
                    {
                        let _ = users_ref.remove(&connect_info.user_id);
                    }

                    // remove: erase the dead game
                    tracing::trace!(ongoing_game.game_id, "removing expired game");
                    true
                }
            ).map(|(_, (ongoing_game, _))| -> OngoingGame { ongoing_game })
    }
}

//-------------------------------------------------------------------------------------------------------------------
