//local shortcuts
use bevy_girk_game_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::ClientId;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of game ticks elapsed.
pub(crate) fn advance_game_tick(mut game_tick : ResMut<GameTick>)
{
    *game_tick.0 += 1;
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of play ticks elapsed.
pub(crate) fn advance_play_tick(mut play_tick : ResMut<PlayTick>)
{
    *play_tick.0 += 1;
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of game-over ticks elapsed.
pub(crate) fn advance_game_over_tick(mut game_over_tick : ResMut<GameOverTick>)
{
    *game_over_tick.0 += 1;
}

//-------------------------------------------------------------------------------------------------------------------

/// Check the game duration conditions and update the game state.
pub(crate) fn update_game_state(
    game_ctx            : Res<ClickGameContext>,
    game_tick           : Res<GameTick>,
    current_game_state  : Res<State<GameState>>,
    mut next_game_state : ResMut<NextState<GameState>>
){
    // get expected state based on elapsed ticks
    let duration_config     = game_ctx.duration_config();
    let expected_game_state = duration_config.expected_state(**game_tick);

    // update the game state
    if expected_game_state == **current_game_state { return; }
    next_game_state.set(expected_game_state);
    tracing::info!(?expected_game_state, "new game state");
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper function-system for accessing the game state.
pub(crate) fn get_current_game_state(current_game_state: Res<State<GameState>>) -> GameState
{
    **current_game_state
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify a single client of the current game state.
pub(crate) fn notify_game_state_single(
    In(client_id)     : In<ClientId>,
    mut sender        : GameSender,
    current_game_state : Res<State<GameState>>,
){
    sender.send_to_client(GameMsg::CurrentGameState(**current_game_state), client_id.get());
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game state.
pub(crate) fn notify_game_state_all(
    mut sender        : GameSender,
    current_game_state : Res<State<GameState>>,
){
    sender.send_to_all(GameMsg::CurrentGameState(**current_game_state));
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn set_game_end_flag(
    game_tick         : Res<GameTick>,
    players           : Query<(&PlayerId, &PlayerScore)>,
    mut game_end_flag : ResMut<GameEndFlag>,
){
    // collect player reports
    let player_reports = players
        .iter()
        .map(|(&player_id, &score)| ClickPlayerReport{ client_id: player_id.id, score })
        .collect();

    // build game over report
    let game_over_report = ClickGameOverReport{
        last_game_tick: **game_tick,
        player_reports,
    };

    // serialize it
    let report = GameOverReport::new(&game_over_report);

    // set the game end flag
    game_end_flag.set(report);
    tracing::info!("game end flag set");
}

//-------------------------------------------------------------------------------------------------------------------
