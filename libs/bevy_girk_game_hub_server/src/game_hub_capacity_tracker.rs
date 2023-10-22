//local shortcuts
use crate::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Tracks game hub capacity.
#[derive(Resource)]
pub(crate) struct GameHubCapacityTracker
{
    /// config: maximum capacity
    max_capacity: GameHubCapacity,
    /// tracked: previously updated load
    prev_load: usize,
    /// tracked: previously updated capacity
    prev_capacity: GameHubCapacity,
}

impl GameHubCapacityTracker
{
    /// Make a new tracker.
    pub(crate) fn new(initial_max_capacity: GameHubCapacity) -> GameHubCapacityTracker
    {
        GameHubCapacityTracker{ max_capacity: initial_max_capacity, prev_load: 0, prev_capacity: initial_max_capacity }
    }

    /// Change max capacity setting.
    pub(crate) fn set_max_capacity(&mut self, new_max_capacity: GameHubCapacity)
    {
        tracing::trace!(?new_max_capacity, "setting max capacity");
        self.max_capacity = new_max_capacity;
    }

    /// Reset the tracker's change detection mechanism.
    pub(crate) fn reset(&mut self)
    {
        self.prev_capacity = GameHubCapacity(0u16);
    }

    /// Get max allowed capacity.
    //todo: use this when collecting game hub stats
    pub(crate) fn _max_capacity(&self) -> GameHubCapacity
    {
        self.max_capacity
    }

    /// Get current capacity.
    pub(crate) fn capacity(&self) -> GameHubCapacity
    {
        GameHubCapacity((self.max_capacity.0 as usize).saturating_sub(self.prev_load) as u16)
    }

    /// Get previously updated game hub capacity.
    fn prev_capacity(&self) -> GameHubCapacity
    {
        self.prev_capacity
    }

    /// Update tracked capacity with the current game hub load.
    fn set_current_load(&mut self, load: usize)
    {
        self.prev_load = load;
        self.prev_capacity = self.capacity();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Update game hub capacity tracker and notify host server if tracked capacity changes.
/// Note that no message will be sent if capacity is at zero and then gets reset after a server reconnect. We assume
/// the host server defaults to zero capacity, so there is no need to send an initial capacity of 0.
pub(crate) fn update_capacity(
    mut capacity_tracker : ResMut<GameHubCapacityTracker>,
    pending_games_cache  : Res<PendingGamesCache>,
    running_games_cache  : Res<RunningGamesCache>,
    host_client          : Res<HostHubClient>,
){
    // old capacity
    let prev_capacity = capacity_tracker.prev_capacity();

    // set the current load
    capacity_tracker.set_current_load(pending_games_cache.num_pending() + running_games_cache.num_running());

    // leave if capacity hasn't changed
    let current_capacity = capacity_tracker.capacity();
    if prev_capacity == current_capacity { return; }

    // send new capacity to host server
    if let Err(_) = host_client.send(HubToHostMsg::Capacity(current_capacity))
    { tracing::error!("failed sending game hub capacity to host"); }
}

//-------------------------------------------------------------------------------------------------------------------
