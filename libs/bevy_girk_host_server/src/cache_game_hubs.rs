//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::{BTreeSet, HashMap, HashSet};
use std::collections::hash_set::Drain;

//-------------------------------------------------------------------------------------------------------------------

/// Capacity of a game hub equals the number of new games it can launch without over-subscribing its CPU.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameHubCapacity(pub u16);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct SortableHubCapacity
{
    /// last reported hub capacity - number pending game requests
    estimated_capacity: i32,
    /// hub id
    id: u128
}

impl SortableHubCapacity
{
    fn from_state(state: &GameHubState, id: u128) -> SortableHubCapacity
    {
        SortableHubCapacity{
                estimated_capacity: (state.capacity.0 as i32) - (state.pending.len() as i32),
                id
            }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct GameHubState
{
    /// hub capacity
    capacity: GameHubCapacity,
    /// [ lobby ids ] of pending game requests
    pending: HashSet<u64>,
    /// [ lobby ids ] of games currently running on the hub
    games: HashSet<u64>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug)]
pub struct GameHubsCache
{
    /// [ hub id : hub state ]
    hubs: HashMap<u128, GameHubState>,
    /// hubs sorted by capacity (the id is baked in so hubs with equal capacity can coexist in the set)
    sorted_capacity: BTreeSet<SortableHubCapacity>
}

impl GameHubsCache
{
    /// insert a hub
    /// - capacity starts at zero
    /// - returns Err(()) if the hub already exists
    pub fn insert_hub(&mut self, hub_id: u128) -> Result<(), ()>
    {
        tracing::info!(hub_id, "insert hub");
        if self.has_hub(hub_id) { return Err(()); }
        if let Some(_) = self.hubs.insert(hub_id, GameHubState::default()) { return Err(()); }
        self.set_hub_capacity(hub_id, GameHubCapacity(0))?;

        Ok(())
    }

    /// set the capacity of a hub
    /// - returns `Err(())` if the hub doesn't exist
    pub fn set_hub_capacity(&mut self, hub_id: u128, capacity: GameHubCapacity) -> Result<(), ()>
    {
        tracing::trace!(hub_id, ?capacity, "set hub capacity");

        // access hub
        let Some(game_hub_state) = self.hubs.get_mut(&hub_id) else { return Err(()); };

        // remove existing hub capacity
        if game_hub_state.capacity != capacity
        {
            let _ = self.sorted_capacity.remove(&SortableHubCapacity::from_state(game_hub_state, hub_id));
        }

        // set new capacity
        game_hub_state.capacity = capacity;
        let _ = self.sorted_capacity.insert(SortableHubCapacity::from_state(game_hub_state, hub_id));

        Ok(())
    }

    /// remove a hub if it exists
    /// - returns `Err(())` on failure
    pub fn remove_hub(&mut self, hub_id: u128) -> Result<(), ()>
    {
        tracing::info!(hub_id, "remove hub");
        let Some(state) = self.hubs.remove(&hub_id) else { return Err(()); };
        if !self.sorted_capacity.remove(&SortableHubCapacity::from_state(&state, hub_id)) { return Err(()); }

        Ok(())
    }

    /// add pending game
    /// - returns `Ok(())` if the game was added to pending
    /// - returns `Err(())` if the hub doesn't exist or the game already exists
    pub fn add_pending_game(&mut self, hub_id: u128, lobby_id: u64) -> Result<(), ()>
    {
        tracing::trace!(hub_id, lobby_id, "add pending game");

        // access game hub
        let Some(game_hub_state) = self.hubs.get_mut(&hub_id) else { return Err(()); };

        // check if the game exists
        if game_hub_state.pending.contains(&lobby_id) { return Err(()); }
        if game_hub_state.games.contains(&lobby_id) { return Err(()); }

        // insert to pending
        let prev_sortable = SortableHubCapacity::from_state(game_hub_state, hub_id);
        if !self.sorted_capacity.remove(&prev_sortable) { return Err(()); }
        if !game_hub_state.pending.insert(lobby_id)
        {
            self.sorted_capacity.insert(prev_sortable);
            return Err(());
        }

        // update sortable hub
        let _ = self.sorted_capacity.insert(SortableHubCapacity::from_state(game_hub_state, hub_id));

        Ok(())
    }

