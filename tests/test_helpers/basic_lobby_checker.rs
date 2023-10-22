//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn count_members(lobby: &Lobby) -> Option<(usize, usize)>
{
    let mut num_players = 0;
    let mut num_watchers = 0;
    for member_data in lobby.data.members.iter().map(|(_, color)| color)
    {
        match BasicLobbyMemberType::try_from(member_data.color)
        {
            Ok(BasicLobbyMemberType::Player)  => num_players += 1,
            Ok(BasicLobbyMemberType::Watcher) => num_watchers += 1,
            Err(_)                            => return None,
        }
    }

    Some((num_players, num_watchers))
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BasicLobbyMemberType
{
    Player,
    Watcher,
}

impl TryFrom<LobbyMemberColor> for BasicLobbyMemberType
{
    type Error = ();

    fn try_from(color: LobbyMemberColor) -> Result<BasicLobbyMemberType, ()>
    {
        match color.0
        {
            0u64 => Ok(BasicLobbyMemberType::Player),
            1u64 => Ok(BasicLobbyMemberType::Watcher),
            _    => Err(())
        }
    }
}

impl Into<LobbyMemberColor> for BasicLobbyMemberType
{
    fn into(self) -> LobbyMemberColor
    {
        match self
        {
            BasicLobbyMemberType::Player  => LobbyMemberColor(0u64),
            BasicLobbyMemberType::Watcher => LobbyMemberColor(1u64),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct BasicLobbyChecker
{
    /// Max number of players allowed in a lobby.
    pub max_lobby_players: u16,
    /// Max number of watchers allowed in a lobby.
    pub max_lobby_watchers: u16,
    /// Min number of players in a lobby required to launch a lobby.
    pub min_players_to_launch: u16,
}

impl LobbyChecker for BasicLobbyChecker
{
    /// Check if a lobby is semantically valid.
    fn check_lobby(&self, lobby: &Lobby) -> bool
    {
        // no custom lobby data allowed
        if lobby.custom_data().len() > 0 { return false; }

        // excessively large passwords not allowed
        if lobby.get_password().len() > 15 { return false; }

        // get max count member types
        let Some((num_players, num_watchers)) = count_members(lobby) else { return false; };

        // check configs
        if num_players  > self.max_lobby_players  as usize { return false; }
        if num_watchers > self.max_lobby_watchers as usize { return false; }

        true
    }

    /// Check if a new lobby member may be added to a lobby.
    fn allow_new_member(
        &self,
        lobby       : &Lobby,
        member_id   : u128,
        member_data : LobbyMemberData,
        password    : &String,
    ) -> bool
    {
        // check if in lobby already
        if lobby.has_member(member_id) { return false; }

        // check password
        if lobby.get_password() != password { return false; }

        // get member type
        let Ok(member_type) = BasicLobbyMemberType::try_from(member_data.color) else { return false; };

        // count current players and watchers
        let Some((num_players, num_watchers)) = count_members(lobby) else { return false; };

        // check if the member's type has exceeded lobby capacity
        match member_type
        {
            BasicLobbyMemberType::Player =>
            {
                if num_players >= self.max_lobby_players as usize { return false; }
            }
            BasicLobbyMemberType::Watcher =>
            {
                if num_watchers >= self.max_lobby_watchers as usize { return false; }
            }
        }

        true
    }

    /// Check if a lobby is launchable.
    fn can_launch(&self, lobby: &Lobby) -> bool
    {
        // count players
        let Some((num_players, _)) = count_members(lobby) else { return false; };

        // check config
        if num_players < self.min_players_to_launch as usize { return false; }

        true
    }
}

//-------------------------------------------------------------------------------------------------------------------
