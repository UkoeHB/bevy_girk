//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;

//third-party shortcuts
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HostToUserMsg
{
    LobbyState{ lobby: LobbyData },
    LobbyLeave{ id: u64 },
    PendingLobbyAckRequest{ id: u64 },
    PendingLobbyAckFail{ id: u64 },
    GameStart{ id: u64, connect: GameConnectInfo },
    GameAborted{ id: u64 },
    GameOver{ id: u64, report: GameOverReport },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HostToUserResponse
{
    /// Response to [`UserToHostRequest::LobbySearch`].
    LobbySearchResult(LobbySearchResult),
    /// Response to [`UserToHostRequest::MakeLobby`] and [`UserToHostRequest::JoinLobby`].
    LobbyJoin{ lobby: LobbyData },
}

//-------------------------------------------------------------------------------------------------------------------

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserToHostMsg
{
    NackPendingLobby{ id: u64 },
    AckPendingLobby{ id: u64 },
}

//-------------------------------------------------------------------------------------------------------------------

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserToHostRequest
{
    LobbySearch(LobbySearchRequest),
    MakeLobby{
        mcolor: LobbyMemberColor,
        pwd: String,
        #[serde_as(as = "Bytes")]
        data: Vec<u8>
    },
    JoinLobby{ id: u64, mcolor: LobbyMemberColor, pwd: String },
    /// Will be acked on success.
    LeaveLobby{ id: u64 },
    /// Will be acked on success.
    LaunchLobbyGame{ id: u64 },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HostUserChannel;
impl bevy_simplenet::ChannelPack for HostUserChannel
{
    type ConnectMsg = ();
    type ServerMsg = HostToUserMsg;
    type ServerResponse = HostToUserResponse;
    type ClientMsg = UserToHostMsg;
    type ClientRequest = UserToHostRequest;
}

//-------------------------------------------------------------------------------------------------------------------

// SERVER: defined in `host_server` crate

//-------------------------------------------------------------------------------------------------------------------

/// CLIENT
pub type HostUserClient      = bevy_simplenet::Client<HostUserChannel>;
pub type HostUserClientEvent = bevy_simplenet::ClientEventFrom<HostUserChannel>;

/// client factory
pub fn host_user_client_factory() -> bevy_simplenet::ClientFactory<HostUserChannel>
{
    bevy_simplenet::ClientFactory::<HostUserChannel>::new(PACKAGE_VERSION)
}

//-------------------------------------------------------------------------------------------------------------------
