//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::fmt::Debug;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for game factory implementations.
pub trait GameFactoryImpl: Debug
{
    fn new_game(&self, app: &mut App, launch_pack: GameLaunchPack) -> Result<GameStartReport, ()>;
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps a game factory implementation in an Arc so it can be cheaply cloned and passed to new threads.
#[derive(Clone, Debug)]
pub struct GameFactory
{
    factory: Arc<dyn GameFactoryImpl + Send + Sync>
}

impl GameFactory
{
    /// Create a new game factory.
    pub fn new<F: GameFactoryImpl + Send + Sync + Debug + 'static>(factory_impl: F) -> GameFactory
    {
        GameFactory { factory: Arc::new(factory_impl) }
    }

    /// Create a new game.
    ///
    /// Returns the game's start report.
    pub fn new_game(&self, app: &mut App, launch_pack: GameLaunchPack) -> Result<GameStartReport, ()>
    {
        self.factory.new_game(app, launch_pack)
    }
}

//-------------------------------------------------------------------------------------------------------------------
