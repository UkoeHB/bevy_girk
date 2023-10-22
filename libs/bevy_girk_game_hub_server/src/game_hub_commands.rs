//local shortcuts
use bevy_girk_host_server::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub enum GameHubCommand
{
    SetMaxCapacity(GameHubCapacity),
    ShutDown,
    //todo: get stats
}

//-------------------------------------------------------------------------------------------------------------------
