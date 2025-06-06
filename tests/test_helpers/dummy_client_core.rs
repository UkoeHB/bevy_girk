//local shortcuts
use bevy_girk_client_fw::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub struct DummyClientCorePlugin;

impl Plugin for DummyClientCorePlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(GameMessageHandler::new(|_, _, _: ()| {}));
    }
}

//-------------------------------------------------------------------------------------------------------------------
