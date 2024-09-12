//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_girk_utils::*;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for types that launch [`GameInstances`](GameInstance).
///
/// Note that all launchers should send a [`GameInstanceReport::Aborted`] on any error that causes `false`
/// to be returned by the game instance. This reduces the need for users to poll game instances for results.
pub trait GameInstanceLauncherImpl: Debug + Send + Sync + 'static
{
    /// Launches a game and returns a [`GameInstance`] for monitoring it.
    ///
    /// The `report_sender` can potentially be tied to a centralized `report_receiver` that collects reports
    /// from many game instances.
    fn launch(
        &self,
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
        launch_pack: GameLaunchPack,
        report_sender: IoSender<GameInstanceReport>,
    ) -> GameInstance
    {
        self.launcher.launch(launch_pack, report_sender)
    }
}

//-------------------------------------------------------------------------------------------------------------------
