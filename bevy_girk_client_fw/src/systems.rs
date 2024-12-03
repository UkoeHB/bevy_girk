//local shortcuts
use crate::*;
use bevy_girk_game_fw::ClientFwRequest;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use iyes_progress::prelude::ProgressTracker;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Make sure all replicated entities are scoped properly.
pub(crate) fn prep_replicated_entities(
    mut c: Commands,
    replicated: Query<Entity, (Added<Replicated>, Without<StateScoped<ClientInstanceState>>)>
)
{
    for entity in replicated.iter() {
        c.entity(entity).insert(StateScoped(ClientInstanceState::Game));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the client's intialization cache.
///
/// Note: The `ProgressCounter` resource is removed when it reaches 100%, but we may still need the initialization cache.
pub(crate) fn update_initialization_cache(
    progress_counter : Option<Res<ProgressTracker::<ClientInitializationState>>>,
    mut cache        : ResMut<InitializationProgressCache>,
){
    match &progress_counter
    {
        Some(counter) => cache.set_progress(counter.get_global_progress()),
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

/// Requests the current game framework state.
pub(crate) fn request_game_fw_state(mut sender: ClientRequestSender)
{
    sender.fw_request(ClientFwRequest::GetGameFwState);
}

//-------------------------------------------------------------------------------------------------------------------
