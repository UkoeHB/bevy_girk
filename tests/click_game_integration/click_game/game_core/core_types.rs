//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The game's deterministic random number generator
#[derive(Resource)]
pub struct GameRand(Rand64);

impl GameRand
{
    pub fn new(seed: u128) -> GameRand
    {
        GameRand(Rand64::new("bevy_girk_click_test", seed))
    }

    pub fn next(&mut self) -> u64 { self.0.next() }
}

//-------------------------------------------------------------------------------------------------------------------

/// The number of ticks that have occurred since the game began (after the game was initialized).
#[derive(Resource, Default)]
pub struct GameTicksElapsed
{
    pub elapsed: TicksElapsed
}

/// The number of ticks that have occurred during [GameMode::Prep].
#[derive(Resource, Default)]
pub struct PrepTicksElapsed
{
    pub elapsed: TicksElapsed
}

/// The number of ticks that have occurred during [GameMode::Play].
#[derive(Resource, Default)]
pub struct PlayTicksElapsed
{
    pub elapsed: TicksElapsed
}

/// The number of ticks that have occurred during [GameMode::GameOver].
#[derive(Resource, Default)]
pub struct GameOverTicksElapsed
{
    pub elapsed: TicksElapsed
}

//-------------------------------------------------------------------------------------------------------------------

/// Game mode
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum GameMode
{
    #[default]
    Init,
    Prep,
    Play,
    GameOver
}

//-------------------------------------------------------------------------------------------------------------------
