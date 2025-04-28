//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_replicon_attributes::*;
use renet2::ClientId;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Checks if init state ended in the previous tick.
fn init_state_is_over(max_init_ticks: u32, game_fw_tick: Tick, client_readiness: &ClientReadiness) -> bool
{
    if *game_fw_tick > max_init_ticks
    || client_readiness.all_ready() { return true; }

    false
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Increments the [`GameFwTick`].
pub(crate) fn advance_game_fw_tick(mut game_fw_tick: ResMut<GameFwTick>)
{
    *game_fw_tick.0 = game_fw_tick.0.checked_add(1).expect("GameFwTick exceeded max tick size");
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the game framework state.
///
/// This runs at the very start of a tick after incrementing the tick counter.
pub(crate) fn update_game_fw_state(
    game_fw_config      : Res<GameFwConfig>,
    game_fw_tick        : Res<GameFwTick>,
    client_readiness    : Res<ClientReadiness>,
    game_end_flag       : Res<GameEndFlag>,
    current_game_state  : Res<State<GameFwState>>,
    mut next_game_state : ResMut<NextState<GameFwState>>
){
    // 1. -> End state
    // a. early exit if we are already in GameFwState::End
    if *current_game_state == GameFwState::End { return; }

    // b. check state -> End transition condition
    if game_end_flag.is_set()
    {
        let next_state = GameFwState::End;
        next_game_state.set(next_state);
        tracing::info!(?next_state, "updated game framework state");
        return;
    }

    // 2. -> Game state
    // a. early exit if we are already in GameFwState::Game
    if *current_game_state == GameFwState::Game { return; }

    // b. check state -> Game transition condition
    if init_state_is_over(game_fw_config.max_init_ticks(), **game_fw_tick, &client_readiness)
    {
        let next_state = GameFwState::Game;
        next_game_state.set(next_state);
        tracing::info!(?next_state, "updated game framework state");
        return;
    }

    // 3. -> Init state
    if *current_game_state != GameFwState::Init { panic!("unexpected current game state (should be GameFwState::Init)"); }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets total initialization progress of the game.
pub(crate) fn refresh_game_init_progress(
    client_readiness  : Res<ClientReadiness>,
    mut init_progress : Query<&mut GameInitProgress>
){
    // check if any client's readiness changed
    if !client_readiness.is_changed() { return }

    // update game init progress entity
    let Ok(mut init_progress) = init_progress.single_mut() else { return };
    init_progress.0 = client_readiness.total_progress();
}

//-------------------------------------------------------------------------------------------------------------------

/// Notifies a single client of the current game framework state.
pub(crate) fn notify_game_fw_state_single(
    In(client_id) : In<ClientId>,
    mut sender    : GameSender,
    current_state : Res<State<GameFwState>>,
){
    sender.fw_send(GameFwMsg::CurrentState(**current_state), vis!(Client(client_id)));
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game framework state.
pub(crate) fn notify_game_fw_state_all(
    mut sender    : GameSender,
    current_state : Res<State<GameFwState>>,
){
    sender.fw_send(GameFwMsg::CurrentState(**current_state), vis!(Global));
}

//-------------------------------------------------------------------------------------------------------------------

/// Starts the 'end state' countdown, which will end in closing the app.
pub(crate) fn start_end_countdown(ending_game_fw_tick: Res<GameFwTick>, mut game_end_tick: ResMut<GameFwPreEndTick>)
{
    game_end_tick.0 = Some(Tick(***ending_game_fw_tick));
}

//-------------------------------------------------------------------------------------------------------------------

/// Exits the app if all game end ticks have elapsed.
///
/// If [`GameFwConfig::max_end_ticks`] is <= 1, then the app will be exited at the end of the tick where [`GameFwState::End`]
/// is set.
//todo: consider exiting early if all clients have acked the game end state
pub(crate) fn try_exit_app(
    current_game_state : Res<State<GameFwState>>,
    game_fw_tick       : Res<GameFwTick>,
    game_end_tick      : Res<GameFwPreEndTick>,
    game_fw_config     : Res<GameFwConfig>,
    mut app_exit       : EventWriter<AppExit>,
){
    // sanity check
    if *current_game_state != GameFwState::End
    { tracing::error!("tried to terminate game app but not in GameFwState::End"); return; }

    // check if all game end ticks have elapsed
    if game_end_tick.num_end_ticks(*game_fw_tick) < game_fw_config.max_end_ticks() { return; }

    // exit the game
    tracing::info!("exiting game app");
    app_exit.write(AppExit::Success);
}

//-------------------------------------------------------------------------------------------------------------------
