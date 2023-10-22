//local shortcuts
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct WatcherMap
{
    /// [ client id  ]
    watchers: HashSet<ClientIdType>,
}

impl WatcherMap
{
    pub fn new(watchers: HashSet<ClientIdType>) -> WatcherMap
    {
        WatcherMap{ watchers }
    }

    pub fn is_watcher(&self, client_id: ClientIdType) -> bool
    {
        self.watchers.contains(&client_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------
