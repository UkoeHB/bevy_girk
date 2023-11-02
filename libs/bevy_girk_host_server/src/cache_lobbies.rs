//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::Bound::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct LobbiesCacheConfig
{
    /// Max number of lobbies that can be requested at once.
    pub max_request_size: u16,

    /// Lobby checker.
    ///
    /// Used to validate new/inserted lobbies and new lobby members. All member-insertion rules are at the discretion of
    /// the lobby checker, including password checks.
    pub lobby_checker: Box<dyn LobbyChecker>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct LobbiesCache
{
    /// config
    config: LobbiesCacheConfig,
    /// number of lobbies generated so far (used to assign ids so lobbies will be sorted by age)
    generated_count: u64,
    /// [ lobby id : lobby data ]
    lobbies: BTreeMap<u64, Lobby>,
}

impl LobbiesCache
{
    /// Make a new cache.
    pub fn new(config: LobbiesCacheConfig) -> LobbiesCache
    {
        LobbiesCache{ config, generated_count: 0, lobbies: BTreeMap::default() }
    }

    /// Add a new lobby.
    /// - Returns `Err` if unable to make the lobby.
    pub fn new_lobby(
        &mut self,
        owner_id    : u128,
        owner_data  : LobbyMemberData,
        password    : String,
        custom_data : Vec<u8>,
    ) -> Result<u64, ()>
    {
        tracing::trace!(owner_id, "new lobby");
        self.generated_count += 1;
        let mut lobby_id = self.generated_count;

        // prepare lobby
        let mut lobby = Lobby::new(lobby_id, owner_id, password.clone(), custom_data);

        // check lobby is valid
        if !self.config.lobby_checker.check_lobby(&lobby) { return Err(()); }

        // check lobby owner can be added as a member to the lobby
        if !self.config.lobby_checker.allow_new_member(&lobby, owner_id, owner_data, &password) { return Err(()); }

        // add lobby owner as a member
        lobby.add_member(owner_id, owner_data);

        // search for a valid lobby id
        // - 'generated count' is not assumed to be unique since a lobby with an arbitrary id may be inserted
        while let Err(lobby_back) = self.insert_lobby(lobby)
        {
            self.generated_count += 1;
            lobby_id      = self.generated_count;
            lobby         = lobby_back;
            lobby.data.id = lobby_id;
        }

        Ok(lobby_id)
    }

    /// Insert a lobby that was previously extracted.
    /// - Returns Err(Lobby) if the lobby id is already registered or the lobby is invalid.
    pub fn insert_lobby(&mut self, lobby: Lobby) -> Result<(), Lobby>
    {
        tracing::trace!(lobby.data.id, "insert lobby");

        // check lobby validity
        if !self.config.lobby_checker.check_lobby(&lobby) { return Err(lobby); }

        // try to insert lobby
        if let Some(_) = self.lobby_ref(lobby.id()) { return Err(lobby); }
        let _ = self.lobbies.insert(lobby.id(), lobby);

        Ok(())
    }

    /// Add a member to the lobby.
    /// - Returns false if unable to add member (lobby doesn't exist, rejected by lobby checker).
    pub fn try_add_member(
        &mut self,
        lobby_id      : u64,
        new_member_id : u128,
        member_data   : LobbyMemberData,
        password      : &String
    ) -> bool
    {
        // check lobby exists
        let Some(lobby_ref) = self.lobby_ref(lobby_id) else { return false; };

        // check if member can be added to the lobby
        if !self.config.lobby_checker.allow_new_member(lobby_ref, new_member_id, member_data, &password) { return false; }

        // add member
        let Some(lobby_ref) = self.lobby_ref_mut(lobby_id) else { return false; };
        lobby_ref.add_member(new_member_id, member_data);

        tracing::trace!(lobby_id, new_member_id, ?member_data, "add lobby member");
        true
    }

    /// Access a specific lobby.
    pub fn lobby_ref(&self, lobby_id: u64) -> Option<&Lobby>
    {
        self.lobbies.get(&lobby_id)
    }

    /// Access a specific lobby.
    pub fn lobby_ref_mut(&mut self, lobby_id: u64) -> Option<&mut Lobby>
    {
        self.lobbies.get_mut(&lobby_id)
    }

    /// Get lobby checker.
    pub fn lobby_checker(&self) -> &dyn LobbyChecker
    {
        self.config.lobby_checker.borrow()
    }

    /// Extract a specific lobby.
    pub fn extract_lobby(&mut self, lobby_id: u64) -> Option<Lobby>
    {
        tracing::trace!(lobby_id, "remove lobby");
        self.lobbies.remove(&lobby_id)
    }

    /// Max number of requests allowed at once.
    pub fn max_request_size(&self) -> u16
    {
        self.config.max_request_size
    }

    /// Get a reference to the lobby map.
    fn lobbies_ref(&self) -> &BTreeMap<u64, Lobby>
    {
        &self.lobbies
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Get requested lobbies.
/// - The returned lobbies are sorted from newest to oldest.
pub fn get_searched_lobbies(lobbies_cache: &LobbiesCache, req: LobbySearchRequest) -> LobbySearchResult
{
    tracing::trace!(?req, "get searched lobbies from LobbiesCache");
    let (lobbies, num_younger) = match req
    {
        LobbySearchRequest::LobbyId(id) =>
        'r: {
            // count the number of lobbies younger than the requested lobby
            let num_younger = lobbies_cache.lobbies_ref().range((Excluded(&id), Unbounded)).count();

            // get the lobby if it exists
            let Some(lobby_ref) = lobbies_cache.lobby_ref(id)
            else { break 'r (Vec::default(), num_younger); };

            (
                vec![lobby_ref.data.clone()],
                num_younger
            )
        }
        LobbySearchRequest::PageNewer{ oldest_id, mut num } =>
        {
            // clamp the number of lobbies requested
            num = std::cmp::min(num, lobbies_cache.max_request_size());

            // iterate up from our start lobby (i.e. toward newer lobbies)
            let mut page_it = lobbies_cache.lobbies_ref().range((Included(&oldest_id), Unbounded));

            // collect the lobbies for this page
            let mut result = Vec::with_capacity(num as usize);

            for _ in 0..num
            {
                let Some((_, lobby)) = page_it.next() else { break; };
                result.push(lobby.data.clone());
            }

            // reverse results so they are sorted youngest to oldest
            result.reverse();

            // get id to count from
            let counter_id = match result.get(0)
            {
                None             => oldest_id,
                Some(lobby_data) => lobby_data.id,
            };

            (
                result,
                lobbies_cache.lobbies_ref().range((Excluded(&counter_id), Unbounded)).count()
            )
        }
        LobbySearchRequest::PageOlder{ youngest_id, mut num } =>
        {
            // clamp the number of lobbies requested
            num = std::cmp::min(num, lobbies_cache.max_request_size());

            // iterate down from our start lobby (i.e. toward older lobbies)
            let mut page_reverse_it = lobbies_cache.lobbies_ref().range((Unbounded, Included(&youngest_id))).rev();

            // collect the lobbies for this page
            let mut result = Vec::with_capacity(num as usize);

            for _ in 0..num
            {
                let Some((_, lobby)) = page_reverse_it.next() else { break; };
                result.push(lobby.data.clone());
            }

            // get id to count from
            let counter_id = match result.get(0)
            {
                None             => youngest_id,
                Some(lobby_data) => lobby_data.id,
            };

            (
                result,
                lobbies_cache.lobbies_ref().range((Excluded(&counter_id), Unbounded)).count()
            )
        }
    };

    LobbySearchResult{
            req,
            lobbies,
            num_younger,
            total: lobbies_cache.lobbies_ref().len(),
        }
}

//-------------------------------------------------------------------------------------------------------------------
