//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_report_pack(
    In(launch_pack)         : In<GameLaunchPack>,
    mut pending_games_cache : ResMut<PendingGamesCache>,
    mut running_games_cache : ResMut<RunningGamesCache>,
){
    let game_id = launch_pack.game_id;

    // remove pending game
    // - discard launch pack if we can't upgrade 
    let Some(game_start_request) = pending_games_cache.extract_game(game_id)
    else { tracing::warn!(game_id, "received launch pack but game is not in pending games cache"); return; };

    // launch game
    if let Err(_) = running_games_cache.make_instance(game_start_request, launch_pack)
    { tracing::warn!(game_id, "failed to make game instance"); }
    else
    { tracing::info!(game_id, "launched game instance"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_report_failure(
    In(game_id)             : In<u64>,
    mut pending_games_cache : ResMut<PendingGamesCache>,
    host_client             : Res<HostHubClient>,
){
    tracing::warn!(game_id, "failed to acquire launch pack for a game");

    // discard pending game
    if let None = pending_games_cache.extract_game(game_id)
    { tracing::warn!(game_id, "tried to discard pending game but game is not present"); return; }

    // notify host server of aborted game
    host_client.send(HubToHostMsg::Abort{ id: game_id });
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_launch_pack_reports(world: &mut World)
{
    while let Some(launch_pack_report) = world.resource_mut::<GameLaunchPackSource>().try_next_report()
    {
        match launch_pack_report
        {
            GameLaunchPackReport::Pack(launch_pack) => world.syscall(launch_pack, handle_report_pack),
            GameLaunchPackReport::Failure(game_id)  => world.syscall(game_id, handle_report_failure),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
