//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::{HashMap, HashSet};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct User
{
    /// this user's environment type
    env_type: bevy_simplenet::EnvType,
    /// this user's state
    user_state: UserState,
}

impl Default for User { fn default() -> Self { unreachable!() } }

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum UserState
{
    #[default]
    Idle,
    InLobby(u64),
    InPendingLobby(u64),
    InGame(u64),
}

//-------------------------------------------------------------------------------------------------------------------

/// Tracks the state of connected users.
#[derive(Resource, Default, Debug)]
pub struct UsersCache
{
    users: HashMap<u128, User>
}

impl UsersCache
{
    /// Add user.
    /// - Returns `Err(())` if the user already exists.
    pub fn add_user(&mut self, user_id: u128, env_type: bevy_simplenet::EnvType) -> Result<(), ()>
    {
        tracing::trace!(user_id, "add user");

        // try to add the user
        let None = self.users.insert(user_id, User{ env_type, user_state: UserState::Idle })
        else { tracing::error!(user_id, "tried to add user that already exists"); return Err(()); };

        Ok(())
    }

    /// Remove user.
    /// - Returns `Err(())` if the user does not exist.
    pub fn remove_user(&mut self, user_id: u128) -> Result<(), ()>
    {
        tracing::trace!(user_id, "remove user");

        // try to remove the target user
        let Some(_) = self.users.remove(&user_id)
        else { tracing::error!(user_id, "tried to remove user that doesn't exist"); return Err(()); };

        Ok(())
    }

    /// Update user to have a new state.
    /// - Returns `Err(())` if the user does not exist.
    pub fn update_user_state(&mut self, user_id: u128, new_state: UserState) -> Result<(), ()>
    {
        tracing::trace!(user_id, ?new_state, "update user state");

        // try to access the target user
        let Some(user) = self.users.get_mut(&user_id)
        else { tracing::error!(user_id, "tried to update user that doesn't exist"); return Err(()); };

        // update the user state
        user.user_state = new_state;

        Ok(())
    }

    /// Update a set of users to have a new state.
    /// - Returns `Err(num missed)` if any of the users do not exist.
    pub fn set_user_states(&mut self, user_ids: &HashSet<u128>, new_state: UserState) -> Result<(), u16>
    {
        tracing::trace!(?user_ids, ?new_state, "set user states");

        let mut found_nonexistent = 0u16;
        for user_id in user_ids.iter()
        {
            if let Err(_) = self.update_user_state(*user_id, new_state) { found_nonexistent += 1; }
        }

        if found_nonexistent > 0 { return Err(found_nonexistent) };
        Ok(())
    }

    /// Get a user's current state.
    /// - Returns `None` if the user doesn't exist.
    pub fn get_user_state(&self, user_id: u128) -> Option<UserState>
    {
        // try to access the target user
        let user = self.users.get(&user_id)?;

        Some(user.user_state)
    }

    /// Get a user's environment type.
    /// - Returns `None` if the user doesn't exist.
    pub fn get_user_env(&self, user_id: u128) -> Option<bevy_simplenet::EnvType>
    {
        // try to access the target user
        let user = self.users.get(&user_id)?;

        Some(user.env_type)
    }

    /// Returns true if a user exists.
    pub fn has_user(&self, user_id: u128) -> bool
    {
        // try to access the target user
        let Some(_) = self.users.get(&user_id) else { return false; };

        true
    }
}

//-------------------------------------------------------------------------------------------------------------------
