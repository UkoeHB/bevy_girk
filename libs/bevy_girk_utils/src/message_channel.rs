//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Multi-producer.
#[derive(Component, Resource, Clone, Debug)]
pub struct MessageSender<T>
{
    sender: crossbeam::channel::Sender<T>
}

impl<T> MessageSender<T>
{
    fn new(sender: crossbeam::channel::Sender<T>) -> MessageSender<T>
    {
        MessageSender{ sender }
    }

    pub fn send(&self, message: T) -> Result<(), crossbeam::channel::SendError<T>>
    {
        self.sender.send(message)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Multi-consumer.
#[derive(Component, Resource, Clone, Debug)]
pub struct MessageReceiver<T>
{
    receiver: crossbeam::channel::Receiver<T>
}

impl<T> MessageReceiver<T>
{
    fn new(receiver: crossbeam::channel::Receiver<T>) -> MessageReceiver<T>
    {
        MessageReceiver{ receiver }
    }

    pub fn try_get_next(&self) -> Option<T>
    {
        let Ok(msg) = self.receiver.try_recv() else { return None; };
        Some(msg)
    }

    pub fn len(&self) -> usize
    {
        self.receiver.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn new_message_channel<T>() -> (MessageSender<T>, MessageReceiver<T>)
{
    let (channel_sender, channel_receiver) = crossbeam::channel::unbounded::<T>();
    return (MessageSender::<T>::new(channel_sender), MessageReceiver::<T>::new(channel_receiver));
}

//-------------------------------------------------------------------------------------------------------------------
