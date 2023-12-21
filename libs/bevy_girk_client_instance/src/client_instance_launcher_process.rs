//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use clap::Parser;
use enfync::{AdoptOrDefault, Handle};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

//standard shortcuts
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Parser, Debug)]
pub struct ClientInstanceCli
{
    #[arg(short = 'T', value_parser = parse_json::<ServerConnectToken>)]
    token: ServerConnectToken,
    #[arg(short = 'S', value_parser = parse_json::<GameStartInfo>)]
    start_info: GameStartInfo,
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a client instance in a new process on the current machine.
#[derive(Debug)]
pub struct ClientInstanceLauncherProcess
{
    /// Path to the client app binary.
    path: String,
}

impl ClientInstanceLauncherProcess
{
    pub fn new(path: String) -> Self
    {
        Self{ path }
    }
}

impl ClientInstanceLauncherImpl for ClientInstanceLauncherProcess
{
    fn launch(
        &self,
        token            : ServerConnectToken,
        start_info       : GameStartInfo,
        command_receiver : IoReceiver<ClientInstanceCommand>,
        report_sender    : IoSender<ClientInstanceReport>,
    ) -> ClientInstance
    {
        // launch client process
        let game_id = start_info.game_id;
        let Ok(token_ser) = serde_json::to_string(&token)
        else
        {
            tracing::warn!(game_id, "failed serializing game connect token for client instance process");
            return ClientInstance::new(game_id, enfync::PendingResult::make_ready(false));
        };
        let Ok(start_info_ser) = serde_json::to_string(&start_info)
        else
        {
            tracing::warn!(game_id, "failed serializing client start info for client instance process");
            return ClientInstance::new(game_id, enfync::PendingResult::make_ready(false));
        };

        let Ok(mut child_process) = tokio::process::Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(["-T", token_ser.as_str()])
            .args(["-S", start_info_ser.as_str()])
            .spawn()
        else
        {
            tracing::warn!(game_id, "failed spawning client instance process");
            return ClientInstance::new(game_id, enfync::PendingResult::make_ready(false));
        };

        // manage the process
        let (process_handle, _stdout_handle) = manage_child_process(
                game_id,
                child_process,
                command_receiver,
                move |report: ClientInstanceReport| -> Option<bool>
                {
                    match &report
                    {
                        ClientInstanceReport::RequestConnectToken =>
                        {
                            let _ = report_sender.send(report);
                            tracing::trace!(game_id, "client instance process report: request connect token");
                        }
                        ClientInstanceReport::Aborted(_) =>
                        {
                            let _ = report_sender.send(report);
                            tracing::trace!(game_id, "client instance process report: client aborted");
                            return Some(false);
                        }
                    }

                    None
                }
            );

        // return client instance
        // - we monitor the process status instead of the stdout reader for speedier termination
        ClientInstance::new(game_id, process_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a client inside a standalone process.
///
/// Reads [`ClientInstanceCommand`]s from `stdin` and writes [`ClientInstanceReport`]s to `stdout`.
pub fn process_client_launcher(args: ClientInstanceCli, factory: ClientFactory)
{
    // get client launch pack
    let game_id = args.start_info.game_id;
    tracing::info!(game_id, "client instance process started");

    // prepare client app
    let (report_sender, report_receiver) = new_io_channel::<ClientInstanceReport>();
    let (command_sender, command_receiver) = new_io_channel::<ClientInstanceCommand>();

    let Ok(app) = client_instance_setup(
            factory,
            args.token,
            args.start_info,
            command_receiver,
            report_sender,
        ).expect("failed setting up client instance");

    // run the app
    run_app_in_child_process(game_id, app, command_sender, report_receiver, ClientInstanceCommand::Abort);

    tracing::info!(game_id, "client instance process finished");
}

//-------------------------------------------------------------------------------------------------------------------
