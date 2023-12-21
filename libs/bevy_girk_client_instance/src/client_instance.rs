//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A client instance is a wrapper around a running client.
///
/// The wrapper monitors the client for completion.
#[derive(Debug)]
pub struct ClientInstance
{
    /// The client instance's game id.
    game_id: u64,
    /// Join handle for the client instance (used to detect status of the instance).
    instance_handle: enfync::PendingResult<bool>,
    /// Cached result.
    result: Option<bool>,
}

impl ClientInstance
{
    /// Make a new client instance.
    pub fn new(
        game_id         : u64,
        instance_handle : enfync::PendingResult<bool>,
    ) -> ClientInstance
    {
        ClientInstance{ game_id, instance_handle, result: None }
    }

    /// Get the client's id.
    pub fn id(&self) -> u64
    {
        self.game_id
    }

    /// Check if instance is running.
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
