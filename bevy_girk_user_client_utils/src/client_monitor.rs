//local shortcuts
use bevy_girk_client_instance::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::bevy_plugin;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Clear the client monitor when the monitor has a result.
fn cleanup_game_monitor(mut c: Commands, mut monitor: ReactResMut<ClientMonitor>)
{
    // check if the monitor may have a result
    if monitor.is_running() || !monitor.has_client() { return; }

    // try to extract the game over report
    let Ok(result) = monitor.get_noreact().take_result() else { return; };

    // send game over report to the app
    if let Some(report) = result
    {
        tracing::info!("received game over report from client monitor");
        c.react().broadcast(report);
    }

    // clear the monitor
    monitor.get_mut(&mut c).clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Implementation trait for [`ClientMonitor`].
pub(crate) trait ClientMonitorImpl
{
    /// Get the monitored client's game id.
    fn game_id(&self) -> u64;
    /// Check if the game client is running.
    fn is_running(&self) -> bool;
    /// Send a fresh connect token into the monitored client.
    fn send_token(&mut self, token: ServerConnectToken);
    /// Kill the monitored client.
    fn kill(&mut self);
    /// Try to take the result from running the client.
    /// - Returns `Err` if the client is still running.
    /// - Returns `Some(None)` if the client is shut down but there is no report available (either it doesn't exist or
    ///   it was already extracted).
    ///
    /// This is mainly used for extracting game over reports from single-player games.
    fn take_result(&mut self) -> Result<Option<GameOverReport>, ()>;
}

//-------------------------------------------------------------------------------------------------------------------

/// Monitors a running game client.
///
/// `ClientMonitor` is intended as a convenience wrapper around client instances. It supports both local single-player where
/// the client app talks to a locally-managed game app, and multiplayer where the client app talks to a remotely-hosted
/// game app. In all cases, client and game apps are **not** running in the current process. On native targets they are
/// executables launched in separate processes, and on WASM targets they are WASM modules launched in TBD (TODO).
///
/// Clients can be registered with the monitor using [`launch_local_player_client()`] and [`launch_multiplayer_client()`].
///
/// This is a `bevy_kot` reactive resource, which makes it easier to use in a GUI app.
#[derive(ReactResource)]
pub struct ClientMonitor
{
    report_receiver : IoReceiver<ClientInstanceReport>,
    inner           : Option<Box<dyn ClientMonitorImpl + Send + Sync + 'static>>,
}

impl ClientMonitor
{
    /// The id of the monitored client's game (if there is one).
    pub fn game_id(&self) -> Option<u64>
    {
        let Some(inner) = &self.inner else { return None; };
        Some(inner.game_id())
    }

    /// Check if a client is currently monitored.
    pub fn has_client(&self) -> bool
    {
        self.inner.is_some()
    }

    /// Check if the monitored client is still running.
    pub fn is_running(&self) -> bool
    {
        let Some(inner) = &self.inner else { return false; };
        inner.is_running()
    }

    /// Get the next report from the monitored client.
    pub fn next_report(&mut self) -> Option<ClientInstanceReport>
    {
        self.report_receiver.try_recv()
    }

    /// Send a fresh connect token into the monitored client.
    ///
    /// Does nothing if no client is monitored.
    pub fn send_token(&mut self, token: ServerConnectToken)
    {
        let Some(inner) = &mut self.inner else { return; };
        inner.send_token(token)
    }

    /// Kill the monitored client.
    ///
    /// Does nothing if not monitoring a client associated with `game_id`.
    pub fn kill(&mut self, game_id: u64)
    {
        let Some(inner) = &mut self.inner else { return; };
        if inner.game_id() != game_id { return; };
        inner.kill();
    }

    /// Set a new client monitor.
    pub(crate) fn set(&mut self, monitor: impl ClientMonitorImpl + Send + Sync + 'static)
    {
        self.inner = Some(Box::new(monitor));
    }

    /// Replace the client instance report channel.
    ///
    /// The report sender can be moved into your [`ClientMonitorImpl`].
    pub(crate) fn reset_report_channel(&mut self) -> IoSender<ClientInstanceReport>
    {
        let (sender, receiver) = new_io_channel();
        self.report_receiver = receiver;
        sender
    }

    /// Take the result of the running game.
    ///
    /// This is mainly useful for extracting game over reports for local games.
    fn take_result(&mut self) -> Result<Option<GameOverReport>, ()>
    {
        let Some(inner) = &mut self.inner else { return Err(()); };
        inner.take_result()
    }

    /// Clear the monitored client.
    ///
    /// [`Self::take_result()`] will return an error after this has been called.
    fn clear(&mut self)
    {
        self.inner = None;
    }
}

impl Default for ClientMonitor
{
    fn default() -> Self
    {
        let (_, report_receiver) = new_io_channel();
        Self{
            report_receiver,
            inner: None
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Add a [`ClientMonitor`] to your app.
///
/// When a local single-player game has completed and a `GameOverReport` has been extracted,
/// the report will be sent as a `bevy_kot` react event in schedule `First`. The react event can be collected with the
/// `bevy_kot::prelude::ReactEvents<GameOverReport>` event reader.
///
/// Depends on `bevy_kot::prelude::ReactPlugin`.
#[bevy_plugin]
pub fn ClientMonitorPlugin(app: &mut App)
{
    app.insert_react_resource(ClientMonitor::default())
        .add_systems(First, cleanup_game_monitor);
}

//-------------------------------------------------------------------------------------------------------------------
