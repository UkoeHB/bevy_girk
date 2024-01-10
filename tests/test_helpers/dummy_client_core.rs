//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_client_fw::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
pub fn DummyClientCorePlugin(app: &mut App)
{
    app.insert_resource(GameMessageHandler::new(
            | _: &mut World, packet: &GamePacket | -> Result<(), Option<(Ticks, GameFwMsg)>>
            {
                deserialize_game_message::<()>(packet)?;
                Ok(())
            }
        ));
}

//-------------------------------------------------------------------------------------------------------------------
