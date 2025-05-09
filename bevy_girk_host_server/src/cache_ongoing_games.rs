//local shortcuts
use bevy_girk_game_instance::*;

//third-party shortcuts
use bevy::prelude::*;
use renet2_setup::{ConnectMetas, ServerConnectToken};
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::vec::Vec;

use crate::UserInfo;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OngoingGame
{
    /// This game's id.
    pub game_id: u64,
    /// Id of game hub hosting this game.
    pub game_hub_id: u128,
    /// Metadata for generating connect tokens for the game.
    pub metas: ConnectMetas,
    /// Game startup information for users (cached in case of reconnections).
    pub start_infos: Vec<GameStartInfo>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct OngoingGamesCacheConfig
{
    /// Amount of time a game may remain in the cache before it expires.
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
    /// Make a new cache.
    pub fn new(config: OngoingGamesCacheConfig) -> OngoingGamesCache
    {
        OngoingGamesCache{
                config,
                timer : Instant::now(),
                games : HashMap::default(),
                users : HashMap::default()
            }
    }

    /// Add an ongoing game.
    /// - Returns Err if the game id is already registered, or if users are already playing a game.
    pub fn add_ongoing_game(&mut self, ongoing_game: OngoingGame) -> Result<(), ()>
    {
        tracing::trace!(ongoing_game.game_id, "add ongoing game");

        // check if game already exists
        if self.games.contains_key(&ongoing_game.game_id)
        { tracing::error!(ongoing_game.game_id, "game already exists in cache"); return Err(()); }

        // add users
        for (idx, start_info) in ongoing_game.start_infos.iter().enumerate()
        {
            // add the user and continue if they were not already in a game
            let user_id = start_info.user_id;
            let Some(prev_game_id) = self.users.insert(user_id, ongoing_game.game_id) else { continue; };

            // we found a user already playing a game, so we must remove all users just added
            // - this should not happen, but needs to be handled for robustness
            tracing::error!(ongoing_game.game_id, user_id, prev_game_id, "user is already playing a game");

            for (re_idx, re_start_info) in ongoing_game.start_infos.iter().enumerate()
            {
                let _ = self.users.remove(&re_start_info.user_id);
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

    /// Remove an ongoing game.
    /// - Returns `Err(())` if the game doesn't exist.
    /// - Returns `Ok(ongoing_game)` containing the removed game.
    pub fn remove_ongoing_game(&mut self, game_id: u64) -> Result<OngoingGame, ()>
    {
        tracing::trace!(game_id, "remove ongoing game");

        // remove the game
        let Some((ongoing_game, _)) = self.games.remove(&game_id)
        else { tracing::warn!(game_id, "tried to remove game that doesn't exit"); return Err(()); };

        // remove the registered users
        for start_info in ongoing_game.start_infos.iter()
        {
            let user_id = start_info.user_id;
            if let None = self.users.remove(&user_id)
            { tracing::warn!(game_id, user_id, "tried to remove user that doesn't exit"); }
        }

        Ok(ongoing_game)
    }

    /// Get game id and game start info for a specific user (if possible).
    pub fn get_user_start_info(&self, user_id: u128, user_info: &UserInfo) -> Option<(u64, ServerConnectToken, &GameStartInfo)>
    {
        // get the game the user is in
        let Some(game_id) = self.users.get(&user_id)
        else { tracing::trace!(user_id, "tried to get start info for unknown user"); return None; };

        // get the game
        let Some((ongoing_game, _)) = self.games.get(game_id)
        else { tracing::error!(game_id, "tried to get start info for missing game"); return None; };

        // find this user in the game
        let Some(start_info) = ongoing_game.start_infos.iter().find(|i| i.user_id == user_id)
        else { tracing::error!(game_id, user_id, "tried to get user start info for missing user"); return None; };

        // make connect token for user
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        let connect_token = match ongoing_game.metas.new_connect_token(
            current_time,
            start_info.client_id,
            user_info.connection()
        )
        {
            Ok(token) => token,
            Err(err) => {
                tracing::debug!("failed getting user start info: {err:?}");
                return None;
            }
        };

        Some((*game_id, connect_token, start_info))
    }

    /// Get game id and game connect token for a specific user (if possible).
    pub fn get_user_connect_token(&self, user_id: u128, user_info: &UserInfo) -> Option<(u64, ServerConnectToken)>
    {
        self.get_user_start_info(user_id, user_info).map(|(id, token, _)| (id, token))
    }

    /// Get start infos associated with a game.
    /// - Returns `None` if the game doesn't exist.
    pub fn get_start_infos(&self, game_id: u64) -> Option<&Vec<GameStartInfo>>
    {
        // try to get the game
        let Some((ongoing_game, _)) = self.games.get(&game_id)
        else { tracing::trace!(game_id, "tried to get users for unknown game"); return None; };

        Some(&ongoing_game.start_infos)
    }

    /// Drain expired games.
    /// - Iterates over all ongoing games (may be inefficient).
    pub fn drain_expired(&mut self) -> impl IntoIterator<Item = OngoingGame> + '_
    {
        // min birth time = current time - expiry duration
        let elapsed         = self.timer.elapsed();
        let expiry_duration = self.config.expiry_duration;
        let lowest_allowed_birth_time  = elapsed.saturating_sub(expiry_duration);

        // ref the users so we can remove the ones in dead games
        let users_ref = &mut self.users;

        // retain games that have not expired
        //todo: use .extract_if once stabilized
        let mut extracted = Vec::default();
        self.games.retain(
                | _, (ongoing_game, birth_time) |
                {
                    // retain: game was born after min birth time
                    if *birth_time >= lowest_allowed_birth_time { return true; }

                    // remove: erase the dead game's users
                    for start_info in ongoing_game.start_infos.iter()
                    {
                        let _ = users_ref.remove(&start_info.user_id);
                    }

                    // remove: erase the dead game
                    tracing::trace!(ongoing_game.game_id, "removing expired game");
                    extracted.push(std::mem::take(ongoing_game));
                    false
                }
            );
        extracted
    }
}

//-------------------------------------------------------------------------------------------------------------------
