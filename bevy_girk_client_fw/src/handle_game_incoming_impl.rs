//local shortcuts
use crate::*;
use bevy_girk_game_fw::{GameFwState, PingResponse, Tick};

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Handles a notification for the current game framework state.
pub(crate) fn handle_current_game_fw_state(
    In(current_game_fw_state)   : In<GameFwState>,
    client_initialization_state : Res<State<ClientInitializationState>>,
    current_client_fw_state     : Res<State<ClientFwState>>,
    mut next_client_fw_state    : ResMut<NextState<ClientFwState>>
){
    // do not update client framework state if we are in the process of initializing the client
    // - reason: we don't want to leave Init until we are really done initializing
    if *client_initialization_state != ClientInitializationState::Done { return; }

    // update state
    let state =
        match current_game_fw_state
        {
            GameFwState::Init => ClientFwState::Init,
            GameFwState::Game => ClientFwState::Game,
            GameFwState::End  => ClientFwState::End,
        };

    if state == **current_client_fw_state { return; }
    next_client_fw_state.set(state);
    let old_state = **current_client_fw_state;
    tracing::info!(?old_state, ?state, "setting client framework state");
}

//-------------------------------------------------------------------------------------------------------------------

/// Handles a ping response.
pub(crate) fn handle_ping_response(
    In((game_fw_tick, response)) : In<(Tick, PingResponse)>,
    time                         : Res<Time>,
    mut ping_tracker             : ResMut<PingTracker>
){
    ping_tracker.add_ping_cycle(game_fw_tick, response.request.timestamp_ns, time.elapsed().as_nanos() as u64);
}

//-------------------------------------------------------------------------------------------------------------------
