//local shortcuts
use crate::*;

//third-party shortcuts

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------

pub struct ClientMonitorMultiplayerWasm;

//-------------------------------------------------------------------------------------------------------------------

impl ClientMonitorImpl for ClientMonitorMultiplayerWasm
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

#[derive(Debug, Clone)]
pub struct MultiPlayerLauncherConfigWasm<S>(std::marker::PhantomData<S>);

//-------------------------------------------------------------------------------------------------------------------

/// Launches a multiplayer game on WASM targets.
pub fn launch_multiplayer_client_wasm(
    _config               : MultiPlayerLauncherConfigWasm<()>,
    _token                : ServerConnectToken,
    _start_info           : GameStartInfo,
    _client_report_sender : IoSender<ClientInstanceReport>,
) -> ClientMonitorMultiplayerWasm
{
    ClientMonitorMultiplayerWasm
}

//-------------------------------------------------------------------------------------------------------------------
