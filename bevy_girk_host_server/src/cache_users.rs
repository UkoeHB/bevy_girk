//local shortcuts
use bevy_girk_wiring_common::ConnectionType;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::{HashMap, HashSet};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct UserInfo
{
    /// this user's environment type
    env_type: bevy_simplenet::EnvType,
    /// this user's connection type
    connection: ConnectionType,
    /// this user's state
    user_state: UserState,
}

impl UserInfo
{
    pub fn new(env_type: bevy_simplenet::EnvType, connection: ConnectionType) -> Self
    {
        Self{ env_type, connection, user_state: UserState::default() }
    }

    pub fn env_type(&self) -> bevy_simplenet::EnvType
    {
        self.env_type
    }

    pub fn connection(&self) -> ConnectionType
    {
        self.connection
    }

    pub fn user_state(&self) -> UserState
    {
        self.user_state
    }

    /// Constructs self for testing.
    pub fn test() -> Self
    {
        Self{
            env_type: bevy_simplenet::EnvType::Native,
            connection: ConnectionType::Native,
            user_state: UserState::default(),
        }
    }
}

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
    users: HashMap<u128, UserInfo>
}

impl UsersCache
{
    /// Add user.
    /// - Returns `Err(())` if the user already exists.
    pub fn add_user(&mut self, user_id: u128, mut user_info: UserInfo) -> Result<(), ()>
    {
        tracing::trace!(user_id, "add user");

        // checks
        if user_info.connection == ConnectionType::Memory {
            tracing::debug!("adding user {user_id} that requested ConnectionType::Memory; converting to \
                ConnectionType::Native");
            user_info.connection = ConnectionType::Native;
        }

        // try to add the user
        let None = self.users.insert(user_id, user_info)
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

    /// Get a user's environment type.
    /// - Returns `None` if the user doesn't exist.
    pub fn get_user_info(&self, user_id: u128) -> Option<&UserInfo>
    {
        self.users.get(&user_id)
    }

    /// Get a user's current state.
    /// - Returns `None` if the user doesn't exist.
    pub fn get_user_state(&self, user_id: u128) -> Option<UserState>
    {
        self.get_user_info(user_id).map(|info| info.user_state)
    }

    /// Returns true if a user exists.
    pub fn has_user(&self, user_id: u128) -> bool
    {
        self.users.contains_key(&user_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------
