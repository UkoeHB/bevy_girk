//! Plugins for the core of a client.
//!
//! PRECONDITION: plugin dependencies
//! - bevy_replicon::core::ReplicationCorePlugin
//!
//! PRECONDITION: the following must be initialized by the client manager
//! - Res<ClickPlayerInitializer>
//! - Res<Receiver<PlayerInput>>
//!

//local shortcuts
use bevy_girk_client_fw::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts
use bevy::{prelude::*, app::PluginGroupBuilder};
use bevy_girk_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_framework_consistency(client_fw_config: &ClientFwConfig, player_initializer: &ClickPlayerInitializer)
{
    // check the client id
    if client_fw_config.client_id() != player_initializer.player_context.id()
        { panic!("client id mismatch with client framework!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before client startup.
fn build_precheck(world: &World)
{
    // check for expected resources
    if !world.contains_resource::<ClickPlayerInitializer>()
        { panic!("ClickPlayerInitializer is missing on startup!"); }
    if !world.contains_resource::<Receiver<PlayerInput>>()
        { panic!("Receiver<PlayerInput> is missing on startup!"); }

    // validate consistency between client framework and core
    if !world.contains_resource::<ClientFwConfig>()
        { panic!("ClientFwConfig is missing on startup!"); }

    check_client_framework_consistency(world.resource::<ClientFwConfig>(), world.resource::<ClickPlayerInitializer>());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Client core sets.
/// These sets are modal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientSet
{
    /// runs the first time the client is initialized
    InitStartup,
    /// runs if the client is reinitialized after exiting 'Init' mode
    InitReinit,
    /// runs main initialization logic
    InitCore,
    /// runs in game mode 'prep' (but not when initializing)
    Prep,
    /// runs in game mode 'play' (but not when initializing)
    Play,
    /// runs in game mode 'game over' (but not when initializing)
    GameOver
}

//-------------------------------------------------------------------------------------------------------------------

/// Client startup plugin.
pub struct ClientCoreStartupPlugin;

impl Plugin for ClientCoreStartupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<ClientCoreMode>()
            .add_systems(PreStartup,
                (
                    build_precheck,
                )
            )
            .add_systems(Startup,
                (
                    setup_player_state,
                    setup_game_output_handler,
                    setup_client_request_buffer,
                )
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client tick plugin.
/// Configures system sets and adds basic administrative systems.
pub struct ClientCoreTickPlugin;

impl Plugin for ClientCoreTickPlugin
{
    fn build(&self, app: &mut App)
    {
        // Init startup. (runs on startup)
        // - load assets
        app.configure_sets(Update,
                    ClientSet::InitStartup
                        .run_if(in_state(ClientFwMode::Init))
                        .run_if(in_state(ClientCoreMode::Init))
                );

        // Init reinitialize. (runs if client needs to be reinitialized during a game)
        // - lock display and show reinitialization progress
        app.configure_sets(Update,
                    ClientSet::InitReinit
                        .run_if(in_state(ClientFwMode::Init))  //framework is reinitializing
                        .run_if(not(in_state(ClientCoreMode::Init)))  //client core is not in init
                );

        // Init core. (always runs when framework is being initialized, regardless of client mode)
        // - connect to game and synchronize times
        app.configure_sets(Update,
                    ClientSet::InitCore
                        .run_if(in_state(ClientFwMode::Init))
                );

        // Prep systems.
        app.configure_sets(Update,
                    ClientSet::Prep
                        .run_if(in_state(ClientFwMode::Game))
                        .run_if(in_state(ClientCoreMode::Prep))
                );

        // Play systems.
        app.configure_sets(Update,
                    ClientSet::Play
                        .run_if(in_state(ClientFwMode::Game))
                        .run_if(in_state(ClientCoreMode::Play))
                );

        // GameOver systems.
        app.configure_sets(Update,
                    ClientSet::GameOver
                        .run_if(in_state(ClientFwMode::End))
                        .run_if(in_state(ClientCoreMode::GameOver))
                );

        // Admin
        app.add_systems(Update,
                    (
                        handle_player_inputs_init.in_set(ClientSet::InitStartup),
                        handle_player_inputs_prep.in_set(ClientSet::Prep),
                        handle_player_inputs_play.in_set(ClientSet::Play),
                        handle_player_inputs_gameover.in_set(ClientSet::GameOver),
                    ).chain().in_set(ClientFwSet::Admin)
                );

        // Misc
        // Systems that should run when the client is fully initialized.
        app.add_systems(OnEnter(ClientInitializationState::Done),
                (
                    request_game_mode,
                ).chain()
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ClientPlugins;

impl PluginGroup for ClientPlugins
{
    fn build(self) -> PluginGroupBuilder
    {
        PluginGroupBuilder::start::<Self>()
            .add(GameReplicationPlugin)
            .add(ClientCoreStartupPlugin)
            .add(ClientCoreTickPlugin)
    }
}

//-------------------------------------------------------------------------------------------------------------------
