//local shortcuts

//third-party shortcuts
use bevy_replicon::prelude::{ChannelKind, ClientId, SendMode};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct TargetClient(pub u64);
#[derive(Debug, Copy, Clone)]
pub struct TargetAll;
#[derive(Debug, Copy, Clone)]
pub struct TargetAllExcept(pub u64);

impl From<TargetClient> for SendMode
{
    fn from(client: TargetClient) -> SendMode { SendMode::Direct(ClientId::new(client.0)) }
}
impl From<TargetAll> for SendMode
{
    fn from(_: TargetAll) -> SendMode { SendMode::Broadcast }
}
impl From<TargetAllExcept> for SendMode
{
    fn from(exception: TargetAllExcept) -> SendMode { SendMode::BroadcastExcept(ClientId::new(exception.0)) }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct SendUnreliable;
#[derive(Debug, Copy, Clone)]
pub struct SendUnordered;
#[derive(Debug, Copy, Clone)]
pub struct SendOrdered;

impl From<SendUnreliable> for ChannelKind
{
    fn from(_: SendUnreliable) -> ChannelKind { ChannelKind::Unreliable }
}
impl From<SendUnordered> for ChannelKind
{
    fn from(_: SendUnordered) -> ChannelKind { ChannelKind::Unordered }
}
impl From<SendOrdered> for ChannelKind
{
    fn from(_: SendOrdered) -> ChannelKind { ChannelKind::Ordered }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for converting a message type into its send policy.
///
/// Especially useful for enum-type messages where different variants have different send policies.
pub trait IntoChannelKind
{
    fn into_event_type(&self) -> ChannelKind;
}

//-------------------------------------------------------------------------------------------------------------------

/// Default implementation for tests.
impl IntoChannelKind for ()
{
    fn into_event_type(&self) -> ChannelKind { ChannelKind::Unreliable }
}

//-------------------------------------------------------------------------------------------------------------------
