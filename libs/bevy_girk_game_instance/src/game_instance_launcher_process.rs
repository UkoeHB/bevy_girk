//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use clap::Parser;

//standard shortcuts
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Forward game instance reports from the app to stdout.
fn drain_game_instance_reports(report_receiver: &mut IoReceiver<GameInstanceReport>)
{
    while let Some(report) = report_receiver.try_recv()
    {
        let report_ser = serde_json::to_string(&report).expect("failed serializing game instance report");
        let _ = std::io::stdout().write(report_ser.as_bytes());
        let _ = std::io::stdout().write("\n".as_bytes());
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn monitor_for_game_instance_reports(mut report_receiver: ResMut<IoReceiver<GameInstanceReport>>)
{
    drain_game_instance_reports(&mut report_receiver);
}

//-------------------------------------------------------------------------------------------------------------------
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
    let launch_pack = args.launch_pack;
    let game_id = launch_pack.game_id;
    tracing::info!(game_id, "game instance process started");

    // prepare game app
    let (report_sender, mut report_receiver) = new_io_channel::<GameInstanceReport>();
    let (command_sender, command_receiver) = new_io_channel::<GameInstanceCommand>();

    let mut app = game_instance_setup(
            game_factory,
            launch_pack,
            report_sender,
            command_receiver,
        ).expect("failed setting up game instance");

    // spawn thread for monitoring input commands
    std::thread::spawn(
        move ||
        {
            let mut stdin_reader = BufReader::new(std::io::stdin());
            let mut line = String::new();

            loop
            {
                // read the next stdin
                line.clear();
                let _ = stdin_reader.read_line(&mut line);

                if line.is_empty()
                {
                    let _ = command_sender.send(GameInstanceCommand::Abort);
                    tracing::error!(game_id, "received null value at stdin, aborting game");
                    return;
                }

                // deserialize command
                let command = serde_json::de::from_str::<GameInstanceCommand>(&line).expect("failed deserializing command");
                tracing::info!(game_id, ?command, "received game instance command");

                // forward to app
                if command_sender.send(command).is_err() { break; }
            }
        }
    );

    // add system for marshalling game instance reports to the parent process
    app.insert_resource(report_receiver.clone())
        .add_systems(Last, monitor_for_game_instance_reports);

    // run the app to completion
    app.run();

    // drain any lingering game instance reports
    drain_game_instance_reports(&mut report_receiver);

    tracing::info!(game_id, "game instance process finished");
}

//-------------------------------------------------------------------------------------------------------------------
