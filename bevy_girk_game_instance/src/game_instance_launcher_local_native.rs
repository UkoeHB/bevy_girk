//local shortcuts
use crate::{game_instance_setup, GameFactory, GameInstance, GameInstanceCommand, GameInstanceLauncherImpl, GameInstanceReport, GameLaunchPack};

//third-party shortcuts
use enfync::{AdoptOrDefault, Handle};
use bevy_girk_utils::{new_io_channel, IoSender};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Launch a game instance in a new thread.
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
        // prepare command channel
        let (command_sender, command_receiver) = new_io_channel::<GameInstanceCommand>();
        let command_receiver_clone = command_receiver.clone();

        // launch game thread
        let game_id = launch_pack.game_id;
        let game_factory = self.game_factory.clone();
        let instance_handle = enfync::builtin::native::CPUHandle::adopt_or_default().spawn(
            async move
            {
                let report_sender_clone = report_sender.clone();
                let Ok(result) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                        move ||
                        {
                            let report_sender_clone2 = report_sender_clone.clone();
                            let mut app = match game_instance_setup(
                                game_factory,
                                launch_pack,
                                report_sender_clone,
                                command_receiver_clone,
                            ) {
                                Ok(app) => app,
                                Err(err) => {
                                    let _ = report_sender_clone2.send(GameInstanceReport::GameAborted(game_id, err));
                                    return false;
                                }
                            };
                            app.run();
                            true
                        }
                    ))
                else {
                    let _ = report_sender.send(GameInstanceReport::GameAborted(game_id, "unexpected panic in local game instance".into()));
                    return false;
                };
                result
            }
        );

        GameInstance::new(game_id, command_sender, command_receiver, instance_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------
