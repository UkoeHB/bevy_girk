//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_client_instance::ClientInstanceCommand;
use bevy_girk_game_instance::GameStartInfo;
use bevy_girk_wiring_common::ServerConnectToken;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Facilitates starting and restarting a game client.
///
/// This is a reactive resource for ease of use in a GUI app.
#[derive(ReactResource, Default)]
pub struct ClientStarter
{
    game_id: u64,
    /// Cached start info for re-use when reconnecting to a hosted game.
    start_info: Option<GameStartInfo>,
}

impl ClientStarter
{
    /// Set the starter.
    ///
    /// This will over-write the existing starter.
    pub fn set(
        &mut self,
        game_id: u64,
        start_info: GameStartInfo
    ){
        self.game_id = game_id;
        self.start_info = Some(start_info);
    }

    /// Check if there is a starter
    pub fn has_starter(&self) -> bool
    {
        self.start_info.is_some()
    }

    /// Gets the game id for the client that can be started.
    ///
    /// Returns `None` if [`Self::has_starter()`] is false.
    pub fn game_id(&self) -> Option<u64>
    {
        if !self.has_starter() { return None; }
        Some(self.game_id)
    }

    /// Tries to send [`ClientInstanceCommand::Start`].
    ///
    /// Returns an error if there is no registered starter.
    pub fn start(&self, c: &mut Commands, token: ServerConnectToken) -> Result<(), ()>
    {
        let Some(start_info) = &self.start_info else { return Err(()); };
        c.queue(ClientInstanceCommand::Start(token, start_info.clone()));
        Ok(())
    }

    /// Clear the starter if it matches the given game id.
    pub fn clear(&mut self, game_id: u64)
    {
        if self.game_id != game_id { return; }
        self.start_info = None;
    }

    /// Clears the starter regardless of the current game id.
    ///
    /// This is useful as a fall-back when [`Self::clear()`] is not possible because the game id is unknown. For example,
    /// if your user client becomes disconnected from the host server and you expect to *maybe* receive a new
    /// game-start package when you reconnect.
    pub fn force_clear(&mut self)
    {
        self.start_info = None;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Add a [`ClientStarter`] to your app.
pub struct ClientStarterPlugin;

impl Plugin for ClientStarterPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<ClientStarter>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
