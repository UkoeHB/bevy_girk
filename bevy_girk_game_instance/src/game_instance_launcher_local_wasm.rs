use crate::{
    GameInstance, GameFactory, game_instance_setup, GameInstanceCommand, GameInstanceLauncherImpl,
    GameInstanceReport, GameLaunchPack
};
use bevy_girk_game_fw::GameFwConfig;
use bevy_girk_utils::{tps_to_duration, new_io_channel, IoSender};

use bevy::prelude::*;
use wasm_timer::Instant;

//-------------------------------------------------------------------------------------------------------------------

/// Launches a game instance in a new WASM thread.
///
/// We drive the app manually instead of using the automatic runner in order to release the thread for use by
/// the instance owner (e.g. a client game when doing local-player on web).
pub struct GameInstanceLauncherLocal
{
    game_factory: GameFactory,
}

impl GameInstanceLauncherLocal
{
    pub fn new(game_factory: GameFactory) -> Self
    {
        Self{ game_factory }
    }
}

impl GameInstanceLauncherImpl for GameInstanceLauncherLocal
{
    fn launch(
        &self,
        launch_pack: GameLaunchPack,
        report_sender: IoSender<GameInstanceReport>,
    ) -> GameInstance
    {
        let report_sender_clone = report_sender.clone();

        // prepare command channel
        let (command_sender, command_receiver) = new_io_channel::<GameInstanceCommand>();
        let command_receiver_clone = command_receiver.clone();

        // launch game thread
        let game_id = launch_pack.game_id;
        let game_factory = self.game_factory.clone();
        wasm_bindgen_futures::spawn_local(
            async move
            {
                // Make game app.
                let mut app = match game_instance_setup(
                    game_factory,
                    launch_pack,
                    report_sender_clone,
                    command_receiver_clone,
                ) {
                    Ok(app) => app,
                    Err(err) => {
                        let _ = report_sender.send(GameInstanceReport::GameAborted(game_id, err));
                        return;
                    }
                };

                // Run the loop manually until the app exits.
                let tick_duration = tps_to_duration(app.world().resource::<GameFwConfig>().ticks_per_sec());

                loop
                {
                    // Tick the app once.
                    let start = Instant::now();
                    app.update();

                    // Check if the app shut down.
                    let events = app.world().resource::<Events<AppExit>>();
                    let mut reader = events.get_reader();
                    match reader.read(events).next().cloned() {
                        None => (),
                        Some(AppExit::Success) => break,
                        Some(AppExit::Error(code)) => {
                            let _ = report_sender.send(GameInstanceReport::GameAborted(
                                game_id,
                                format!("local game instance {game_id} closed with error code {code:?}"))
                            );
                            return;
                        },
                    }

                    // Wait until the next tick is needed.
                    // - We always sleep here even if the duration is zero in order to release the thread so
                    //   a frame can be rendered if necessary.
                    // - Note that this behavior mimics bevy's ScheduleRunnerPlugin loop, which will *not* try
                    //   to 'catch up' if ticks take longer than the target duration. If fixed-update
                    //   behavior is desired then the FixedUpdate schedule should be used.
                    // - Instant::saturating_duration_since is not implemented for wasm_time::Instant. We
                    //   assume WASM instants are always monotonically increasing.
                    let end = Instant::now();
                    let duration_to_next_update = tick_duration.saturating_sub(
                        end.duration_since(start)
                    );
                    gloo_timers::future::TimeoutFuture::new(duration_to_next_update.as_millis() as u32).await;
                }
            }
        );

        // Use a fake pending result since enfync requires Send on tasks but TimeoutFuture is non-Send. We assume
        // this launcher is only used for local WASM games where the pending result will be ignored.
        GameInstance::new(game_id, command_sender, command_receiver, enfync::PendingResult::make_ready(true))
    }
}

//-------------------------------------------------------------------------------------------------------------------
