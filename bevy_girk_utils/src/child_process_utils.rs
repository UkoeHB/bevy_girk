//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

//standard shortcuts
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Write};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Forward process outputs from the app to `stdout`.
fn drain_outputs<O: Serialize>(output_receiver: &mut IoReceiver<O>)
{
    while let Some(output) = output_receiver.try_recv()
    {
        let output_ser = serde_json::to_string(&output).expect("failed serializing process output");
        let _ = std::io::stdout().write(output_ser.as_bytes());
        let _ = std::io::stdout().write("\n".as_bytes());
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn monitor_for_outputs<O: Serialize + Send + Sync + 'static>(mut output_receiver: ResMut<IoReceiver<O>>)
{
    drain_outputs(&mut output_receiver);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Manage a child process.
/// - Spawns an enfync task for managing the child process. Items received from `stdin_receiver` will be serialized
///   to JSON and forwarded to the child's `stdin`.
/// - Spawns an enfync task for monitoring the child process's `stdout`. Lines received from the child's `stdout`
///   will be deserialized from JSON and passed to the `stdout_handler` callback. If that callback returns `Some`
///   (e.g. on receipt of a 'process aborted' message), then the contained result will be returned from the task. The
///   possible results are true/false to indicate if the task closed 'normally'.
///
/// Returns handles to the two tasks.
///
/// This is designed for compatibility with [`run_app_in_child_process()`].
pub fn manage_child_process<I, O>(
    spawner             : &impl enfync::Handle,
    id                  : u64,
    mut child_process   : tokio::process::Child,
    mut stdin_receiver  : IoReceiver<I>,
    mut stdout_handler  : impl FnMut(O) -> Option<bool> + Send + Sync + 'static,
    mut on_stdout_error : impl FnMut() + Send + Sync + 'static,
) -> (enfync::PendingResult<bool>, enfync::PendingResult<bool>)
where
    I: Debug + Serialize + Send + Sync + 'static,
    O: for<'de> Deserialize<'de> + Send + Sync + 'static
{
    // extract child process io
    let child_stdin = child_process.stdin.take().unwrap();
    let child_stdout = child_process.stdout.take().unwrap();
    let mut child_stdin_writer = tokio::io::BufWriter::new(child_stdin);
    let mut child_stdout_reader = tokio::io::BufReader::new(child_stdout);

    // manage process
    let process_handle = spawner.spawn(
        async move
        {
            loop
            {
                tokio::select!
                {
                    // forward inputs to the process
                    Some(input) = stdin_receiver.recv() =>
                    {
                        let Ok(input_ser) = serde_json::to_string(&input)
                        else
                        {
                            tracing::warn!(id, "failed serializing input, aborting");
                            let _ = child_process.kill().await;
                            return false;
                        };
                        if let Err(err) = child_stdin_writer.write(input_ser.as_bytes()).await
                        {
                            tracing::warn!(id, ?err, "failed sending input, aborting");
                            let _ = child_process.kill().await;
                            return false;
                        }
                        if let Err(err) = child_stdin_writer.write("\n".as_bytes()).await
                        {
                            tracing::warn!(id, ?err, "failed sending input, aborting");
                            let _ = child_process.kill().await;
                            return false;
                        }
                        if let Err(err) = child_stdin_writer.flush().await
                        {
                            tracing::warn!(id, ?err, "failed sending input, aborting");
                            let _ = child_process.kill().await;
                            return false;
                        }
                        tracing::trace!(id, ?input, "forwarded input to process");
                    }

                    // await process termination
                    _ = child_process.wait() =>
                    {
                        tracing::trace!(id, "process closed");
                        return true;
                    }

                    // catch errors
                    else =>
                    {
                        tracing::warn!(id, "failed unexpectedly, aborting");
                        let _ = child_process.kill().await;
                        return false;
                    }
                }
            }
        }
    );

    // monitor process outputs
    let stdout_handle = spawner.spawn(
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
                        let Ok(output) = serde_json::de::from_str::<O>(&buf)
                        else {
                            tracing::warn!(id, ?buf, "failed deserializing process output");
                            (on_stdout_error)();
                            return false;
                        };

                        if let Some(result) = (stdout_handler)(output)
                        {
                            return result;
                        }
                    }
                    Err(_) =>
                    {
                        (on_stdout_error)();
                        return false;
                    }
                }
            }
        }
    );

    (process_handle, stdout_handle)
}

//-------------------------------------------------------------------------------------------------------------------

/// Run an app in a child process.
/// - Reads `I` messages from `stdin` (deserialized from JSON) and forwards them to the app via `stdin_sender`. If
///   the `stdin` handle closes, then `on_stdin_null` will be invoked. This facilitates graceful
///   handling of parent process closure, although graceful shutdown is not guaranteed on all machines.
/// - Reads `O` messages from `stdout_receiver`, serializes them to JSON, and forwards them to the process's `stdout`.
///
/// This is designed for compatibility with [`manage_child_process()`].
pub fn run_app_in_child_process<I, O>(
    id                  : u64,
    mut app             : App,
    stdin_sender        : IoSender<I>,
    mut stdout_receiver : IoReceiver<O>,
    on_input_failure    : impl FnOnce() + Send + Sync + 'static,
    on_critical_err     : impl FnOnce() + Send + Sync + 'static,
)
where
    I: Debug + for<'de> Deserialize<'de> + Send + Sync + 'static,
    O: Clone + Serialize + Send + Sync + 'static
{
    // spawn thread for monitoring inputs
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
                    (on_input_failure)();
                    return;
                }

                // deserialize input
                let input = serde_json::de::from_str::<I>(&line).expect("failed deserializing input");
                tracing::info!(id, ?input, "received process input");

                // forward to app
                if stdin_sender.send(input).is_err() { break; }
            }
        }
    );

    // add system for marshalling outputs to the parent process
    app.insert_resource(stdout_receiver.clone())
        .add_systems(Last, monitor_for_outputs::<O>);

    // run the app to completion
    if let Err(_) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || { app.run(); }))
    {
        (on_critical_err)();
    }

    // drain any lingering outputs
    drain_outputs(&mut stdout_receiver);
}

//-------------------------------------------------------------------------------------------------------------------
