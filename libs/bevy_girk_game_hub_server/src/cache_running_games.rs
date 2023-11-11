//local shortcuts
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::collections::HashMap;
use std::time::{Duration, Instant};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct RunningGamesCacheConfig
{
    /// amount of time a game may remain in the cache before it expires
    pub expiry_duration: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

/// Games sit in this cache while running.
#[derive(Resource)]
pub struct RunningGamesCache
{
    /// cache config
    config: RunningGamesCacheConfig,
    /// game launcher
    game_launcher: GameInstanceLauncher,
    /// game instance channel
    instance_report_sender   : IOMessageSender<GameInstanceReport>,
    instance_report_receiver : IOMessageReceiver<GameInstanceReport>,
    /// timer
    timer: Instant,
    /// [ game id : (game instance, game start request birth time) ]
    games: HashMap<u64, (GameInstance, GameStartRequest, Duration)>,
}

impl RunningGamesCache
{
    /// make a new cache
    pub fn new(config: RunningGamesCacheConfig, game_launcher: GameInstanceLauncher) -> RunningGamesCache
    {
        let (instance_report_sender, instance_report_receiver) = new_io_message_channel::<GameInstanceReport>();

        RunningGamesCache{
                config,
                game_launcher,
                instance_report_sender,
                instance_report_receiver,
                timer : Instant::now(),
                games : HashMap::default(),
            }
    }

    /// make new game instance
    /// - returns `Err(())` if there is already a game instance with the given game id
    /// Note that if the game instance experiences an internal launch failure, it will by revealed by an instance
    /// report or by polling for dead instances, but not by this function.
    pub fn make_instance(&mut self, start_request: GameStartRequest, launch_pack: GameLaunchPack) -> Result<(), ()>
    {
        let game_id = launch_pack.game_id;

        // verify that start request and launch pack are consistent
        if start_request.game_id() != game_id { return Err(()); }

        // verify the game doesn't already exist
        if self.has_game(game_id) { return Err(()); }

        // launch the game instance
        let game_instance = self.game_launcher.launch(launch_pack, self.instance_report_sender.clone());

        // insert the game
        if let Some(_) = self.games.insert(game_id, (game_instance, start_request, self.timer.elapsed()))
        { tracing::error!("game instance insertion error"); }

        Ok(())
    }

    /// try to remove the game instance
    /// - returns `None` if the game instance doesn't exist
    pub fn extract_instance(&mut self, game_id: u64) -> Option<GameInstance>
    {
        self.games.remove(&game_id).map(|(instance, _, _)| instance)
    }

    /// try to access the game start request for a game instance
    /// - returns `None` if the game instance doesn't exist
    pub fn game_start_request(&self, game_id: u64) -> Option<&GameStartRequest>
    {
        self.games.get(&game_id).map(|(_, game_start_request, _)| game_start_request)
    }

    /// check if cache has a game with the given game id
    pub fn has_game(&self, game_id: u64) -> bool
    {
        self.games.contains_key(&game_id)
    }

    /// current number of running games
    pub fn num_running(&self) -> usize
    {
        self.games.len()
    }

    /// get next available instance report
    pub fn try_get_next_instance_report(&mut self) -> Option<GameInstanceReport>
    {
        self.instance_report_receiver.try_get_next()
    }

    /// drain expired and terminated running games
    /// - iterates over all running games (may be inefficient)
    /// - the caller is expected to check the game instance's status to decide how to handle it
    pub fn drain_invalid(&mut self) -> impl Iterator<Item = GameInstance> + '_
    {
        // min birth time = current time - expiry duration
        let elapsed         = self.timer.elapsed();
        let expiry_duration = self.config.expiry_duration;
        let min_birth_time  = elapsed.saturating_sub(expiry_duration);

        // retain games that have expired or terminated
        self.games.drain_filter(
                move | game_id, (running_game, _, birth_time) |
                {
                    // retain: still running and not expired
                    if running_game.try_get().is_none() && (*birth_time >= min_birth_time)
                    { return false; }

                    // remove: game has a result or is expired
                    tracing::trace!(game_id, "removing dead/expired running game");
                    true
                }
            ).map(|(_, (game_instance, _, _))| -> GameInstance { game_instance })
    }

    /// drain all running games
    pub fn drain_all(&mut self) -> impl Iterator<Item = GameInstance> + '_
    {
        self.games.drain().map(|(_, (game_instance, _, _))| -> GameInstance { game_instance })
    }
}

//-------------------------------------------------------------------------------------------------------------------
