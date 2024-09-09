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

/// The current game tick (starting after the game was initialized).
#[derive(Resource, Default, Deref)]
pub struct GameTick(pub Tick);

/// The current tick within [GameState::Prep].
#[derive(Resource, Default, Deref)]
pub struct PrepTick(pub Tick);

/// The current tick within [GameState::Play].
#[derive(Resource, Default, Deref)]
pub struct PlayTick(pub Tick);

/// The current tick within [GameState::GameOver].
#[derive(Resource, Default, Deref)]
pub struct GameOverTick(pub Tick);

//-------------------------------------------------------------------------------------------------------------------

/// Game mode
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum GameState
{
    #[default]
    Init,
    Prep,
    Play,
    GameOver
}

//-------------------------------------------------------------------------------------------------------------------
