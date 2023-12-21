//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_kot_utils::*;
use clap::Parser;

//standard shortcuts
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
pub struct GameInstanceLauncherProcess
{
    /// Path to the game app binary.
    path: String,
}

impl GameInstanceLauncherProcess
{
    pub fn new(path: String) -> Self
    {
        Self{ path }
    }
}

impl GameInstanceLauncherImpl for GameInstanceLauncherProcess
{
    fn launch(
        &self,
        launch_pack   : GameLaunchPack,
        report_sender : IoSender<GameInstanceReport>,
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
        let (_process_handle, stdout_handle) = manage_child_process(
                game_id,
                child_process,
                command_receiver,
                move |report: GameInstanceReport| -> Option<bool>
                {
                    match &report
                    {
                        GameInstanceReport::GameStart(_, _) =>
                        {
                            let _ = report_sender.send(report);
                            tracing::trace!(game_id, "game instance process report: game start");
                        }
                        GameInstanceReport::GameOver(_, _) =>
                        {
                            let _ = report_sender.send(report);
                            tracing::trace!(game_id, "game instance process report: game over");
                            return Some(true);
                        }
                        GameInstanceReport::GameAborted(_) =>
                        {
                            let _ = report_sender.send(report);
                            tracing::trace!(game_id, "game instance process report: game aborted");
                            return Some(false);
                        }
                    }

                    None
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
pub fn process_game_launcher(args: GameInstanceCli, game_factory: GameFactory)
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
            report_sender,
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
                tracing::error!("stdin received null unexpectedly, aborting game");
            }
        );

    tracing::info!(game_id, "game instance process finished");
}

//-------------------------------------------------------------------------------------------------------------------
