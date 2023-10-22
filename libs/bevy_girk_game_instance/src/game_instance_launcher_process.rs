//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Launch a game instance in a new process on the current machine.
#[derive(Debug)]
pub struct GameInstanceLauncherProcess
{

}

impl GameInstanceLauncherProcess
{
    pub fn new() -> Self
    {
        todo!()
    }
}

impl GameInstanceLauncherImpl for GameInstanceLauncherProcess
{
    fn launch(
        &self,
        _launch_pack   : GameLaunchPack,
        _report_sender : IOMessageSender<GameInstanceReport>,
    ) -> GameInstance
    {
        todo!()
        //set tokio runtime to 1 thread
    }
}

//-------------------------------------------------------------------------------------------------------------------
