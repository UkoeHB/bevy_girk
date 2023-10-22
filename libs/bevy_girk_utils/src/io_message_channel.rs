//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Single-producer (IO: tokio).
#[derive(Component, Resource, Clone, Debug)]
pub struct IOMessageSender<T>
{
    sender: tokio::sync::mpsc::UnboundedSender<T>
}

impl<T> IOMessageSender<T>
{
    fn new(sender: tokio::sync::mpsc::UnboundedSender<T>) -> IOMessageSender<T>
    {
        IOMessageSender{ sender }
    }

    /// Send a message.
    /// Returns `Err` if the channel is closed.
    pub fn send(&self, message: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>>
    {
        self.sender.send(message)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Single-consumer (IO: tokio).
#[derive(Component, Resource, Debug)]
pub struct IOMessageReceiver<T>
{
    receiver: tokio::sync::mpsc::UnboundedReceiver<T>
}

impl<T> IOMessageReceiver<T>
{
    fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<T>) -> IOMessageReceiver<T>
    {
        IOMessageReceiver{ receiver }
    }

    /// Get next available message.
    /// Returns `None` if the channel is closed.
    pub async fn get_next(&mut self) -> Option<T>
    {
        self.receiver.recv().await
    }

    /// Get next available message.
    /// Returns `None` if there are no available messages or the channel is closed.
    pub fn try_get_next(&mut self) -> Option<T>
    {
        let Ok(msg) = self.receiver.try_recv() else { return None; };
        Some(msg)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn new_io_message_channel<T>() -> (IOMessageSender<T>, IOMessageReceiver<T>)
{
    let (channel_sender, channel_receiver) = tokio::sync::mpsc::unbounded_channel::<T>();
    (IOMessageSender::<T>::new(channel_sender), IOMessageReceiver::<T>::new(channel_receiver))
}

//-------------------------------------------------------------------------------------------------------------------
