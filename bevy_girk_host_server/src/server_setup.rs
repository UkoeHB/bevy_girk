//local shortcuts
use crate::*;
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
fn init_resources(app: &mut App, startup_pack: HostServerStartupPack)
{
    app.insert_resource(TickCounter::default());
    app.insert_resource(GameHubsCache::default());
    app.insert_resource(LobbiesCache::new(startup_pack.lobbies_cache_config));
    app.insert_resource(PendingLobbiesCache::new(startup_pack.pending_lobbies_cache_config));
    app.insert_resource(OngoingGamesCache::new(startup_pack.ongoing_games_cache_config));
    app.insert_resource(UsersCache::default());
    app.insert_resource(GameHubDisconnectBuffer::new(startup_pack.game_hub_disconnect_buffer_config));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct HostServerConfig
{
    /// tick rate (optional: no runner will be defined if `None`)
    pub ticks_per_sec: Option<u16>,
    /// number of ticks to wait between each ongoing game cache purge
    pub ongoing_game_purge_period_ticks: u64,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug)]
pub struct HostServerStartupPack
{
    /// server config
    pub host_server_config: HostServerConfig,

    /// config: lobbies cache
    pub lobbies_cache_config: LobbiesCacheConfig,
    /// config: pending lobbies cache
    pub pending_lobbies_cache_config: PendingLobbiesConfig,
    /// config: ongoing games cache
    pub ongoing_games_cache_config: OngoingGamesCacheConfig,

    /// config: game hub disconnect buffer
    pub game_hub_disconnect_buffer_config: GameHubDisconnectBufferConfig,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_host_server(
    startup_pack     : HostServerStartupPack,
    host_hub_server  : HostHubServer,
    host_user_server : HostUserServer,
) -> App
{
    // make empty app
    tracing::info!("making host server");
    let mut app = App::new();

    // configure bevy scheduling
    app.init_schedule(Main);

    // set schedule runner
    let host_server_config = startup_pack.host_server_config;
    set_schedule_runner(&mut app, host_server_config.ticks_per_sec);

    // initialize server resources
    init_resources(&mut app, startup_pack);
    app.insert_resource(host_hub_server);
    app.insert_resource(host_user_server);

    // add server systems
    app.add_systems(Main,
            (
                increment_tick_counter,
                handle_user_incoming,
                handle_game_hub_incoming,
                clean_pending_lobbies,  //no purge period since cache should be relatively small
                clean_ongoing_games.run_if(
                        on_tick_counter(host_server_config.ongoing_game_purge_period_ticks)
                    ),
                clean_game_hub_dc_buffer,  //no purge period since cache should be relatively small
            ).chain()
        );

    app
}

//-------------------------------------------------------------------------------------------------------------------
