//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use enfync::{AdoptOrDefault, Handle};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

//standard shortcuts
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Arg tag for passing a game launch pack into a game app subprocess.
const GAME_LAUNCH_PACK_TAG: &'static str = "-glp";

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_game_launch_pack(args: &mut std::env::Args) -> Result<GameLaunchPack, ()>
{
    // find launch pack tag
    loop
    {
        match args.next()
        {
            Some(arg) => if arg == GAME_LAUNCH_PACK_TAG { break; },
            None => return Err(()),
        }
    }

    // extract launch pack
    let Some(arg) = args.next() else { return Err(()); };
    let launch_pack = serde_json::de::from_str::<GameLaunchPack>(&arg).map_err(|_| ())?;

    Ok(launch_pack)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Forward game instance reports from the app to stdout.
fn drain_game_instance_reports(report_receiver: &mut IoReceiver<GameInstanceReport>)
{
    while let Some(report) = report_receiver.try_recv()
    {
        let report_ser = serde_json::to_string(&report).expect("failed serializing game instance report");
        let _ = std::io::stdout().write(report_ser.as_bytes());
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
        let (command_sender, mut command_receiver) = new_io_channel::<GameInstanceCommand>();
        let command_receiver_clone = command_receiver.clone();

        // launch game process
        let game_id = launch_pack.game_id;
        let Ok(launch_pack_ser) = serde_json::to_string(&launch_pack)
        else
        {
            tracing::warn!(game_id, "failed serializing game launch pack for game instance process");
            return GameInstance::new(game_id, command_sender, command_receiver, enfync::PendingResult::make_ready(false));
        };

        let Ok(mut child_process) = tokio::process::Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args([GAME_LAUNCH_PACK_TAG, &launch_pack_ser])
            .spawn()
        else
        {
            tracing::warn!(game_id, "failed spawning game instance process");
            return GameInstance::new(game_id, command_sender, command_receiver, enfync::PendingResult::make_ready(false));
        };

        // extract child process io
        let child_stdin = child_process.stdin.take().unwrap();
        let child_stdout = child_process.stdout.take().unwrap();
        let mut child_stdin_writer = tokio::io::BufWriter::new(child_stdin);
        let mut child_stdout_reader = tokio::io::BufReader::new(child_stdout);

        // manage game instance process
        let tokio_spawner = enfync::builtin::native::TokioHandle::adopt_or_default();
        tokio_spawner.spawn(
            async move
            {
                loop
                {
                    tokio::select!
                    {
                        // forward commands to the process
                        Some(command) = command_receiver.recv() =>
                        {
                            let Ok(command_ser) = serde_json::to_string(&command)
                            else
                            {
                                tracing::warn!(game_id, "game process monitor failed serializing command, aborting");
                                let _ = child_process.kill().await;
                                return;
                            };
                            if let Err(err) = child_stdin_writer.write(command_ser.as_bytes()).await
                            {
                                tracing::warn!(game_id, ?err, "game process monitor failed sending command, aborting");
                                let _ = child_process.kill().await;
                                return;
                            }
                            tracing::trace!(game_id, ?command, "forwarded command to game instance process");
                        }

                        // await process termination
                        _ = child_process.wait() =>
                        {
                            tracing::trace!(game_id, "game instance process closed");
                            return;
                        }

                        // catch errors
                        else =>
                        {
                            tracing::warn!(game_id, "game process monitor failed unexpectedly, aborting");
                            let _ = child_process.kill().await;
                            return;
                        }
                    }
                }
            }
        );

        // monitor process outputs
        let handle = tokio_spawner.spawn(
            async move
            {
                let mut buf = String::default();

                loop
                {
                    buf.clear();
                    match child_stdout_reader.read_line(&mut buf).await
                    {
                        Ok(_) =>
                        {
                            let Ok(report) = serde_json::de::from_str::<GameInstanceReport>(&buf)
                            else { tracing::warn!(game_id, "failed deserializing game instance report"); return false; };

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
                                    return true;
                                }
                                GameInstanceReport::GameAborted(_) =>
                                {
                                    let _ = report_sender.send(report);
                                    tracing::trace!(game_id, "game instance process report: game aborted");
                                    return false;
                                }
                            }
                        }
                        Err(_) => return false,
                    }
                }
            }
        );

        // return game instance
        // - we monitor the stdout reader instead of the process status because we want to wait for the game over report
        //   before terminating the instance
        GameInstance::new(game_id, command_sender, command_receiver_clone, handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a game inside a standalone process.
///
/// Assumes the next useful `std::env::Args` item is the game launch pack.
///
/// Reads [`GameInstanceCommand`]s from `stdin` and writes [`GameInstanceReport`]s to `stdout`.
pub fn process_game_launcher(args: &mut std::env::Args, game_factory: GameFactory)
{
    // get game launch pack
    let launch_pack = get_game_launch_pack(args).expect("failed getting game launch pack from args");
    let game_id = launch_pack.game_id;
    tracing::trace!(game_id, "game instance process start");

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
                line.clear();
                let _ = stdin_reader.read_line(&mut line);

                if line.is_empty() { return; }

                let command = serde_json::de::from_str::<GameInstanceCommand>(&line).expect("failed deserializing command");
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

    tracing::trace!(game_id, "game instance process finished");
}

//-------------------------------------------------------------------------------------------------------------------
