//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::deser_msg;
use serde::Deserialize;

//standard shortcuts
use std::any::type_name;
use std::fmt::Debug;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for game factory implementations.
pub trait GameFactoryImpl: Debug
{
    /// Type of the launch pack for this factory.
    type Launch: for<'de> Deserialize<'de>;

    /// Makes a new game in the provided app.
    fn new_game(
        &self,
        app: &mut App,
        game_id: u64,
        data: Self::Launch
    ) -> Result<GameStartReport, String>;
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps a game factory implementation in an Arc so it can be cheaply cloned and passed to new threads.
#[derive(Clone)]
pub struct GameFactory
{
    callback: Arc<dyn Fn(&mut App, GameLaunchPack) -> Result<GameStartReport, String> + Send + Sync + 'static>
}

impl GameFactory
{
    /// Create a new game factory.
    pub fn new<F: GameFactoryImpl + Send + Sync + 'static>(factory_impl: F) -> GameFactory
    {
        let callback = move |app: &mut App, launch_pack: GameLaunchPack| -> Result<GameStartReport, String> {
            let Some(data) = deser_msg::<F::Launch>(&launch_pack.game_launch_data)
            else {
                return Err(format!("could not deserialize {} game factory config {}",
                    type_name::<F>(), type_name::<F::Launch>()));
            };
            factory_impl.new_game(app, launch_pack.game_id, data)
        };

        GameFactory { callback: Arc::new(callback) }
    }

    /// Create a new game.
    ///
    /// Returns the game's start report.
    pub fn new_game(
        &self,
        app: &mut App,
        launch_pack: GameLaunchPack
    ) -> Result<GameStartReport, String>
    {
        (self.callback)(app, launch_pack)
    }
}

//-------------------------------------------------------------------------------------------------------------------
