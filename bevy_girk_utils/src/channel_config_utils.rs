//local shortcuts

//third-party shortcuts
use bevy_replicon::prelude::Channel;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct SendUnreliable;
#[derive(Debug, Copy, Clone)]
pub struct SendUnordered;
#[derive(Debug, Copy, Clone)]
pub struct SendOrdered;

impl From<SendUnreliable> for Channel
{
    fn from(_: SendUnreliable) -> Channel { Channel::Unreliable }
}
impl From<SendUnordered> for Channel
{
    fn from(_: SendUnordered) -> Channel { Channel::Unordered }
}
impl From<SendOrdered> for Channel
{
    fn from(_: SendOrdered) -> Channel { Channel::Ordered }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for converting a message type into its send policy.
///
/// Especially useful for enum-type messages where different variants have different send policies.
pub trait IntoChannel
{
    fn into_event_type(&self) -> Channel;
}

//-------------------------------------------------------------------------------------------------------------------

/// Default implementation for tests.
impl IntoChannel for ()
{
    fn into_event_type(&self) -> Channel { Channel::Unreliable }
}

//-------------------------------------------------------------------------------------------------------------------