    /// upgrade pending game to game
    /// - returns `Ok(())` if the game was moved from pending to the game list
    /// - returns `Err(())` if the hub doesn't exist, the game already exists, or the game is not pending
    pub fn upgrade_pending_game(&mut self, hub_id: u128, lobby_id: u64) -> Result<(), ()>
    {
        tracing::trace!(hub_id, lobby_id, "add game");

        // access game hub
        let Some(game_hub_state) = self.hubs.get_mut(&hub_id) else { return Err(()); };

        // remove from pending
        let prev_sortable = SortableHubCapacity::from_state(game_hub_state, hub_id);
        if !self.sorted_capacity.remove(&prev_sortable) { return Err(()); }
        if !game_hub_state.pending.remove(&lobby_id)
        {
            self.sorted_capacity.insert(prev_sortable);
            return Err(());
        }

        // update sortable hub
        let _ = self.sorted_capacity.insert(SortableHubCapacity::from_state(game_hub_state, hub_id));

        // insert to games
        if !game_hub_state.games.insert(lobby_id) { return Err(()); }

        Ok(())
    }

    /// remove pending game
    /// - returns `Err(())` if the hub or game doesn't exist
    pub fn remove_pending_game(&mut self, hub_id: u128, lobby_id: u64) -> Result<(), ()>
    {
        tracing::trace!(hub_id, lobby_id, "remove pending game");

        // get hub state
        let Some(game_hub_state) = self.hubs.get_mut(&hub_id) else { return Err(()); };

        // remove from pending
        let prev_sortable = SortableHubCapacity::from_state(game_hub_state, hub_id);
        if !self.sorted_capacity.remove(&prev_sortable) { return Err(()); }
        if !game_hub_state.pending.remove(&lobby_id)
        {
            self.sorted_capacity.insert(prev_sortable);
            return Err(());
        }

        // update sortable hub
        let _ = self.sorted_capacity.insert(SortableHubCapacity::from_state(game_hub_state, hub_id));

        Ok(())
    }

    /// remove game
    /// - returns `Err(())` if the hub or game doesn't exist
    pub fn remove_game(&mut self, hub_id: u128, lobby_id: u64) -> Result<(), ()>
    {
        tracing::trace!(hub_id, lobby_id, "remove game");
        let Some(game_hub_state) = self.hubs.get_mut(&hub_id) else { return Err(()); };
        if !game_hub_state.games.remove(&lobby_id) { return Err(()); }

        Ok(())
    }

    /// drain games
    /// - returns `Err(())` if the hub doesn't exist
    pub fn drain_games(&mut self, hub_id: u128) -> Result<Drain<'_, u64>, ()>
    {
        tracing::trace!(hub_id, "drain games");
        let Some(game_hub_state) = self.hubs.get_mut(&hub_id) else { return Err(()); };

        Ok(game_hub_state.games.drain())
    }

    /// get number of hubs
    pub fn num_hubs(&self) -> usize
    {
        self.hubs.len()
    }

    /// check if the given hub is registered
    pub fn has_hub(&self, hub_id: u128) -> bool
    {
        self.hubs.contains_key(&hub_id)
    }

    /// get id of highest-capacity hub
    /// - returns `None` if there are no hubs
    pub fn highest_capacity_hub(&self) -> Option<u128>
    {
        let last_hub = self.sorted_capacity.last()?;
        Some(last_hub.id)
    }

    /// get id of highest-capacity hub
    /// - returns `None` if the most-eligible hub has <= estimated capacity
    pub fn highest_nonzero_capacity_hub(&self) -> Option<u128>
    {
        let last_hub = self.sorted_capacity.last()?;
        if last_hub.estimated_capacity <= 0 { return None; }
        Some(last_hub.id)
    }

    /// check if the specified hub has a given pending game
    pub fn has_pending_game(&self, hub_id: u128, lobby_id: u64) -> bool
    {
        let Some(game_hub_state) = self.hubs.get(&hub_id) else { return false; };
        game_hub_state.pending.contains(&lobby_id)
    }

    /// check if the specified hub has a given game
    pub fn has_game(&self, hub_id: u128, lobby_id: u64) -> bool
    {
        let Some(game_hub_state) = self.hubs.get(&hub_id) else { return false; };
        game_hub_state.games.contains(&lobby_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------
