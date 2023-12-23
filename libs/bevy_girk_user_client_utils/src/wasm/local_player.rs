//local shortcuts
use crate::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/*

- game instance app
- game client app with shared-process connect info
- run apps concurrently
    - game client takes over renderer

*/

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub struct ClientMonitorLocalWasm;

//-------------------------------------------------------------------------------------------------------------------

impl<S> ClientMonitorImpl for ClientMonitorLocalWasm<S>
{
    fn game_id(&self) -> u64
    {
        u64::MAX
    }

    fn is_running(&self) -> bool
    {
        false
    }

    fn send_token(&mut self, _token: ServerConnectToken)
    {}

    fn kill(&mut self)
    {}

    fn take_result(&mut self) -> Result<Option<GameOverReport>, ()>
    {
        Err(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct LocalPlayerLauncherConfigWasm<S>(std::marker::PhantomData<S>);

//-------------------------------------------------------------------------------------------------------------------

/// Launches a local single-player game.
pub fn launch_local_player_game_wasm(
    _config               : LocalPlayerLauncherConfigWasm<()>,
    _launch_pack          : GameLaunchPack,
    _client_report_sender : IoSender<ClientInstanceReport>,
) -> ClientMonitorLocalWasm
{
    ClientMonitorLocalWasm
}

//-------------------------------------------------------------------------------------------------------------------
