//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The current game framework tick.
#[derive(Resource, Default, Deref, Copy, Clone, Debug)]
pub struct GameFwTick(pub(crate) Tick);

//-------------------------------------------------------------------------------------------------------------------

/// The last [`GameFwTick`] before [`GameFwMode::End`].
///
/// The number of end ticks elapsed equals [`GameFwTick`] - [`GameFwPreEndTick`].
#[derive(Resource, Default, Deref, Copy, Clone, Debug)]
pub struct GameFwPreEndTick(pub(crate) Option<Tick>);

impl GameFwPreEndTick
{
    pub fn num_end_ticks(&self, game_fw_tick: GameFwTick) -> u32
    {
        let Some(pre_tick) = self.0 else { return 0; };
        (**game_fw_tick).saturating_sub(*pre_tick)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Total initialization progress of the game.
///
/// Can be replicated to clients.
/// If replicated, then the client framework will automatically reset it when the client has just disconnected.
/// This makes it easier to use when displaying a loading screen, since otherwise it won't go back to zero until
/// the client has reconnected.
#[derive(Component, Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize, Deref)]
pub struct GameInitProgress(pub(crate) f32);

impl GameInitProgress
{
    pub fn reset(&mut self)
    {
        self.0 = 0.0;
    }
}

#[derive(Bundle, Default)]
pub struct GameInitProgressEntity
{
    progress    : GameInitProgress,
    replication : Replication
}

//-------------------------------------------------------------------------------------------------------------------
