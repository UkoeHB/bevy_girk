//local shortcuts
use crate::test_helpers::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;

//third-party shortcuts

//standard shortcuts
use std::collections::VecDeque;

//-------------------------------------------------------------------------------------------------------------------

pub struct DummyGameLaunchPackSource
{
    game_config  : DummyGameConfig,
    source_works : Option<bool>,
    queue        : VecDeque<GameLaunchPackReport>,
}

impl DummyGameLaunchPackSource
{
    pub fn new(game_config: DummyGameConfig, source_works: Option<bool>) -> DummyGameLaunchPackSource
    {
        DummyGameLaunchPackSource{ game_config, source_works, queue: VecDeque::default() }
    }
}

impl GameLaunchPackSourceImpl for DummyGameLaunchPackSource
{
    /// Request a game launch pack.
    fn request_launch_pack(&mut self, start_request: &GameStartRequest)
    {
        match self.source_works
        {
            Some(true)  =>
            {
                // collect user ids
                let clients: Vec<DummyClientInit> = start_request
                    .lobby_data
                    .members
                    .iter()
                    .map(
                        |(id, _)|
                        DummyClientInit{
                            env     : bevy_simplenet::env_type(),
                            user_id : *id
                        })
                    .collect();

                // add launch pack
                let config = self.game_config.clone();
                self.queue.push_back(
                    GameLaunchPackReport::Pack(
                        GameLaunchPack::new(start_request.game_id(), DummyLaunchPack{ config, clients })
                    )
                )
            }
            Some(false) => self.queue.push_back(GameLaunchPackReport::Failure(start_request.game_id())),
            None        => (),
        }
    }

    /// Get next available report.
    fn try_next(&mut self) -> Option<GameLaunchPackReport>
    {
        self.queue.pop_front()
    }
}

//-------------------------------------------------------------------------------------------------------------------
