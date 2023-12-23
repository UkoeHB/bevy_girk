//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(target_family = "wasm"))]
pub type LocalPlayerLauncherConfig<S> = LocalPlayerLauncherConfigNative<S>;
#[cfg(target_family = "wasm")]
pub type LocalPlayerLauncherConfig<S> = LocalPlayerLauncherConfigWasm<S>;

#[cfg(not(target_family = "wasm"))]
pub type MultiPlayerLauncherConfig<S> = MultiPlayerLauncherConfigNative<S>;
#[cfg(target_family = "wasm")]
pub type MultiPlayerLauncherConfig<S> = MultiPlayerLauncherConfigWasm<S>;

#[cfg(target_family = "wasm")]
pub trait HandleReqs {}
#[cfg(target_family = "wasm")]
impl HandleReqs for () {}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a local single-player game client.
///
/// The `monitor` can be used to interact with the client.
pub fn launch_local_player_client<S: HandleReqs>(
    monitor     : &mut ClientMonitor,
    config      : LocalPlayerLauncherConfig<S>,
    launch_pack : GameLaunchPack,
){
    tracing::info!("launching local single-player client");

    let client_report_sender = monitor.reset_report_channel();

    let monitor_impl =
    {
        #[cfg(not(target_family = "wasm"))]
        {
            launch_local_player_client_native(config, launch_pack, client_report_sender)
        }

        #[cfg(target_family = "wasm")]
        {
            launch_local_player_client_wasm(config, launch_pack, client_report_sender)
        }
    };

    monitor.set(monitor_impl);
}

//-------------------------------------------------------------------------------------------------------------------

/// Launch a multiplayer game client.
///
/// The `monitor` can be used to interact with the client.
pub fn launch_multiplayer_client<S: HandleReqs>(
    monitor    : &mut ClientMonitor,
    config     : MultiPlayerLauncherConfig<S>,
    token      : ServerConnectToken,
    start_info : GameStartInfo,
){
    tracing::info!("launching multiplayer client");

    let client_report_sender = monitor.reset_report_channel();

    let monitor_impl =
    {
        #[cfg(not(target_family = "wasm"))]
        {
            launch_multiplayer_client_native(config, token, start_info, client_report_sender)
        }

        #[cfg(target_family = "wasm")]
        {
            launch_multiplayer_client_wasm(config, token, start_info, client_report_sender)
        }
    };

    monitor.set(monitor_impl);
}

//-------------------------------------------------------------------------------------------------------------------
