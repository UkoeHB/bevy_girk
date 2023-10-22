//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of game ticks elapsed.
pub(crate) fn advance_game_tick(mut game_ticks : ResMut<GameTicksElapsed>)
{
    game_ticks.elapsed.advance();
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of prep ticks elapsed.
pub(crate) fn advance_prep_tick(mut prep_ticks : ResMut<PrepTicksElapsed>)
{
    prep_ticks.elapsed.advance();
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of play ticks elapsed.
pub(crate) fn advance_play_tick(mut play_ticks : ResMut<PlayTicksElapsed>)
{
    play_ticks.elapsed.advance();
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of game-over ticks elapsed.
pub(crate) fn advance_game_over_tick(mut game_over_ticks : ResMut<GameOverTicksElapsed>)
{
    game_over_ticks.elapsed.advance();
}

//-------------------------------------------------------------------------------------------------------------------

/// Check the game duration conditions and update the game mode.
pub(crate) fn update_game_mode(
    game_ctx           : Res<ClickGameContext>,
    game_ticks         : Res<GameTicksElapsed>,
    current_game_mode  : Res<State<GameMode>>,
    mut next_game_mode : ResMut<NextState<GameMode>>
){
    // get expected mode based on elapsed ticks
    let duration_config    = game_ctx.duration_config();
    let expected_game_mode = duration_config.expected_mode(game_ticks.elapsed.ticks());

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
    In(client_id)           : In<ClientIdType>,
    current_game_mode       : Res<State<GameMode>>,
    mut game_message_buffer : ResMut<GameMessageBuffer>
){
    game_message_buffer.add_core_msg(
            &GameMsg::CurrentGameMode(**current_game_mode),
            vec![InfoAccessConstraint::Targets(vec![client_id])],
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game mode.
pub(crate) fn notify_game_mode_all(
    current_game_mode       : Res<State<GameMode>>,
    mut game_message_buffer : ResMut<GameMessageBuffer>
){
    game_message_buffer.add_core_msg(
            &GameMsg::CurrentGameMode(**current_game_mode),
            vec![],
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn set_game_end_flag(
    game_ticks        : Res<GameTicksElapsed>,
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
            game_ticks: game_ticks.elapsed.ticks(),
            player_reports,
        };

    // serialize it
    let game_over_report_final = GameOverReport::new(ser_msg(&game_over_report));

    // set the game end flag
    game_end_flag.set(game_over_report_final);
    tracing::info!("game end flag set");
}

//-------------------------------------------------------------------------------------------------------------------
