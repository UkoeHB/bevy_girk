//local shortcuts
use crate::*;
use bevy_girk_client_instance::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_kot_utils::*;

//standard shortcuts
use std::fmt::Debug;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Client monitor for multiplayer games on native targets.
pub struct ClientMonitorMultiplayerNative
{
    game_id        : u64,
    task           : enfync::PendingResult<()>,
    command_sender : IoSender<ClientInstanceCommand>,
}

//-------------------------------------------------------------------------------------------------------------------

impl ClientMonitorImpl for ClientMonitorMultiplayerNative
{
    fn game_id(&self) -> u64
    {
        self.game_id
    }

    fn is_running(&self) -> bool
    {
        !self.task.done()
    }

    fn send_token(&mut self, token: ServerConnectToken)
    {
        let _ = self.command_sender.send(ClientInstanceCommand::Connect(token));
    }

    fn kill(&mut self)
    {
        let _ = self.command_sender.send(ClientInstanceCommand::Abort);
    }

    fn take_result(&mut self) -> Result<Option<GameOverReport>, ()>
    {
        if self.is_running() { return Err(()); }
        Ok(None)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Config for launching a multiplayer client on native targets.
#[derive(Debug, Clone)]
pub struct MultiPlayerLauncherConfigNative<S: HandleReqs>
{
    /// Getter for task spawner.
    pub spawner_fn: Arc<dyn TaskSpawnerGetterFn<S>>,
    /// Path to the client instance binary.
    pub client_instance_path: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// Launches a multi-player client on a native target.
pub fn launch_multiplayer_client_native<S: HandleReqs>(
    config               : MultiPlayerLauncherConfigNative<S>,
    token                : ServerConnectToken,
    start_info           : GameStartInfo,
    client_report_sender : IoSender<ClientInstanceReport>,
) -> ClientMonitorMultiplayerNative
{
    // launch in task
    let game_id = start_info.game_id;
    let spawner = (config.spawner_fn)();
    let spawner_clone = spawner.clone();
    let (command_sender, command_receiver) = new_io_channel::<ClientInstanceCommand>();
    let task = spawner.spawn(
        async move
        {
            // launch game client
            tracing::trace!("launching game client for multiplayer game");
            let client_launcher = ClientInstanceLauncherProcess::new(config.client_instance_path, spawner_clone);
            let mut client_instance = client_launcher.launch(
                    token,
                    start_info,
                    command_receiver,
                    client_report_sender
                );

            // wait for client to close
            // - we must wait for client closure to avoid zombie process leak
            if !client_instance.get().await
            { tracing::warn!("multiplayer client instance closed with error"); }

            tracing::info!("multiplayer game ended");
        }
    );

    ClientMonitorMultiplayerNative{ game_id, task, command_sender }
}

//-------------------------------------------------------------------------------------------------------------------
