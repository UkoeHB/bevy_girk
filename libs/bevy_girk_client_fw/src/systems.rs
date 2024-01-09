//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use iyes_progress::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Resets the game message buffer for a new tick.
pub(crate) fn reset_client_request_buffer(mut buffer: ResMut<ClientRequestBuffer>)
{
    buffer.reset();
}

//-------------------------------------------------------------------------------------------------------------------

/// Update client's intialization cache.
///
/// Note: the ProgressCounter is removed when it reaches 100%, but we may still need the initialization cache.
pub(crate) fn update_initialization_cache(
    progress_counter                  : Option<Res<ProgressCounter>>,
    mut initialization_progress_cache : ResMut<InitializationProgressCache>,
){
    match &progress_counter
    {
        Some(counter) => initialization_progress_cache.set_progress(counter.progress_complete()),
        None          => initialization_progress_cache.set_progress_complete()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Send client initialization progress report to the game.
pub(crate) fn send_initialization_progress_report(
    initialization_progress_cache : Res<InitializationProgressCache>,
    mut client_message_buffer     : ResMut<ClientRequestBuffer>
){
    // don't send if no intialization progress has been made since last frame
    if !initialization_progress_cache.progress_changed_last_update() { return; }

    // sent progress report
    client_message_buffer.push_fw(
            ClientFwRequest::SetInitProgress(initialization_progress_cache.progress().into()),
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Change client framework mode to 'init'.
pub(crate) fn reinitialize_client_fw(
    mut client_initialization_state : ResMut<NextState<ClientInitializationState>>,
    mut client_fw_mode              : ResMut<NextState<ClientFwMode>>
){
    tracing::info!("reinitializing client framework");
    client_initialization_state.set(ClientInitializationState::InProgress);
    client_fw_mode.set(ClientFwMode::Init);
}

//-------------------------------------------------------------------------------------------------------------------

/// Request the current game framework mode.
pub(crate) fn request_game_fw_mode(mut client_message_buffer: ResMut<ClientRequestBuffer>)
{
    client_message_buffer.push_fw(ClientFwRequest::GetGameFwMode, SendUnordered);
}

//-------------------------------------------------------------------------------------------------------------------

/// Take client messages, dispatch to game.
pub(crate) fn dispatch_client_packets(
    mut client_message_buffer : ResMut<ClientRequestBuffer>,
    client_packet_sender      : Res<Sender<ClientPacket>>,
    client_config             : Res<ClientFwConfig>
){
    for pending_client_message in client_message_buffer.drain()
    {
        client_packet_sender.send(
                ClientPacket{
                        client_id   : client_config.client_id(),
                        send_policy : pending_client_message.send_policy,
                        message     : pending_client_message.request,
                    }
            ).expect("client fw packet dispatch sender should always succeed");
    }
}

//-------------------------------------------------------------------------------------------------------------------
