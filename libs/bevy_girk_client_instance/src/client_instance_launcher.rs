//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_kot_utils::*;

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

pub trait ClientInstanceLauncherImpl: Debug + Send + Sync
{
    fn launch(
        &self,
        token                   : ServerConnectToken,
        start_info              : GameStartInfo,
        client_command_receiver : IoReceiver<ClientInstanceCommand>,
        client_report_sender    : IoSender<ClientInstanceReport>,
    ) -> ClientInstance;
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ClientInstanceLauncher
{
    launcher: Box<dyn ClientInstanceLauncherImpl>,
}

impl ClientInstanceLauncher
{
    pub fn new<L: ClientInstanceLauncherImpl + 'static>(launcher: L) -> Self
    {
        Self{ launcher: Box::new(launcher) }
    }

    pub fn launch(
        &self,
        token                   : ServerConnectToken,
        start_info              : GameStartInfo,
        client_command_receiver : IoReceiver<ClientInstanceCommand>,
        client_report_sender    : IoSender<ClientInstanceReport>,
    ) -> ClientInstance
    {
        self.launcher.launch(token, start_info, client_command_receiver, client_report_sender)
    }
}

//-------------------------------------------------------------------------------------------------------------------
