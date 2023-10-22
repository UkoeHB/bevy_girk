//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ClickLobbyMemberType
{
    Player,
    Watcher,
}

impl TryFrom<LobbyMemberColor> for ClickLobbyMemberType
{
    type Error = ();

    fn try_from(color: LobbyMemberColor) -> Result<ClickLobbyMemberType, ()>
    {
        match color.0
        {
            0u64 => Ok(ClickLobbyMemberType::Player),
            1u64 => Ok(ClickLobbyMemberType::Watcher),
            _    => Err(())
        }
    }
}

impl Into<LobbyMemberColor> for ClickLobbyMemberType
{
    fn into(self) -> LobbyMemberColor
    {
        match self
        {
            ClickLobbyMemberType::Player  => LobbyMemberColor(0u64),
            ClickLobbyMemberType::Watcher => LobbyMemberColor(1u64),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ClickLobbyChecker
{
    /// Max number of players allowed in a lobby.
    pub max_lobby_players: u16,
    /// Max number of watchers allowed in a lobby.
    pub max_lobby_watchers: u16,
    /// Min number of players in a lobby required to launch a lobby.
    pub min_players_to_launch: u16,
}

impl ClickLobbyChecker
{
    pub fn count_members(lobby_data: &LobbyData) -> Result<(usize, usize), ()>
    {
        let mut num_players = 0;
        let mut num_watchers = 0;
        for member_data in lobby_data.members.iter().map(|(_, color)| color)
        {
            match ClickLobbyMemberType::try_from(member_data.color)?
            {
                ClickLobbyMemberType::Player  => num_players += 1,
                ClickLobbyMemberType::Watcher => num_watchers += 1,
            }
        }

        Ok((num_players, num_watchers))
    }

    pub fn collect_members(
        lobby_data: &LobbyData
    ) -> Result<(Vec<(bevy_simplenet::EnvType, u128)>, Vec<(bevy_simplenet::EnvType, u128)>), ()>
    {
        let mut players = Vec::default();
        let mut watchers = Vec::default();
        for (user_id, member_data) in lobby_data.members.iter()
        {
            match ClickLobbyMemberType::try_from(member_data.color)?
            {
                ClickLobbyMemberType::Player  => players.push((member_data.env, *user_id)),
                ClickLobbyMemberType::Watcher => watchers.push((member_data.env, *user_id)),
            }
        }

        Ok((players, watchers))
    }
}

impl LobbyChecker for ClickLobbyChecker
{
    /// Check if a lobby is semantically valid.
    fn check_lobby(&self, lobby: &Lobby) -> bool
    {
        // no custom lobby data allowed
        if lobby.custom_data().len() > 0 { return false; }

        // excessively large passwords not allowed
        if lobby.get_password().len() > 15 { return false; }

        // get max count member types
        let Ok((num_players, num_watchers)) = Self::count_members(&lobby.data) else { return false; };

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

        // validate env type
        //todo: allow WASM env
        if member_data.env != bevy_simplenet::EnvType::Native { panic!("only native clients currently supported"); }

        // get member type
        let Ok(member_type) = ClickLobbyMemberType::try_from(member_data.color) else { return false; };

        // count current players and watchers
        let Ok((num_players, num_watchers)) = Self::count_members(&lobby.data) else { return false; };

        // check if the member's type has exceeded lobby capacity
        match member_type
        {
            ClickLobbyMemberType::Player =>
            {
                if num_players >= self.max_lobby_players as usize { return false; }
            }
            ClickLobbyMemberType::Watcher =>
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
        let Ok((num_players, _)) = Self::count_members(&lobby.data) else { return false; };

        // check config
        if num_players < self.min_players_to_launch as usize { return false; }

        true
    }
}

//-------------------------------------------------------------------------------------------------------------------
