//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use clap::Parser;

//standard shortcuts
use std::fmt::Debug;
use std::process::Stdio;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Parser, Debug)]
pub struct GameInstanceCli
{
    #[arg(short = 'G', value_parser = parse_json::<GameLaunchPack>)]
    pub launch_pack: GameLaunchPack,
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a game instance in a new process on the current machine.
#[derive(Debug)]
pub struct GameInstanceLauncherProcess<S: enfync::Handle + Debug + Send + Sync + 'static>
{
    /// Path to the game app binary.
    path: String,
    /// Spawner for internal async tasks.
    spawner: S,
}

impl<S: enfync::Handle + Debug + Send + Sync + 'static> GameInstanceLauncherProcess<S>
{
    pub fn new(path: String, spawner: S) -> Self
    {
        Self{ path, spawner }
    }
}

impl<S: enfync::Handle + Debug + Send + Sync + 'static> GameInstanceLauncherImpl for GameInstanceLauncherProcess<S>
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

        // launch game process
        let game_id = launch_pack.game_id;
        let Ok(launch_pack_ser) = serde_json::to_string(&launch_pack)
        else
        {
            tracing::warn!(game_id, "failed serializing game launch pack for game instance process");
            return GameInstance::new(game_id, command_sender, command_receiver, enfync::PendingResult::make_ready(false));
        };

        let Ok(child_process) = tokio::process::Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(["-G", &launch_pack_ser])
            .spawn()
        else
        {
            tracing::warn!(game_id, "failed spawning game instance process");
            return GameInstance::new(game_id, command_sender, command_receiver, enfync::PendingResult::make_ready(false));
        };

        // manage the process
        let report_sender_clone = report_sender.clone();
        let (_process_handle, stdout_handle) = manage_child_process(
            &self.spawner,
            game_id,
            child_process,
            command_receiver,
            move |report: GameInstanceReport| -> Option<bool>
            {
                match &report
                {
                    GameInstanceReport::GameStart(_, _) =>
                    {
                        tracing::trace!(game_id, "game instance process report: game start");
                        let _ = report_sender.send(report);
                    }
                    GameInstanceReport::GameOver(_, _) =>
                    {
                        tracing::trace!(game_id, "game instance process report: game over");
                        let _ = report_sender.send(report);
                        return Some(true);
                    }
                    GameInstanceReport::GameAborted(_) =>
                    {
                        tracing::trace!(game_id, "game instance process report: game aborted");
                        let _ = report_sender.send(report);
                        return Some(false);
                    }
                }

                None
            },
            move ||
            {
                tracing::trace!(game_id, "game instance process report: game aborted (killed by critical error)");
                let _ = report_sender_clone.send(GameInstanceReport::GameAborted(game_id));
            }
        );

        // return game instance
        // - we monitor the stdout reader instead of the process status because we want to wait for the game over report
        //   before terminating the instance
        GameInstance::new(game_id, command_sender, command_receiver_clone, stdout_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a game inside a standalone process.
///
/// Reads [`GameInstanceCommand`]s from `stdin` and writes [`GameInstanceReport`]s to `stdout`.
pub fn inprocess_game_launcher(args: GameInstanceCli, game_factory: GameFactory)
{
    // get game launch pack
    let game_id = args.launch_pack.game_id;
    tracing::info!(game_id, "game instance process started");

    // prepare game app
    let (report_sender, report_receiver) = new_io_channel::<GameInstanceReport>();
    let (command_sender, command_receiver) = new_io_channel::<GameInstanceCommand>();

    let app = game_instance_setup(
        game_factory,
        args.launch_pack,
        report_sender.clone(),
        command_receiver,
    ).expect("failed setting up game instance");

    // run the app
    run_app_in_child_process(
        game_id,
        app,
        command_sender.clone(),
        report_receiver,
        move ||
        {
            let _ = command_sender.send(GameInstanceCommand::Abort);
            tracing::error!("child process input failed unexpectedly, aborting game");
        },
        move ||
        {
            let _ = report_sender.send(GameInstanceReport::GameAborted(game_id));
            tracing::error!("critical error in child process, game aborted");
        }
    );

    tracing::info!(game_id, "game instance process finished");
}

//-------------------------------------------------------------------------------------------------------------------
