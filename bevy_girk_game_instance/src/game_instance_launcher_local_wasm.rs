//local shortcuts
use crate::{
    GameFactory, GameFwConfig, game_instance_setup, GameInstanceCommand, GameInstanceReport, GameLaunchPack
};

//third-party shortcuts
use enfync::{AdoptOrDefault, Handle};
use bevy_girk_utils::tps_to_duration;
use wasmtimer::tokio::sleep;
use wasm_timer::Instant;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Launches a game instance in a new WASM thread.
///
/// We drive the app manually instead of using the automatic runner in order to release the thread for use by
/// the instance owner (e.g. a client game when doing local-player on web).
#[derive(Debug)]
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
        let instance_handle = enfync::builtin::wasm::WasmHandle.spawn(
            async move
            {
                // Make game app.
                let Ok(mut app) = game_instance_setup(
                    game_factory,
                    launch_pack,
                    report_sender_clone,
                    command_receiver_clone,
                ) else {
                    let _ = report_sender.send(GameInstanceReport::GameAborted(game_id));
                    return false;
                };

                // Run the loop manually until the app exits.
                let tick_duration = tps_to_duration(world.resource::<GameFwConfig>().ticks_per_sec());

                loop
                {
                    // Tick the app once.
                    let start = Instant::now();
                    app.update();

                    // Check if the app shut down.
                    match app.world().resource::<Events<AppExit>>().get_reader().read().next().copied() {
                        None => (),
                        Some(AppExit::Success) => break,
                        Some(AppExit::Error(code)) => {
                            tracing::warn!("local game instance {game_id} closed with error code {code}");
                            let _ = report_sender.send(GameInstanceReport::GameAborted(game_id));
                            return false;
                        },
                    }

                    // Wait until the next tick is needed.
                    // - We always sleep here even if the duration is zero in order to release the thread so
                    //   a frame can be rendered if necessary.
                    // - Note that this behavior mimics bevy's ScheduleRunnerPlugin loop, which will *not* try
                    //   to 'catch up' if ticks take longer than the target duration. If fixed-update
                    //   behavior is desired then the FixedUpdate schedule should be used.
                    let end = Instant::now();
                    let duration_to_next_update = tick_duration.saturating_sub(
                        end.saturating_duration_since(start)
                    );
                    sleep(duration_to_next_update).await;
                }

                true
            }
        );

        GameInstance::new(game_id, command_sender, command_receiver, instance_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------
