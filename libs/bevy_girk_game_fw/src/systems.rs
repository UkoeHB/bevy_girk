//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns true if all clients are ready.
fn clients_are_ready(client_readiness: &Query<&Readiness, With<ClientId>>) -> bool
{
    for readiness in client_readiness
    {
        if !readiness.is_ready() { return false; }
    }

    return true;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Checks if init mode is over.
fn init_mode_is_over(
    max_init_ticks   : Ticks,
    game_ticks       : Ticks,
    client_readiness : &Query<&Readiness, With<ClientId>>
) -> bool
{
    if game_ticks > max_init_ticks         { return true; }
    if clients_are_ready(client_readiness) { return true; }
    return false;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the game framework mode.
pub(crate) fn update_game_fw_mode(
    game_fw_config     : Res<GameFwConfig>,
    game_ticks         : Res<GameFwTicksElapsed>,
    client_readiness   : Query<&Readiness, With<ClientId>>,
    game_end_flag      : Res<GameEndFlag>,
    current_game_mode  : Res<State<GameFwMode>>,
    mut next_game_mode : ResMut<NextState<GameFwMode>>
){
    // 1. -> End mode
    // a. early exit if we are already in GameFwMode::End
    if *current_game_mode == GameFwMode::End { return; }

    // b. check mode -> End transition condition
    if game_end_flag.is_set()
    {
        let next_mode = GameFwMode::End;
        next_game_mode.set(next_mode);
        tracing::info!(?next_mode, "updated game framework mode");
        return;
    }

    // 2. -> Game mode
    // a. early exit if we are already in GameFwMode::Game
    if *current_game_mode == GameFwMode::Game { return; }

    // b. check mode -> Game transition condition
    if init_mode_is_over(game_fw_config.max_init_ticks(), game_ticks.elapsed.ticks(), &client_readiness)
    {
        let next_mode = GameFwMode::Game;
        next_game_mode.set(next_mode);
        tracing::info!(?next_mode, "updated game framework mode");
        return;
    }

    // 3. -> Init mode
    if *current_game_mode != GameFwMode::Init { panic!("Unexpected current game mode (should be GameFwMode::Init)!"); }
}

//-------------------------------------------------------------------------------------------------------------------

/// Increments the number of game framework ticks elapsed.
pub(crate) fn advance_game_fw_tick(mut game_ticks : ResMut<GameFwTicksElapsed>)
{
    game_ticks.elapsed.advance();
}

//-------------------------------------------------------------------------------------------------------------------

/// Resets the game message buffer for a new tick.
pub(crate) fn reset_game_message_buffer(mut buffer: ResMut<GameMessageBuffer>, game_ticks: Res<GameFwTicksElapsed>)
{
    buffer.reset(game_ticks.elapsed.ticks());
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets total initialization progress of the game.
pub(crate) fn refresh_game_init_progress(
    change_query      : Query<(), (With<ClientId>, Changed<Readiness>)>,
    readiness_query   : Query<&Readiness, With<ClientId>>,
    mut init_progress : Query<&mut GameInitProgress>
){
    // check if any client's readiness changed
    if change_query.is_empty() || readiness_query.is_empty() { return; }

    // update game init progress entity
    let mut total = 0.0;
    let mut count = 0;
    for readiness in &readiness_query
    {
        total += readiness.value();
        count += 1;
    }

    init_progress.single_mut().0 = total / (count as f32);
}

//-------------------------------------------------------------------------------------------------------------------

/// Takes client messages, filters by information access rights, dispatches to clients.
///
/// Note: The `GamePacket` receiver may drop messages sent to disconnected clients.
//todo: refactor to use bevy_replicon rooms
pub(crate) fn dispatch_messages_to_client(
    mut game_message_buffer : ResMut<GameMessageBuffer>,
    game_message_sender     : Res<Sender<GamePacket>>,
    client_query            : Query<(&ClientId, &InfoAccessRights)>
){
    for pending_game_message in game_message_buffer.drain()
    {
        for (client_id, access_rights) in &client_query
        {
            if !access_rights.can_access(&pending_game_message.access_constraints)
                { continue; }
            game_message_sender.send(
                    GamePacket{
                            client_id   : client_id.id(),
                            send_policy : pending_game_message.send_policy,
                            message     : pending_game_message.message.clone()
                        }
                ).expect("game fw message dispatch sender should always succeed");
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Notifies a single client of the current game framework mode.
pub(crate) fn notify_game_fw_mode_single(
    In(client_id)           : In<ClientIdType>,
    current_game_mode       : Res<State<GameFwMode>>,
    mut game_message_buffer : ResMut<GameMessageBuffer>
){
    game_message_buffer.push_fw(
            GameFwMsg::CurrentGameFwMode(**current_game_mode),
            vec![InfoAccessConstraint::Targets(vec![client_id])],
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game framework mode.
pub(crate) fn notify_game_fw_mode_all(
    current_game_mode       : Res<State<GameFwMode>>,
    mut game_message_buffer : ResMut<GameMessageBuffer>
){
    game_message_buffer.push_fw(
            GameFwMsg::CurrentGameFwMode(**current_game_mode),
            vec![],
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Starts the 'end mode' countdown, which will end in closing the app.
pub(crate) fn start_end_countdown(game_ticks: Res<GameFwTicksElapsed>, mut game_end_tick: ResMut<GameFwEndTick>)
{
    game_end_tick.0 = Some(game_ticks.elapsed.ticks());
}

//-------------------------------------------------------------------------------------------------------------------

/// Exits the app if all game end ticks have elapsed.
///
/// If the max game end ticks equals zero, then the app will be exited in the same tick that `GameFwMode::End` is set.
//todo: consider exiting early if all clients have acked the game end state
pub(crate) fn try_exit_app(
    current_game_mode : Res<State<GameFwMode>>,
    game_ticks        : Res<GameFwTicksElapsed>,
    game_end_tick     : Res<GameFwEndTick>,
    game_fw_config    : Res<GameFwConfig>,
    mut app_exit      : EventWriter<AppExit>,
){
    // sanity check
    if *current_game_mode != GameFwMode::End
    { tracing::error!("tried to terminate game app but not in GameFwMode::End"); return; }

    // get end tick
    let Some(end_tick) = game_end_tick.0
    else { tracing::error!("tried to terminate game app but game fw end tick is missing"); return; };

    // check if game end ticks have elapsed
    if game_ticks.elapsed.ticks().0.saturating_sub(end_tick.0) < game_fw_config.max_end_ticks().0 { return; }

    // exit the game
    tracing::info!("exiting game app");
    app_exit.send(AppExit{});
}

//-------------------------------------------------------------------------------------------------------------------
