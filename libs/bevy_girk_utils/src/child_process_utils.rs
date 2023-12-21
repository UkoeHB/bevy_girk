//local shortcuts

//third-party shortcuts
use bevy_kot_utils::*;
use enfync::{AdoptOrDefault, Handle};
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Manage a child process.
/// - Spawns a tokio task for managing the child process. Items received from `stdin_receiver` will be serialized
///   to JSON and forwarded to the child's `stdin`.
/// - Spawns a tokio task for monitoring the child process `stdout`. Lines received from the child's `stdout`
///   will be deserialized from JSON and passed to the `stdout_handler` callback. If that callback returns `Some`
///   (e.g. on receipt of a 'process aborted' message), then the contained result will be returned from the task.
///
/// Returns handles to the two tasks.
pub fn manage_child_process<I, O>(
    id                 : u64,
    mut child_process  : tokio::process::Child,
    mut stdin_receiver : IoReceiver<I>,
    mut stdout_handler : impl FnMut(O) -> Option<bool> + Send + Sync + 'static,
) -> (enfync::PendingResult<()>, enfync::PendingResult<bool>)
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
    let tokio_spawner = enfync::builtin::native::TokioHandle::adopt_or_default();
    let process_handle = tokio_spawner.spawn(
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
                            return;
                        };
                        if let Err(err) = child_stdin_writer.write(input_ser.as_bytes()).await
                        {
                            tracing::warn!(id, ?err, "failed sending input, aborting");
                            let _ = child_process.kill().await;
                            return;
                        }
                        if let Err(err) = child_stdin_writer.write("\n".as_bytes()).await
                        {
                            tracing::warn!(id, ?err, "failed sending input, aborting");
                            let _ = child_process.kill().await;
                            return;
                        }
                        if let Err(err) = child_stdin_writer.flush().await
                        {
                            tracing::warn!(id, ?err, "failed sending input, aborting");
                            let _ = child_process.kill().await;
                            return;
                        }
                        tracing::trace!(id, ?input, "forwarded input to process");
                    }

                    // await process termination
                    _ = child_process.wait() =>
                    {
                        tracing::trace!(id, "process closed");
                        return;
                    }

                    // catch errors
                    else =>
                    {
                        tracing::warn!(id, "failed unexpectedly, aborting");
                        let _ = child_process.kill().await;
                        return;
                    }
                }
            }
        }
    );

    // monitor process outputs
    let stdout_handle = tokio_spawner.spawn(
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
                        else { tracing::warn!(id, "failed deserializing process output"); return false; };

                        if let Some(result) = (stdout_handler)(output) { return result; }
                    }
                    Err(_) => return false,
                }
            }
        }
    );

    (process_handle, stdout_handle)
}

//-------------------------------------------------------------------------------------------------------------------
