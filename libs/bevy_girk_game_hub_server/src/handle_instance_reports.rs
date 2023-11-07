//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn instance_report_game_start(
    In((game_id, game_start_report)) : In<(u64, GameStartReport)>,
    running_games_cache              : Res<RunningGamesCache>,
    host_client                      : Res<HostHubClient>,
){
    // get game start request for this game
    // - if the game start report is very delayed due to some async issue, it may appear after the game has already
    //   finished (unlikely and unwanted, but technically not an error)
    let Some(game_start_request) = running_games_cache.game_start_request(game_id)
    else { tracing::warn!(game_id, "dropping game start report for game not present in running games cache"); return; };

    // forward game start report to host server
    // - we include the game start request so the server can check for consistency with its local records
    if let Err(_) = host_client.send(
            HubToHostMsg::GameStart{
                    id      : game_id,
                    request : game_start_request.clone(),
                    report  : game_start_report 
                }
        )
    { tracing::error!(game_id, "failed sending game start report to host"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn instance_report_game_over(
    In((game_id, game_over_report)) : In<(u64, GameOverReport)>,
    mut running_games_cache         : ResMut<RunningGamesCache>,
    host_client                     : Res<HostHubClient>,
){
    // forward game over report to host server
    if let Err(_) = host_client.send(HubToHostMsg::GameOver{ id: game_id, report: game_over_report })
    { tracing::error!(game_id, "failed sending game over report to host"); }

    // try to remove instance
    if let Some(_) = running_games_cache.extract_instance(game_id)
    { tracing::trace!(game_id, "removed game instance from running games in response to game over report"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn instance_report_game_aborted(
    In(game_id)             : In<u64>,
    mut running_games_cache : ResMut<RunningGamesCache>,
    host_client             : Res<HostHubClient>,
){
    // try to remove instance
    // - note that the instance may have already been removed for one reason or another
    if let None = running_games_cache.extract_instance(game_id)
    { tracing::trace!(game_id, "aborted game instance not in running games"); return; }
    else
    { tracing::trace!(game_id, "removed aborted game instance from running games"); }

    // notify host server
    // - only notify if the aborted game was removed; we assume the host server was already notified if the game isn't
    //   present
    if let Err(_) = host_client.send(HubToHostMsg::AbortGame{ id: game_id })
    { tracing::error!(game_id, "failed sending abort game to host"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_instance_reports(world: &mut World)
{
    while let Some(instance_report) = world.resource_mut::<RunningGamesCache>().try_get_next_instance_report()
    {
        match instance_report
        {
            GameInstanceReport::GameStart(id, report) => syscall(world, (id, report), instance_report_game_start),
            GameInstanceReport::GameOver(id, report)  => syscall(world, (id, report), instance_report_game_over),
            GameInstanceReport::GameAborted(id)       => syscall(world, id, instance_report_game_aborted),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
