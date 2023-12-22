//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ClientRunnerState
{
    /// Expected protocol ids for new connect tokens.
    pub(crate) protocol_id: u64,
    /// This game's id.
    pub(crate) game_id: u64,
    /// Sends reports to the instance's owner.
    pub(crate) report_sender: IoSender<ClientInstanceReport>,
    /// Receives commands from the instance's owner.
    pub(crate) command_receiver: IoReceiver<ClientInstanceCommand>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up a client app for a client instance.
/// - Makes a new client app configured for use in a client instance.
/// - When you run the app, it will continue updating until manually shut down.
pub fn client_instance_setup(
    client_factory   : ClientFactory,
    token            : ServerConnectToken,
    start_info       : GameStartInfo,
    report_sender    : IoSender<ClientInstanceReport>,
    command_receiver : IoReceiver<ClientInstanceCommand>,
) -> Result<App, ()>
{
    // add client to app
    let game_id = start_info.game_id;
    let (mut client_app, protocol_id) = client_factory.new_client(token, start_info)?;

    // make runner state
    let runner_state = ClientRunnerState{
            protocol_id,
            game_id,
            report_sender,
            command_receiver,
        };

    // prepare app
    client_app
        .insert_resource(runner_state)
        .add_systems(First, handle_command_incoming);

    // return the app
    Ok(client_app)
}

//-------------------------------------------------------------------------------------------------------------------
