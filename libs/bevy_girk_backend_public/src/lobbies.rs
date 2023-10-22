//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LobbySearchType
{
    LobbyId(u64),
    Page{ youngest_lobby_id: u64, num_lobbies: u16 }
}

//-------------------------------------------------------------------------------------------------------------------

/// Opaque type signifying what a lobby member is (e.g. player/watcher, team membership, etc.).
///
/// Note: For simplicity, all member types must ack a lobby before it can start.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LobbyMemberColor(pub u64);

//-------------------------------------------------------------------------------------------------------------------

/// Lobby member data.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LobbyMemberData
{
    pub env: bevy_simplenet::EnvType,
    pub color: LobbyMemberColor,
}

//-------------------------------------------------------------------------------------------------------------------

/// Lobby data.
///
/// Note: We require `Eq` and `PartialEq` in order to validate that game start reports from game hubs are associated
/// with exactly the same lobby that we have cached, since lobby contents can theoretically change after requesting
/// 'game start' from a game hub.
#[serde_as]
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LobbyData
{
    /// This lobby's id.
    pub id: u64,
    /// The id of this lobby's owner.
    pub owner_id: u128,
    /// Custom lobby data defined by the lobby creator.
    #[serde_as(as = "Bytes")]
    pub serialized_custom_data: Vec<u8>,

    /// Lobby members.
    pub members: HashMap<u128, LobbyMemberData>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
pub struct Lobby
{
    /// Lobby data.
    pub data: LobbyData,
    /// Lobby password.
    password: String
}

impl Lobby
{
    /// Make a new lobby (owner is not automatically inserted into member list).
    pub fn new(lobby_id: u64, owner_id: u128, password: String, serialized_custom_data: Vec<u8>) -> Lobby
    {
        Lobby{
                data: LobbyData { id: lobby_id, owner_id, serialized_custom_data, ..Default::default() },
                password,
            }
    }

    /// Add a member to the lobby.
    pub fn add_member(&mut self, new_member_id: u128, member_color: LobbyMemberData) -> bool
    {
        // add based on member type
        self.data.members.insert(new_member_id, member_color).is_none()
    }

    /// Remove a member from the lobby.
    /// - Returns `None` if the member doesn't exist (or is the owner).
    pub fn remove_member(&mut self, member_id: u128) -> Option<LobbyMemberData>
    {
        if self.is_owner(member_id) { return None; }
        self.data.members.remove(&member_id)
    }

    /// This lobby's id.
    pub fn id(&self) -> u64 { self.data.id }

    /// Test if the given id is the lobby's owner.
    pub fn is_owner(&self, test_owner_id: u128) -> bool
    {
        test_owner_id == self.data.owner_id
    }

    /// Test if the lobby has a member (they may be the lobby owner).
    pub fn has_member(&self, member_id: u128) -> bool
    {
        self.data.members.contains_key(&member_id)
    }

    /// Get member color.
    /// - Returns `None` if member doesn't exist.
    pub fn member_color(&self, member_id: u128) -> Option<LobbyMemberData>
    {
        self.data.members.get(&member_id).copied()
    }

    /// Get number of lobby members.
    pub fn num_members(&self) -> usize
    {
        self.data.members.len()
    }

    /// Get the password.
    pub fn get_password(&self) -> &String
    {
        &self.password
    }

    /// Read the custom lobby data.
    pub fn custom_data(&self) -> &Vec<u8>
    {
        &self.data.serialized_custom_data
    }
}

//-------------------------------------------------------------------------------------------------------------------
