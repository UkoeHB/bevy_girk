//local shortcuts
use crate::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_renet::{renet::RenetClient, RenetReceive};

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn request_new_connect_token(runner: Res<ClientRunnerState>)
{
    tracing::info!("requesting new connect token");
    let _ = runner.report_sender.send(ClientInstanceReport::RequestConnectToken);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn on_reconnect_inverval(
    reconnect_inverval: u32
) -> impl FnMut(Local<bool>, Option<Res<RenetClient>>, Res<Time>) -> bool
{
    let mut timer = Timer::new(Duration::from_secs(reconnect_inverval as u64), TimerMode::Repeating);
    move |mut last_connected: Local<bool>, client: Option<Res<RenetClient>>, time: Res<Time>|
    {
        // detect just connected
        let disconnected = client.map(|client| client.is_disconnected()).unwrap_or(true);
        let just_disconnected = *last_connected && disconnected;
        *last_connected = !disconnected;

        // abort if not disconnected
        if !disconnected { return false; }

        // reset timer
        // - we reconnect when just disconnected
        if just_disconnected
        {
            timer.reset();
            return true;
        }

        // increment timer (we are connected and already past 'just connected')
        timer.tick(time.delta());
        timer.just_finished()
    }
}

//-------------------------------------------------------------------------------------------------------------------
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
/// - Includes logic to request new connect tokens from the instance owner when disconnected.
pub fn client_instance_setup(
    client_factory   : &mut ClientFactory,
    token            : ServerConnectToken,
    start_info       : GameStartInfo,
    config           : ClientInstanceConfig,
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
        .add_systems(First, handle_command_incoming)
        .add_systems(PreUpdate,
            request_new_connect_token
                .run_if(on_reconnect_inverval(config.reconnect_interval_secs))
                .after(RenetReceive)
        );

    // return the app
    Ok(client_app)
}

//-------------------------------------------------------------------------------------------------------------------
