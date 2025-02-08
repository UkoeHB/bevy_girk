//local shortcuts
use crate::{LobbyData, LobbyMemberColor, LobbySearchRequest, LobbySearchResult};
use bevy_girk_game_fw::GameOverReport;
use bevy_girk_game_instance::GameStartInfo;

//third-party shortcuts
use renet2_setup::{ConnectionType, ServerConnectToken};
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HostToUserMsg
{
    LobbyState{ lobby: LobbyData },
    LobbyLeave{ id: u64 },
    PendingLobbyAckRequest{ id: u64 },
    PendingLobbyAckFail{ id: u64 },
    GameStart{ id: u64, token: ServerConnectToken, start: GameStartInfo },
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
    /// Response to [`UserToHostRequest::GetConnectToken`];
    ConnectToken{ id: u64, token: ServerConnectToken },
}

//-------------------------------------------------------------------------------------------------------------------

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
    /// Request lobby data from the server.
    LobbySearch(LobbySearchRequest),
    /// Make a new lobby.
    MakeLobby{
        mcolor: LobbyMemberColor,
        pwd: String,
        #[serde_as(as = "Bytes")]
        data: Vec<u8>
    },
    /// Join the specified lobby.
    JoinLobby{ id: u64, mcolor: LobbyMemberColor, pwd: String },
    /// Leave the specified lobby.
    ///
    /// Will be acked on success.
    LeaveLobby{ id: u64 },
    /// Try to launch the specified lobby as a game.
    ///
    /// Only the lobby owner can send this.
    ///
    /// Will be acked on success.
    LaunchLobbyGame{ id: u64 },
    /// Request a game connect token.
    ///
    /// Used to reconnect to ongoing games.
    GetConnectToken{ id: u64 },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostUserConnectMsg
{
    pub connection_type: ConnectionType
}

impl HostUserConnectMsg
{
    /// Makes a new connect message.
    pub fn new() -> Self
    {
        Self{ connection_type: ConnectionType::inferred() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HostUserChannel;
impl bevy_simplenet::ChannelPack for HostUserChannel
{
    type ConnectMsg = HostUserConnectMsg;
    type ServerMsg = HostToUserMsg;
    type ServerResponse = HostToUserResponse;
    type ClientMsg = UserToHostMsg;
    type ClientRequest = UserToHostRequest;
}

//-------------------------------------------------------------------------------------------------------------------

// SERVER: defined in `host_server` crate

//-------------------------------------------------------------------------------------------------------------------

/// CLIENT
#[cfg(feature = "client")]
pub type HostUserClient = bevy_simplenet::Client<HostUserChannel>;
#[cfg(feature = "client")]
pub type HostUserClientEvent = bevy_simplenet::ClientEventFrom<HostUserChannel>;

/// client factory
#[cfg(feature = "client")]
pub fn host_user_client_factory() -> bevy_simplenet::ClientFactory<HostUserChannel>
{
    const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
    bevy_simplenet::ClientFactory::<HostUserChannel>::new(PACKAGE_VERSION)
}

//-------------------------------------------------------------------------------------------------------------------
