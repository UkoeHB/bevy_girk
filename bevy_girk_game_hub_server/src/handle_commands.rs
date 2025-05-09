//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;

//third-party shortcuts
use bevy::app::*;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn command_set_max_capacity(In(new_max_capacity): In<GameHubCapacity>, world: &mut World)
{
    // set max capacity
    world.resource_mut::<GameHubCapacityTracker>().set_max_capacity(new_max_capacity);

    // update capacity tracker immediately (and send updated capacity to host server)
    // - We do this because the host-hub channel is LIFO. We want all messages sent by the hub after this point
    //   to arrive after the host receives a hub capacity update.
    world.syscall((), update_capacity);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn command_shut_down(
    mut pending_games_cache : ResMut<PendingGamesCache>,
    mut running_games_cache : ResMut<RunningGamesCache>,
    host_client             : Res<HostHubClient>,
    mut app_exit            : EventWriter<AppExit>,
){
    tracing::info!("shutting down game hub...");

    // remove all pending games
    for game_start_request in pending_games_cache.drain_all()
    {
        let game_id = game_start_request.game_id();
        tracing::warn!(game_id, "...removed pending game");

        // notify the host the game was aborted
        host_client.send(HubToHostMsg::Abort{ id: game_id });
    }

    // remove all running games
    for mut game_instance in running_games_cache.drain_all()
    {
        let game_id = game_instance.id();
        tracing::warn!(game_id, "...removed running game");

        // if game instance completed successfully, we don't need to do anything else
        if let Some(true) = game_instance.try_get() { continue; }

        // command game instance to abort (otherwise it may hang)
        if let Err(_) = game_instance.send_command(GameInstanceCommand::Abort)
        { tracing::error!(game_id, "failed sending abort game command to game instance"); }

        // notify the host the game was aborted
        host_client.send(HubToHostMsg::Abort{ id: game_id });
    }

    // close the app at the end of this tick
    app_exit.write(AppExit::Success);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_commands(world: &mut World)
{
    while let Some(command) = world.resource::<Receiver<GameHubCommand>>().try_recv()
    {
        match command
        {
            GameHubCommand::SetMaxCapacity(capacity) => world.syscall(capacity, command_set_max_capacity),
            GameHubCommand::ShutDown                 =>
            {
                world.syscall(GameHubCapacity(0u16), command_set_max_capacity);
                world.syscall((), command_shut_down);
            },
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
