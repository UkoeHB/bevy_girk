//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
pub struct PendingLobbiesConfig
{
    /// Amount of time a lobby may pend while waiting for acks.
    pub ack_timeout: Duration,
    /// Buffer period after ack timeout to wait for a game to start before giving up.
    pub start_buffer: Duration
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct PendingLobby
{
    /// the pending lobby
    lobby: Lobby,
    /// [ ids of acked users ]
    user_acks: HashSet<u128>
}

impl PendingLobby
{
    /// Make a new pending lobby.
    fn new(lobby: Lobby) -> PendingLobby
    {
        PendingLobby{
                lobby,
                user_acks: HashSet::default(),
            }
    }

    /// Pending lobby is fully acked when all members have acked.
    fn is_fully_acked(&self) -> bool
    {
        self.user_acks.len() == self.lobby.num_members()
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct PendingLobbiesCache
{
    /// config
    config: PendingLobbiesConfig,
    /// cache timer
    timer: Instant,
    /// [ lobby id : (pending lobby, registration timestamp) ]
    pending_lobbies: HashMap<u64, (PendingLobby, Duration)>
}

impl PendingLobbiesCache
{
    /// Make a new cache.
    pub fn new(config: PendingLobbiesConfig) -> PendingLobbiesCache
    {
        PendingLobbiesCache{
                config,
                timer           : Instant::now(),
                pending_lobbies : HashMap::default(),
            }
    }

    /// Add a lobby.
    /// - returns `Err` if the lobby id is already registered
    pub fn add_lobby(&mut self, lobby: Lobby) -> Result<(), ()>
    {
        let lobby_id = lobby.id();
        tracing::trace!(lobby_id, "add pending lobby");

        // check if lobby id is registered
        if self.pending_lobbies.contains_key(&lobby_id)
        { tracing::error!(lobby_id, "game already exists when adding pending lobby"); return Err(()); }

        // insert the pending lobby
        if let Some(_) = self.pending_lobbies.insert(lobby_id, (PendingLobby::new(lobby), self.timer.elapsed()))
        { tracing::error!(lobby_id, "pending lobby insertion error"); }

        Ok(())
    }

    /// Add a user ack to a pending lobby.
    /// - returns `Err(())` if the game isn't registered or user isn't a member of the lobby or user already acked
    /// - returns `Ok(())` if ack is successful
    pub fn add_user_ack(&mut self, lobby_id: u64, user_id: u128) -> Result<(), ()>
    {
        tracing::trace!(lobby_id, user_id, "add user ack");

        // try to access the target lobby
        let Some((pending_lobby, _)) = self.pending_lobbies.get_mut(&lobby_id)
        else { tracing::debug!(lobby_id, user_id, "lobby missing for ack"); return Err(()); };

        // verify the user is a member of the lobby
        if !pending_lobby.lobby.has_member(user_id)
        { tracing::debug!(lobby_id, user_id, "user missing for ack"); return Err(()); }

        // add the ack
        if let false = pending_lobby.user_acks.insert(user_id)
        { tracing::debug!(lobby_id, user_id, "user acked multiple times"); return Err(()); }

        tracing::trace!(lobby_id, user_id, "lobby acked");
        Ok(())
    }

    /// Remove a pending lobby in response to a user no-ack.
    /// - returns `Err` if the game isn't registered or user isn't a member of the lobby
    /// - we do NOT error if the user already acked, because nacking can be used to force-abort failed pending lobbies
    pub fn remove_nacked_lobby(&mut self, lobby_id: u64, user_id: u128) -> Result<Lobby, ()>
    {
        tracing::trace!(lobby_id, user_id, "remove nacked lobby");

        // try to access the target lobby
        let Some((pending_lobby, _)) = self.pending_lobbies.get(&lobby_id)
        else { tracing::debug!(lobby_id, user_id, "lobby missing for nack"); return Err(()); };

        // verify the user is a member of the lobby
        if !pending_lobby.lobby.has_member(user_id)
        { tracing::debug!(lobby_id, user_id, "user missing for nack"); return Err(()); }

        // remove lobby in response to nack
        let Ok(lobby) = self.remove_lobby(lobby_id)
        else { tracing::error!(lobby_id, user_id, "lobby removal error for nack"); return Err(()); };

        Ok(lobby)
    }

    /// Remove a pending lobby.
    /// - returns `Err(())` if the game isn't registered
    pub fn remove_lobby(&mut self, lobby_id: u64) -> Result<Lobby, ()>
    {
        tracing::trace!(lobby_id, "remove lobby");
        let Some((pending_lobby, _)) = self.pending_lobbies.remove(&lobby_id)
        else { tracing::error!(lobby_id, "tried to remove lobby that doesn't exist"); return Err(()); };

        Ok(pending_lobby.lobby)
    }

    /// Access a lobby if it's fully acked.
    /// - returns `Some(LobbyData)` if the lobby exists and is fully acked
    pub fn try_get_full_acked_lobby(&self, lobby_id: u64) -> Option<&LobbyData>
    {
        // try to access the target lobby
        let Some((pending_lobby, _)) = self.pending_lobbies.get(&lobby_id) else { return None; };

        // check if the the lobby is fully acked
        if !pending_lobby.is_fully_acked() { return None; };

        Some(&pending_lobby.lobby.data)
    }

    /// Drain expired pending lobbies.
    /// - if lobby reached ack timeout and insufficient acks
    /// - if lobby has acks but reached the end of the game-start buffer
    pub fn drain_expired(&mut self) -> impl Iterator<Item = Lobby> + '_
    {
        let current_timestamp = self.timer.elapsed();
        let ack_timeout       = self.config.ack_timeout;
        let max_lifetime      = ack_timeout + self.config.start_buffer;

        self.pending_lobbies.drain_filter(
                move |lobby_id, (pending_lobby, birth_time)| -> bool
                {
                    // remove: if lobby has exceeded max lifetime
                    if birth_time.saturating_add(max_lifetime) < current_timestamp
                    { tracing::trace!(lobby_id, "removing expired pending lobby (max lifetime)"); return true; }

                    // keep: if lobby is still waiting for acks
                    if birth_time.saturating_add(ack_timeout) >= current_timestamp { return false; }

                    // remove: if lobby has insufficient acks
                    // - when the lifetime is between ack timeout and max lifetime, it should have full acks
                    if !pending_lobby.is_fully_acked()
                    { tracing::trace!(lobby_id, "removing expired pending lobby (ack timeout)"); return true; }

                    // keep: lobby is fully acked and waiting for a game to start
                    false
                }
            ).map(|(_, (pending_lobby, _))| -> Lobby { pending_lobby.lobby })
    }
}

//-------------------------------------------------------------------------------------------------------------------
