//local shortcuts
use crate::{Lobby, LobbyMemberData};

//third-party shortcuts

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

pub trait LobbyChecker: Debug + Send + Sync
{
    /// Check if a lobby is semantically valid.
    fn check_lobby(&self, lobby: &Lobby) -> bool;

    /// Check if a new lobby member may be added to a lobby.
    fn allow_new_member(
        &self,
        lobby       : &Lobby,
        member_id   : u128,
        member_data : LobbyMemberData,
        password    : &String,
    ) -> bool;

    /// Check if a lobby is launchable.
    fn can_launch(&self, lobby: &Lobby) -> bool;
}

//-------------------------------------------------------------------------------------------------------------------
