//local shortcuts
use bevy_girk_game_instance::GameLaunchPack;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GameStartRequest;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub enum GameLaunchPackReport
{
    /// Contains the launch pack.
    Pack(GameLaunchPack),
    /// Records the failed game id.
    Failure(u64),
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for a game launch pack source.
///
/// A launch pack source provides startup data for setting up a game instance. This data is consumed by a
/// game instance factory, which produces [`GameConnectInfo`] for clients to use when setting up their game clients.
pub trait GameLaunchPackSourceImpl
{
    /// Request a launch pack for a specified game.
    fn request_launch_pack(&mut self, game_request: &GameStartRequest);
    /// Get the next available report.
    fn try_next(&mut self) -> Option<GameLaunchPackReport>;
}

//-------------------------------------------------------------------------------------------------------------------

/// Wraps a launch pack source implementation.
#[derive(Resource)]
pub struct GameLaunchPackSource
{
    /// Implementation
    source_impl: Box<dyn GameLaunchPackSourceImpl + Send + Sync>,
}

impl GameLaunchPackSource
{
    /// Make a new source from a source implementation.
    pub fn new<S: GameLaunchPackSourceImpl + Send + Sync + 'static>(source_impl: S) -> GameLaunchPackSource
    {
        GameLaunchPackSource{ source_impl: Box::new(source_impl) }
    }

    /// Request launch pack from the source for the given game start request.
    pub fn request_launch_pack(&mut self, game_request: &GameStartRequest)
    {
        self.source_impl.request_launch_pack(game_request);
    }

    /// Poll the next available report.
    pub fn try_next_report(&mut self) -> Option<GameLaunchPackReport>
    {
        self.source_impl.try_next()
    }
}

//-------------------------------------------------------------------------------------------------------------------
