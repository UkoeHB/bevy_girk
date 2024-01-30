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
///
// Note: Does not interfere with `bevy_replicon_repair` because:
// A) This runs in [`ClientFwMode::Connecting`] and repair runs in the first tick of [`ClientFwMode::Init`].
// B) This component should never be removed by the server, so it should always show up in the replicon sync message
//    regardless of possible conflict with repair.
pub(crate) fn reset_init_progress(mut progress: Query<&mut GameInitProgress>)
{
    let Ok(mut progress) = progress.get_single_mut() else { return; };
    tracing::trace!("resetting GameInitProgress");
    progress.reset();
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
    cache      : Res<InitializationProgressCache>,
    mut sender : ClientRequestSender
){
    // don't send if no intialization progress has been made since last frame
    if !cache.progress_changed_last_update() { return; }

    // sent progress report
    sender.fw_request(ClientFwRequest::SetInitProgress(cache.progress().into()));
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
pub(crate) fn request_game_fw_mode(mut sender: ClientRequestSender)
{
    sender.fw_request(ClientFwRequest::GetGameFwMode);
}

//-------------------------------------------------------------------------------------------------------------------
