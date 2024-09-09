//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::app::*;
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn set_schedule_runner(app: &mut App, ticks_per_sec: Option<u16>)
{
    app.add_event::<AppExit>();
    let Some(ticks_per_sec) = ticks_per_sec else { tracing::info!("set server tick to manual updating"); return; };

    app.add_plugins(ScheduleRunnerPlugin::run_loop(tps_to_duration(ticks_per_sec as u32)));
    tracing::info!(ticks_per_sec, "set server tick to timed looping");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// initialize server resources
fn init_resources(
    app                        : &mut App,
    pending_games_cache_config : PendingGamesCacheConfig,
    running_games_cache_config : RunningGamesCacheConfig,
    game_launcher              : GameInstanceLauncher,
    initial_max_capacity       : u16,
){
    app.insert_resource(TickCounter::default());
    app.insert_resource(PendingGamesCache::new(pending_games_cache_config));
    app.insert_resource(RunningGamesCache::new(running_games_cache_config, game_launcher));
    app.insert_resource(GameHubCapacityTracker::new(GameHubCapacity(initial_max_capacity)));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_shutdown(host_client: Res<HostHubClient>)
{
    // close the host server connection
    host_client.close();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct GameHubServerConfig
{
    /// tick rate (optional: no runner will be defined if `None`)
    pub ticks_per_sec: Option<u16>,
    /// initial max capacity (max number of game instances that can exist simultaneously) (may be updated via commands)
    pub initial_max_capacity: u16,
    /// number of ticks to wait between each running game cache purge
    pub running_game_purge_period_ticks: u64,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug)]
pub struct GameHubServerStartupPack
{
    /// server config
    pub game_hub_server_config: GameHubServerConfig,

    /// config: pending games cache
    pub pending_games_cache_config: PendingGamesCacheConfig,
    /// config: running games cache
    pub running_games_cache_config: RunningGamesCacheConfig,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_game_hub_server(
    startup_pack            : GameHubServerStartupPack,
    command_receiver        : Receiver<GameHubCommand>,
    host_hub_client         : HostHubClient,
    game_launch_pack_source : GameLaunchPackSource,
    game_launcher           : GameInstanceLauncher,
) -> App
{
    // make empty app
    tracing::info!("making game hub server");
    let mut app = App::new();

    // configure bevy scheduling
    app.init_schedule(Main);

    // set schedule runner
    let game_hub_server_config = startup_pack.game_hub_server_config;
    set_schedule_runner(&mut app, game_hub_server_config.ticks_per_sec);

    // initialize server resources
    init_resources(
            &mut app,
            startup_pack.pending_games_cache_config,
            startup_pack.running_games_cache_config,
            game_launcher,
            game_hub_server_config.initial_max_capacity,
        );
    app.insert_resource(command_receiver);
    app.insert_resource(host_hub_client);
    app.insert_resource(game_launch_pack_source);

    // add server systems
    app.add_systems(Main,
            (
                increment_tick_counter,
                handle_commands,
                handle_host_incoming,
                handle_launch_pack_reports,
                handle_instance_reports,  //after 'handle launch pack reports' to maybe catch failed instance launches
                clean_pending_games,      //no purge period since cache should be relatively small
                clean_running_games.run_if(
                        on_tick_counter(game_hub_server_config.running_game_purge_period_ticks)
                    ),
                update_capacity,
                handle_shutdown.run_if(on_event::<AppExit>()),
            ).chain()
        );

    app
}

//-------------------------------------------------------------------------------------------------------------------
