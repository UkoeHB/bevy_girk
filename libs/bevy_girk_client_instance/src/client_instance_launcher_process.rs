//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy_kot_utils::*;
use clap::Parser;

//standard shortcuts
use std::fmt::Debug;
use std::process::Stdio;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// CLI parameters for launching a client instance in a child process.
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
pub struct ClientInstanceLauncherProcess<S: enfync::Handle + Debug + Send + Sync + 'static>
{
    /// Path to the game app binary.
    path: String,
    /// Spawner for internal async tasks.
    spawner: Arc<S>,
}

impl<S: enfync::Handle + Debug + Send + Sync + 'static> ClientInstanceLauncherProcess<S>
{
    pub fn new(path: String, spawner: Arc<S>) -> Self
    {
        Self{ path, spawner }
    }
}

impl<S: enfync::Handle + Debug + Send + Sync + 'static> ClientInstanceLauncherImpl for ClientInstanceLauncherProcess<S>
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

        let Ok(child_process) = tokio::process::Command::new(&self.path)
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
                &*self.spawner,
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
pub fn inprocess_client_launcher(args: ClientInstanceCli, factory: &mut ClientFactory)
{
    // get client launch pack
    let game_id = args.start_info.game_id;
    tracing::info!(game_id, "client instance process started");

    // prepare client app
    let (report_sender, report_receiver) = new_io_channel::<ClientInstanceReport>();
    let (command_sender, command_receiver) = new_io_channel::<ClientInstanceCommand>();

    let app = client_instance_setup(
            factory,
            args.token,
            args.start_info,
            report_sender,
            command_receiver,
        ).expect("failed setting up client instance");

    // run the app
    run_app_in_child_process(
            game_id,
            app,
            command_sender.clone(),
            report_receiver,
            move ||
            {
                let _ = command_sender.send(ClientInstanceCommand::Abort);
                tracing::error!("stdin received null unexpectedly, aborting client");
            }
        );

    tracing::info!(game_id, "client instance process finished");
}

//-------------------------------------------------------------------------------------------------------------------
