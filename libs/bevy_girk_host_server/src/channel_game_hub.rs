//local shortcuts
use crate::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HostToHubMsg
{
    StartGame(GameStartRequest),
    AbortGame{ id: u64 },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HubToHostMsg
{
    Capacity(GameHubCapacity),
    AbortGame{ id: u64 },
    GameStart{ id: u64, request: GameStartRequest, report: GameStartReport },
    GameOver{ id: u64, report: GameOverReport },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HostHubChannel;
impl bevy_simplenet::ChannelPack for HostHubChannel
{
    type ConnectMsg = ();
    type ServerMsg = HostToHubMsg;
    type ServerResponse = ();
    type ClientMsg = HubToHostMsg;
    type ClientRequest = ();
}

//-------------------------------------------------------------------------------------------------------------------

/// SERVER
pub type HostHubServer       = bevy_simplenet::Server<HostHubChannel>;
pub type HostHubServerReport = bevy_simplenet::ServerReport<()>;
pub type HostHubServerEvent  = bevy_simplenet::ServerEventFrom<HostHubChannel>;

/// server factory
pub fn host_hub_server_factory() ->  bevy_simplenet::ServerFactory<HostHubChannel>
{
    bevy_simplenet::ServerFactory::<HostHubChannel>::new(PACKAGE_VERSION)
}

//-------------------------------------------------------------------------------------------------------------------

/// CLIENT
pub type HostHubClient    = bevy_simplenet::Client<HostHubChannel>;
pub type HostHubClientEvent = bevy_simplenet::ClientEventFrom<HostHubChannel>;

/// client factory
pub fn host_hub_client_factory() -> bevy_simplenet::ClientFactory<HostHubChannel>
{
    bevy_simplenet::ClientFactory::<HostHubChannel>::new(PACKAGE_VERSION)
}

//-------------------------------------------------------------------------------------------------------------------
