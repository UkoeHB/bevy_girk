//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_fn_plugin::bevy_plugin;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
pub fn UserClientUtilsPlugin(app: &mut App)
{
    app.add_plugins(ClientMonitorPlugin)
        .add_plugins(ClientStarterPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
