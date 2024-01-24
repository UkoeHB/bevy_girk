//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use iyes_progress::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Resets the [`GameInitProgress`] component if it exists.
/// 
/// Runs when the client has just disconnected.
pub(crate) fn reset_init_progress(mut progress: Query<&mut GameInitProgress>)
{
    let Ok(mut progress) = progress.get_single_mut() else { return; };
    tracing::trace!("resetting GameInitProgress");
    progress.reset();
}

//-------------------------------------------------------------------------------------------------------------------

/// Resets the game message buffer for a new tick.
pub(crate) fn reset_client_request_buffer(mut buffer: ResMut<ClientRequestBuffer>)
{
    buffer.reset();
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the client's intialization cache.
///
/// Note: The `ProgressCounter` resource is removed when it reaches 100%, but we may still need the initialization cache.
pub(crate) fn update_initialization_cache(
    progress_counter : Option<Res<ProgressCounter>>,
    mut cache        : ResMut<InitializationProgressCache>,
){
    match &progress_counter
    {
        Some(counter) => cache.set_progress(counter.progress_complete()),
        None          => cache.set_progress_complete()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sends client initialization progress report to the game.
pub(crate) fn send_initialization_progress_report(
    cache  : Res<InitializationProgressCache>,
    buffer : Res<ClientRequestBuffer>
){
    // don't send if no intialization progress has been made since last frame
    if !cache.progress_changed_last_update() { return; }

    // sent progress report
    buffer.fw_request(ClientFwRequest::SetInitProgress(cache.progress().into()));
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets [`ClientFwMode::Connecting`] and [`ClientInitializationState::InProgress`].
pub(crate) fn reinitialize_client_fw(
    mut client_init_state : ResMut<NextState<ClientInitializationState>>,
    mut client_fw_mode    : ResMut<NextState<ClientFwMode>>
){
    tracing::info!("connecting client");
    client_init_state.set(ClientInitializationState::InProgress);
    client_fw_mode.set(ClientFwMode::Connecting);
}

//-------------------------------------------------------------------------------------------------------------------

/// Requests the current game framework mode.
pub(crate) fn request_game_fw_mode(buffer: Res<ClientRequestBuffer>)
{
    buffer.fw_request(ClientFwRequest::GetGameFwMode);
}

//-------------------------------------------------------------------------------------------------------------------

/// Takes client messages and dispatches them to the game.
pub(crate) fn dispatch_client_packets(
    mut buffer         : ResMut<ClientRequestBuffer>,
    mut client_packets : EventWriter<ClientPacket>,
){
    while let Some(pending_packet) = buffer.next()
    {
        client_packets.send(pending_packet);
    }
}

//-------------------------------------------------------------------------------------------------------------------
