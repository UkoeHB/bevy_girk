//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Handle current game framework mode.
pub(crate) fn handle_current_game_fw_mode(
    In(current_game_fw_mode)    : In<GameFWMode>,
    client_initialization_state : Res<State<ClientInitializationState>>,
    current_client_fw_mode      : Res<State<ClientFWMode>>,
    mut next_client_fw_mode     : ResMut<NextState<ClientFWMode>>
){
    // do not update client framework mode if we are in the process of initializing the client
    // - reason: we don't want to leave Init until we are really done initializing
    if *client_initialization_state != ClientInitializationState::Done { return; }

    // update mode
    let new_client_fw_mode =
        match current_game_fw_mode
        {
            GameFWMode::Init => ClientFWMode::Init,
            GameFWMode::Game => ClientFWMode::Game,
            GameFWMode::End  => ClientFWMode::End,
        };

    if new_client_fw_mode == **current_client_fw_mode { return; }
    next_client_fw_mode.set(new_client_fw_mode);
    tracing::info!(?new_client_fw_mode, "new client framework mode");
}

//-------------------------------------------------------------------------------------------------------------------

/// Handle ping response.
pub(crate) fn handle_ping_response(
    In((game_ticks_elapsed, response)) : In<(Ticks, PingResponse)>,
    time                               : Res<Time>,
    mut ping_tracker                   : ResMut<PingTracker>
){
    ping_tracker.add_ping_cycle(game_ticks_elapsed, response.request.timestamp_ns, time.elapsed().as_nanos() as u64);
}

//-------------------------------------------------------------------------------------------------------------------
