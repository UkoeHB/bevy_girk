//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Depends on Res<GameFwConfig>.
fn set_game_app_runner(app: &mut App)
{
    // get tick rate
    let ticks_per_sec = app.world.resource::<GameFwConfig>().ticks_per_sec();

    // add runner
    app.add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(tps_to_duration(ticks_per_sec.0)));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_collect_game_over_report(
    mut game_end_flag : ResMut<GameEndFlag>,
    runner_state      : Res<GameRunnerState>,
){
    // try to get game over report
    let Some(game_over_report) = game_end_flag.take_report() else { return; };

    // send game over report
    if let Err(_) = runner_state.report_sender.send(GameInstanceReport::GameOver(runner_state.game_id, game_over_report))
    { tracing::error!(runner_state.game_id, "failed sending game over message"); }

    // log
    tracing::info!("game over report collected from game app");
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct GameRunnerState
{
    /// This game's id.
    pub(crate) game_id: u64,
    /// Sends reports to the instance's owner.
    pub(crate) report_sender: IoSender<GameInstanceReport>,
    /// Receives commands from the instance's owner.
    pub(crate) command_receiver: IoReceiver<GameInstanceCommand>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a game app for a game instance.
/// - Makes a new game app configured for use in a game instance. Depends on `GameFwConfig`.
/// - When you run the app, it will continue updating until a game over report appears.
pub fn game_instance_setup(
    game_factory     : GameFactory,
    launch_pack      : GameLaunchPack,
    report_sender    : IoSender<GameInstanceReport>,
    command_receiver : IoReceiver<GameInstanceCommand>,
) -> Result<App, ()>
{
    let game_id = launch_pack.game_id;

    // add game to app
    let mut game_app = App::default();
    let game_start_report = game_factory.new_game(&mut game_app, launch_pack)?;

    // send game start report
    if let Err(_) = report_sender.send(GameInstanceReport::GameStart(game_id, game_start_report))
    { tracing::error!(game_id, "failed sending game start message"); return Err(()); }

    // set app runner
    set_game_app_runner(&mut game_app);

    // make runner state
    let runner_state = GameRunnerState{
            game_id,
            report_sender,
            command_receiver,
        };

    // prepare app
    game_app
        .insert_resource(runner_state)
        .add_systems(First, handle_command_incoming)
        .add_systems(Last, try_collect_game_over_report);

    // return the app
    Ok(game_app)
}

//-------------------------------------------------------------------------------------------------------------------
