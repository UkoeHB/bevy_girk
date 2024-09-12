//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub struct UserClientUtilsPlugin;

impl Plugin for UserClientUtilsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ClientStarterPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
