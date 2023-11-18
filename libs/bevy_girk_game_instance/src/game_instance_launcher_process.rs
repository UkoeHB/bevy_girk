//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_kot_utils::*;

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
        _report_sender : IoSender<GameInstanceReport>,
    ) -> GameInstance
    {
        todo!()
        //set tokio runtime to 1 thread in process
    }
}

//-------------------------------------------------------------------------------------------------------------------
