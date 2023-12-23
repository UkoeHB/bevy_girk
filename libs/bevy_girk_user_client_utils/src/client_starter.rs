//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_fn_plugin::bevy_plugin;
use bevy_girk_utils::*;
use bevy_kot_derive::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Facilitates starting and restarting a game client.
///
/// This is a `bevy_kot` reactive resource for ease of use in a GUI app.
#[derive(ReactResource)]
pub struct ClientStarter
{
    game_id       : u64,
    game_location : GameLocation,
    starter       : Option<Box<dyn FnMut(&mut ClientMonitor, ServerConnectToken) + Send + Sync + 'static>>,
}

impl ClientStarter
{
    /// Set the starter.
    ///
    /// This will over-write the existing starter.
    pub fn set(
        &mut self,
        game_id       : u64,
        game_location : GameLocation,
        starter       : impl FnMut(&mut ClientMonitor, ServerConnectToken) + Send + Sync + 'static
    ){
        self.game_id       = game_id;
        self.game_location = game_location;
        self.starter       = Some(Box::new(starter));
    }

    /// Check if there is a starter
    pub fn has_starter(&self) -> bool
    {
        self.starter.is_some()
    }

    /// Get the game id for the client that can be started.
    ///
    /// Returns `None` if [`Self::has_starter()`] is false.
    pub fn game_id(&self) -> Option<u64>
    {
        if !self.has_starter() { return None; }
        Some(self.game_id)
    }

    /// Try to start a client.
    ///
    /// The new client will be monitored by `monitor`.
    ///
    /// Returns an error if there is no registered starter.
    pub fn start(&mut self, monitor: &mut ClientMonitor, token: ServerConnectToken) -> Result<(), ()>
    {
        let Some(starter) = &mut self.starter else { return Err(()); };
        (starter)(monitor, token);
        Ok(())
    }

    /// Clear the starter if it matches the given game id.
    pub fn clear(&mut self, game_id: u64)
    {
        if self.game_id != game_id { return; }
        self.starter = None;
    }

    /// Clear the starter if it matches `game_location`.
    ///
    /// This is useful as a fall-back when [`Self::clear()`] is not possible because the game id is unknown. For example,
    /// if your user client becomes disconnected you can clear the starter if the game location is
    /// [`GameLocation::Hosted`], but leave it if the game location is [`GameLocation::Local`].
    pub fn force_clear_if(&mut self, game_location: GameLocation)
    {
        if self.game_location != game_location { return; }
        self.starter = None;
    }
}

impl Default for ClientStarter
{
    fn default() -> Self
    {
        Self{ game_id: 0u64, game_location: GameLocation::Local, starter: None }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Add a [`ClientStarter`] to your app.
#[bevy_plugin]
pub fn ClientStarterPlugin(app: &mut App)
{
    app.insert_react_resource(ClientStarter::default());
}

//-------------------------------------------------------------------------------------------------------------------
