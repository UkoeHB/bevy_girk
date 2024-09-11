//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_girk_utils::*;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

pub trait GameInstanceLauncherImpl: Debug + Send + Sync + 'static
{
    fn launch(
        &self,
        memory_transport: bool,
        launch_pack: GameLaunchPack,
        report_sender: IoSender<GameInstanceReport>,
    ) -> GameInstance;
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct GameInstanceLauncher
{
    launcher: Box<dyn GameInstanceLauncherImpl>,
}

impl GameInstanceLauncher
{
    pub fn new<L: GameInstanceLauncherImpl>(launcher: L) -> Self
    {
        Self{ launcher: Box::new(launcher) }
    }

    pub fn launch(
        &self,
        memory_transport: bool,
        launch_pack: GameLaunchPack,
        report_sender: IoSender<GameInstanceReport>,
    ) -> GameInstance
    {
        self.launcher.launch(memory_transport, launch_pack, report_sender)
    }
}

//-------------------------------------------------------------------------------------------------------------------
