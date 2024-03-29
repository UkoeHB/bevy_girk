//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for [`ClientInstanceLauncher`].
pub trait ClientInstanceLauncherImpl: Debug + Send + Sync
{
    fn launch(
        &self,
        token                   : ServerConnectToken,
        start_info              : GameStartInfo,
        config                  : ClientInstanceConfig,
        client_command_receiver : IoReceiver<ClientInstanceCommand>,
        client_report_sender    : IoSender<ClientInstanceReport>,
    ) -> ClientInstance;
}

//-------------------------------------------------------------------------------------------------------------------

/// Launches client instances.
///
/// The launcher implementation is type-erased. This is useful if you want to plug-and-play different launchers without
/// modifying the code that owns the launcher.
#[derive(Debug)]
pub struct ClientInstanceLauncher
{
    launcher: Box<dyn ClientInstanceLauncherImpl>,
}

impl ClientInstanceLauncher
{
    /// Make a new launcher.
    pub fn new<L: ClientInstanceLauncherImpl + 'static>(launcher: L) -> Self
    {
        Self{ launcher: Box::new(launcher) }
    }

    /// Launch a client instance using the internal launcher implementation.
    pub fn launch(
        &self,
        token                   : ServerConnectToken,
        start_info              : GameStartInfo,
        config                  : ClientInstanceConfig,
        client_command_receiver : IoReceiver<ClientInstanceCommand>,
        client_report_sender    : IoSender<ClientInstanceReport>,
    ) -> ClientInstance
    {
        self.launcher.launch(token, start_info, config, client_command_receiver, client_report_sender)
    }
}

//-------------------------------------------------------------------------------------------------------------------
