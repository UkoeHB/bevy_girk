//local shortcuts
use crate::*;
use bevy_girk_game_fw::ClientFwRequest;

//third-party shortcuts
use bevy::prelude::*;
use iyes_progress::prelude::ProgressTracker;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Updates the client's intialization cache.
///
/// Note: The `ProgressCounter` resource is removed when it reaches 100%, but we may still need the initialization cache.
pub(crate) fn update_initialization_cache(
    progress_counter : Option<Res<ProgressTracker<ClientInitState>>>,
    mut cache        : ResMut<InitProgressCache>,
){
    match &progress_counter
    {
        Some(counter) => cache.set_progress(counter.get_global_progress()),
        None          => cache.set_progress_complete()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sends client initialization progress report to the game.
pub(crate) fn try_send_initialization_progress_report(
    cache      : Res<InitProgressCache>,
    mut sender : ClientSender
){
    // don't send if no intialization progress has been made since last frame
    if !cache.progress_changed_last_update() { return; }

    // sent progress report
    sender.fw_send(ClientFwRequest::SetInitProgress(cache.progress().into()));
}

//-------------------------------------------------------------------------------------------------------------------

/// Requests the current game framework state.
pub(crate) fn request_game_fw_state(mut sender: ClientSender)
{
    sender.fw_send(ClientFwRequest::GetGameFwState);
}

//-------------------------------------------------------------------------------------------------------------------
