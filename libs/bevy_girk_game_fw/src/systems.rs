//local shortcuts
use crate::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

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

/// Check if init mode is over.
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

/// Update the game framework mode.
pub(crate) fn update_game_fw_mode(
    game_fw_config     : Res<GameFWConfig>,
    game_ticks         : Res<GameFWTicksElapsed>,
    client_readiness   : Query<&Readiness, With<ClientId>>,
    game_end_flag      : Res<GameEndFlag>,
    current_game_mode  : Res<State<GameFWMode>>,
    mut next_game_mode : ResMut<NextState<GameFWMode>>
){
    // 1. -> End mode
    // a. early exit if we are already in GameFWMode::End
    if *current_game_mode == GameFWMode::End { return; }

    // b. check mode -> End transition condition
    if game_end_flag.is_set()
    {
        let next_mode = GameFWMode::End;
        next_game_mode.set(next_mode);
        tracing::info!(?next_mode, "updated game framework mode");
        return;
    }

    // 2. -> Game mode
    // a. early exit if we are already in GameFWMode::Game
    if *current_game_mode == GameFWMode::Game { return; }

    // b. check mode -> Game transition condition
    if init_mode_is_over(game_fw_config.max_init_ticks(), game_ticks.elapsed.ticks(), &client_readiness)
    {
        let next_mode = GameFWMode::Game;
        next_game_mode.set(next_mode);
        tracing::info!(?next_mode, "updated game framework mode");
        return;
    }

    // 3. -> Init mode
    if *current_game_mode != GameFWMode::Init { panic!("Unexpected current game mode (should be GameFWMode::Init)!"); }
}

//-------------------------------------------------------------------------------------------------------------------

/// Increment the number of game framework ticks elapsed.
pub(crate) fn advance_game_fw_tick(mut game_ticks : ResMut<GameFWTicksElapsed>)
{
    game_ticks.elapsed.advance();
}

//-------------------------------------------------------------------------------------------------------------------

/// Set total initialization progress of the game.
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

/// Take client messages, filter by information access rights, dispatch to clients.
/// Note: the `GamePacket` receiver may drop messages sent to disconnected clients.
//todo: refactor to use bevy_replicon rooms
pub(crate) fn dispatch_messages_to_client(
    mut game_message_buffer : ResMut<GameMessageBuffer>,
    game_ticks              : Res<GameFWTicksElapsed>,
    game_message_sender     : Res<MessageSender<GamePacket>>,
    client_query            : Query<(&ClientId, &InfoAccessRights)>
){
    let ticks = game_ticks.elapsed.ticks();

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
                            message     : GameMessage{ message: pending_game_message.message.clone(), ticks }
                        }
                ).expect("game fw message dispatch sender should always succeed");
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify a single client of the current game framework mode.
pub(crate) fn notify_game_fw_mode_single(
    In(client_id)           : In<ClientIdType>,
    current_game_mode       : Res<State<GameFWMode>>,
    mut game_message_buffer : ResMut<GameMessageBuffer>
){
    game_message_buffer.add_fw_msg(
            &GameFWMsg::CurrentGameFWMode(**current_game_mode),
            vec![InfoAccessConstraint::Targets(vec![client_id])],
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game framework mode.
pub(crate) fn notify_game_fw_mode_all(
    current_game_mode       : Res<State<GameFWMode>>,
    mut game_message_buffer : ResMut<GameMessageBuffer>
){
    game_message_buffer.add_fw_msg(
            &GameFWMsg::CurrentGameFWMode(**current_game_mode),
            vec![],
            SendOrdered
        );
}

//-------------------------------------------------------------------------------------------------------------------
