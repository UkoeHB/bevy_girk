//local shortcuts
use crate::*;

//third-party shortcuts
use enfync::{AdoptOrDefault, Handle};
use bevy_girk_utils::*;
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
        memory_transport: bool,
        launch_pack: GameLaunchPack,
        report_sender: IoSender<GameInstanceReport>,
    ) -> GameInstance
    {
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
                        memory_transport,
                        report_sender_clone,
                        command_receiver_clone,
                    ) else { return false; };

                    // Run the loop manually until the app exits.
                    let tick_duration = tps_to_duration(world.resource::<GameFwConfig>().ticks_per_sec());
                    let instant = Instant::now();
                    let mut tick_count = 0u64;

                    loop
                    {
                        // Tick the app once.
                        app.update();
                        tick_count += 1;

                        // Check if the app shut down.
                        match app.world().resource::<Events<AppExit>>().get_reader().read().next().copied() {
                            None => (),
                            Some(AppExit::Success) => break,
                            Some(AppExit::Error(code)) => {
                                tracing::warn!("local game instance {game_id} closed with error code {code}");
                                return false;
                            },
                        }

                        // Wait until the next tick is needed.
                        // - We always sleep here even if the duration is zero in order to release the thread so
                        //   a frame can be rendered if necessary.
                        let elapsed = instant.elapsed();
                        let duration_to_next_update = (tick_count*tick_duration).saturating_sub(elapsed);
                        sleep(duration_to_next_update).await;
                    }

                    true
                }
            );

        GameInstance::new(game_id, command_sender, command_receiver, instance_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------
