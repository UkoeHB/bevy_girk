//local shortcuts
use crate::*;

//third-party shortcuts
use enfync::{AdoptOrDefault, Handle};
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Launch a game instance in a new thread.
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
        launch_pack   : GameLaunchPack,
        report_sender : IoSender<GameInstanceReport>,
    ) -> GameInstance
    {
        // prepare command channel
        let (command_sender, command_receiver) = new_channel::<GameInstanceCommand>();
        let command_receiver_clone = command_receiver.clone();

        // launch game thread
        let game_id = launch_pack.game_id;
        let game_factory = self.game_factory.clone();
        let instance_handle = enfync::builtin::native::CPUHandle::adopt_or_default().spawn(
                    async move
                    {
                        let Ok(mut app) = game_instance_setup(
                                game_factory,
                                launch_pack,
                                report_sender,
                                command_receiver_clone,
                            ) else { return false; };
                        app.run();
                        true
                    }
            );

        GameInstance::new(game_id, command_sender, command_receiver, instance_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------
