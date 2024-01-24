//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of game ticks elapsed.
pub(crate) fn advance_game_tick(mut game_tick : ResMut<GameTick>)
{
    *game_tick.0 += 1;
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of prep ticks elapsed.
pub(crate) fn advance_prep_tick(mut prep_tick : ResMut<PrepTick>)
{
    *prep_tick.0 += 1;
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

/// Check the game duration conditions and update the game mode.
pub(crate) fn update_game_mode(
    game_ctx           : Res<ClickGameContext>,
    game_tick         : Res<GameTick>,
    current_game_mode  : Res<State<GameMode>>,
    mut next_game_mode : ResMut<NextState<GameMode>>
){
    // get expected mode based on elapsed ticks
    let duration_config    = game_ctx.duration_config();
    let expected_game_mode = duration_config.expected_mode(**game_tick);

    // update the game mode
    if expected_game_mode == **current_game_mode
        { return; }
    next_game_mode.set(expected_game_mode);
    tracing::info!(?expected_game_mode, "new game mode");
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper function-system for accessing the game mode.
pub(crate) fn get_current_game_mode(current_game_mode: Res<State<GameMode>>) -> GameMode
{
    **current_game_mode
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify a single client of the current game mode.
pub(crate) fn notify_game_mode_single(
    In(client_id)     : In<ClientIdType>,
    buffer            : Res<GameMessageBuffer>,
    current_game_mode : Res<State<GameMode>>,
){
    buffer.send(
            GameMsg::CurrentGameMode(**current_game_mode),
            vec![InfoAccessConstraint::Targets(vec![client_id])]
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game mode.
pub(crate) fn notify_game_mode_all(
    buffer            : Res<GameMessageBuffer>,
    current_game_mode : Res<State<GameMode>>,
){
    buffer.send(
            GameMsg::CurrentGameMode(**current_game_mode),
            vec![]
        );
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn set_game_end_flag(
    game_tick         : Res<GameTick>,
    players           : Query<(&PlayerId, &PlayerScore)>,
    mut game_end_flag : ResMut<GameEndFlag>,
){
    // collect player reports
    let player_reports =
        players
            .iter()
            .map(
                |(&player_id, &score)|
                {
                    ClickPlayerReport{ client_id: player_id.id, score }
                }
            )
            .collect();

    // build game over report
    let game_over_report = ClickGameOverReport{
            last_game_tick: **game_tick,
            player_reports,
        };

    // serialize it
    let game_over_report_final = GameOverReport::new(ser_msg(&game_over_report));

    // set the game end flag
    game_end_flag.set(game_over_report_final);
    tracing::info!("game end flag set");
}

//-------------------------------------------------------------------------------------------------------------------
