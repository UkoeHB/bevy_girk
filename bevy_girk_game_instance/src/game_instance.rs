//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A game instance is a wrapper around a running game.
///
/// The wrapper monitors the game for completion and sends commands and reports in and out of the game.
#[derive(Debug)]
pub struct GameInstance
{
    /// the game instance's id
    game_id: u64,
    /// command sender for the game instance (passes commands into the game instance)
    command_sender: IoSender<GameInstanceCommand>,
    /// command receiver; cached so the channel will not return errors when the game instance shuts down
    _command_receiver: IoReceiver<GameInstanceCommand>,
    /// join handle for the game instance (used to detect status of the instance)
    instance_handle: enfync::PendingResult<bool>,
    /// cached result
    result: Option<bool>,
}

impl GameInstance
{
    /// Make a new game instance.
    ///
    /// The report sender is for a channel that passes reports out of the instance.
    pub fn new(
        game_id           : u64,
        command_sender    : IoSender<GameInstanceCommand>,
        _command_receiver : IoReceiver<GameInstanceCommand>,
        instance_handle   : enfync::PendingResult<bool>,
    ) -> GameInstance
    {
        GameInstance{ game_id, command_sender, _command_receiver, instance_handle, result: None }
    }

    /// Send a command to the game instance.
    /// - Should never return `Err`.
    pub fn send_command(&self, command: GameInstanceCommand) -> Result<(), ()>
    {
        self.command_sender.send(command).map_err(|_| ())
    }

    /// Get the game's id.
    pub fn id(&self) -> u64
    {
        self.game_id
    }

    /// Check if the instance is running.
    pub fn is_running(&mut self) -> bool
    {
        self.try_get().is_none()
    }

    /// Try to get the runner result.
    /// - Returns `None` if no result is available.
    /// - Returns `Some(false)` if the runner failed erroneously.
    /// - Returns `Some(true)` if the runner closed without error.
    pub fn try_get(&mut self) -> Option<bool>
    {
        // try to return the saved result
        if self.result.is_some() { return self.result; }

        // see if a result is available
        let Some(result) = self.instance_handle.try_extract() else { return None; };
        let bool_result = result.map_or_else(|_| false, |ok| ok);

        // save the result and return it
        self.result = Some(bool_result);
        self.result
    }

    /// Get the result.
    /// - Returns `false` if the runner failed erroneously.
    /// - Returns `true` if the runner closed without error.
    pub async fn get(&mut self) -> bool
    {
        // try to return the saved result
        if self.result.is_some() { return self.result.unwrap(); }

        // wait for the result to appear
        let result = self.instance_handle.extract().await;
        let bool_result = result.map_or_else(|_| false, |ok| ok);

        // save the result and return it
        self.result = Some(bool_result);
        bool_result
    }
}

//-------------------------------------------------------------------------------------------------------------------
