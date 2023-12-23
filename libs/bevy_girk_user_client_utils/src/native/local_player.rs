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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_systime() -> Duration
{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Client monitor for local single-player games on native targets.
pub struct ClientMonitorLocalNative
{
    task           : enfync::PendingResult<Option<GameOverReport>>,
    command_sender : IoSender<ClientInstanceCommand>,
}

//-------------------------------------------------------------------------------------------------------------------

impl ClientMonitorImpl for ClientMonitorLocalNative
{
    fn game_id(&self) -> u64
    {
        u64::MAX
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
        self.task.try_extract()
            .unwrap_or_else(|| Ok(None))
            .or_else(|_| Ok(None))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Config for launching a local single-player client on native targets.
#[derive(Debug, Clone)]
pub struct LocalPlayerLauncherConfigNative<S: HandleReqs>
{
    /// Getter for task spawner.
    pub spawner_fn: Arc<dyn TaskSpawnerGetterFn<S>>,
    /// Path to the game instance binary.
    pub game_instance_path: String,
    /// Path to the client instance binary.
    pub client_instance_path: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a local single-player client on a native target.
pub(crate) fn launch_local_player_client_native<S: HandleReqs>(
    config               : LocalPlayerLauncherConfigNative<S>,
    launch_pack          : GameLaunchPack,
    client_report_sender : IoSender<ClientInstanceReport>,
) -> ClientMonitorLocalNative
{
    // launch in task
    let spawner = (config.spawner_fn)();
    let spawner_clone = spawner.clone();
    let (client_command_sender, client_command_receiver) = new_io_channel::<ClientInstanceCommand>();
    let task = spawner.spawn(
        async move
        {
            // launch game
            tracing::trace!("launching game instance for local player game");
            let (game_report_sender, mut game_report_receiver) = new_io_channel::<GameInstanceReport>();
            let game_launcher = GameInstanceLauncherProcess::new(config.game_instance_path, spawner_clone.clone());
            let mut game_instance = game_launcher.launch(launch_pack, game_report_sender);

            // wait for game start report
            let Some(GameInstanceReport::GameStart(_, mut report)) = game_report_receiver.recv().await
            else { tracing::error!("failed getting game start report for local player game"); return None; };

            // prepare to launch the client
            let Some(meta) = &report.native_meta
            else { tracing::error!("missing native meta for setting up local player renet client"); return None; };

            let Some(start_info) = report.start_infos.pop()
            else { tracing::error!("missing start info for local player game"); return None; };

            let Ok(token) = new_connect_token_native(meta, get_systime(), start_info.client_id)
            else { tracing::error!("failed producing connect token for local player game"); return None; };

            // launch game client
            tracing::trace!("launching game client for local player game");
            let client_launcher = ClientInstanceLauncherProcess::new(config.client_instance_path, spawner_clone);
            let mut client_instance = client_launcher.launch(
                    token,
                    start_info,
                    client_command_receiver,
                    client_report_sender
                );

            // wait for client to close
            // - we must wait for client closure to avoid zombie process leak
            if !client_instance.get().await
            { tracing::warn!("local player client instance closed with error"); }

            // command game instance to abort
            // - we assume if the client is closed then the game should die, since this is singleplayer
            // - this will do nothing if the game instance already closed
            let _ = game_instance.send_command(GameInstanceCommand::Abort);

            // wait for game instance to close
            if !game_instance.get().await
            { tracing::warn!("local player game instance closed with error"); }

            // get game instance report
            let Some(GameInstanceReport::GameOver(_, game_over_report)) = game_report_receiver.recv().await
            else { tracing::error!("did not receive game over report for local player game"); return None; };

            tracing::info!("local player game ended");
            Some(game_over_report)
        }
    );

    ClientMonitorLocalNative{ task, command_sender: client_command_sender  }
}

//-------------------------------------------------------------------------------------------------------------------
