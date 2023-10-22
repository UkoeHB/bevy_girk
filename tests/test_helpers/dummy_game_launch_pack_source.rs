//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;

//third-party shortcuts

//standard shortcuts
use std::collections::VecDeque;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct DummyGameLaunchPackSource
{
    game_config  : Vec<u8>,
    source_works : Option<bool>,
    queue        : VecDeque<GameLaunchPackReport>,
}

impl DummyGameLaunchPackSource
{
    pub fn new(game_config: Vec<u8>, source_works: Option<bool>) -> DummyGameLaunchPackSource
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
                let client_init_data: Vec<ClientInitDataForGame> =
                    start_request
                        .lobby_data
                        .members
                        .iter()
                        .map(
                            |(id, _)|
                            ClientInitDataForGame{
                                env     : bevy_simplenet::env_type(),
                                user_id : *id,
                                data    : Vec::default()
                            })
                        .collect();

                // add launch pack
                self.queue.push_back(
                        GameLaunchPackReport::Pack(
                                GameLaunchPack{
                                        game_id        : start_request.game_id(),
                                        game_init_data : self.game_config.clone(),
                                        client_init_data,
                                    }
                            )
                    )
            }
            Some(false) => self.queue.push_back(GameLaunchPackReport::Failure(start_request.game_id())),
            None        => (),
        }
    }

    /// Get next available report.
    fn try_get_next(&mut self) -> Option<GameLaunchPackReport>
    {
        self.queue.pop_front()
    }
}

//-------------------------------------------------------------------------------------------------------------------
