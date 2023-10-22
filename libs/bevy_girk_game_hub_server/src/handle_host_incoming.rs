//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot::ecs::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn host_connected(world: &mut World)
{
    tracing::trace!("host server connected");

    // reset capacity tracker
    let mut capacity_tracker = world.resource_mut::<GameHubCapacityTracker>();
    capacity_tracker.reset();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn host_start_game(
    In(game_start_request)      : In<GameStartRequest>,
    capacity_tracker            : Res<GameHubCapacityTracker>,
    mut pending_games_cache     : ResMut<PendingGamesCache>,
    running_games_cache         : Res<RunningGamesCache>,
    mut game_launch_pack_source : ResMut<GameLaunchPackSource>,
    host_client                 : Res<HostHubClient>,
){
    let game_id = game_start_request.game_id();

    // ignore if present in pending game or game registries
    if pending_games_cache.has_game(game_id)
    { tracing::warn!(game_id, "received game start request but game is already pending"); return; }
    if running_games_cache.has_game(game_id)
    { tracing::warn!(game_id, "received game start request but game is already running"); return; }

    // send 'abort game' if no capacity
    if capacity_tracker.capacity() == GameHubCapacity(0u16)
    {
        if let Err(_) = host_client.send(HubToHostMsg::AbortGame{ id: game_id })
        { tracing::error!(game_id, "failed sending abort game to host"); }
        return;
    }

    // request launch pack
    game_launch_pack_source.request_launch_pack(&game_start_request);

    // register as pending game
    if let Err(_) = pending_games_cache.add_pending_game(game_start_request)
    { tracing::error!(game_id, "failed adding pending game to cache"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn host_abort_game(
    In(game_id)             : In<u64>,
    mut pending_games_cache : ResMut<PendingGamesCache>,
    mut running_games_cache : ResMut<RunningGamesCache>,
    host_client             : Res<HostHubClient>,
){
    // try to remove from pending game registry
    if let Some(_) = pending_games_cache.extract_game(game_id)
    {
        // notify host server
        if let Err(_) = host_client.send(HubToHostMsg::AbortGame{ id: game_id })
        { tracing::error!(game_id, "failed sending abort game to host"); }
        return;
    }

    // try to remove the game instance and tell it to abort
    let Some(instance) = running_games_cache.extract_instance(game_id)
    else { tracing::trace!(game_id, "tried to abort game but could not find it"); return; };
    if let Err(_) = instance.send_command(GameInstanceCommand::AbortGame)
    { tracing::error!(game_id, "failed sending abort game command to game instance"); }

    // notify host server
    // - if the game was not available then we don't notify the server, because we assume it was notified by another
    //   process in the hub
    if let Err(_) = host_client.send(HubToHostMsg::AbortGame{ id: game_id })
    { tracing::error!(game_id, "failed sending abort game to host"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub (crate) fn handle_host_connection_reports(world: &mut World)
{
    while let Some(connection_report) = world.resource::<HostHubClient>().next_report()
    {
        match connection_report
        {
            HostHubClientReport::Connected         => syscall(world, (), host_connected),
            HostHubClientReport::Disconnected      => (),
            HostHubClientReport::ClosedByServer(_) => (),
            HostHubClientReport::ClosedBySelf      => (),
            HostHubClientReport::IsDead            => world.send_event(bevy::app::AppExit),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_host_incoming(world: &mut World)
{
    while let Some(host_val) = world.resource::<HostHubClient>().next_val()
    {
        let HostHubServerVal::Msg(host_message) = host_val
        else { tracing::warn!("received non-message server val"); continue; };

        match host_message
        {
            HostToHubMsg::StartGame(req) => syscall(world, req, host_start_game),
            HostToHubMsg::AbortGame{id}  => syscall(world, id, host_abort_game),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
