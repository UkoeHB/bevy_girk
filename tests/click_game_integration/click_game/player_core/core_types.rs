//local shortcuts
use crate::click_game_integration::click_game::GameState;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Client core state.
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ClientCoreState
{
    #[default]
    Init,
    Play,
    GameOver
}

impl From<GameState> for ClientCoreState
{
    fn from(state: GameState) -> Self
    {
        match state
        {
            GameState::Init     => Self::Init,
            GameState::Play     => Self::Play,
            GameState::GameOver => Self::GameOver,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
