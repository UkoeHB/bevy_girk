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
    /// runs when the client game is initialized
    Init,
    /// runs in game state 'play' (but not when initializing)
    Play,
    /// runs in game state 'game over' (but not when initializing)
    GameOver
}

//-------------------------------------------------------------------------------------------------------------------

/// Client startup plugin.
pub struct ClientCoreStartupPlugin;

impl Plugin for ClientCoreStartupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<ClientCoreState>()
            .add_systems(OnExit(ClientInstanceState::Client), build_precheck)
            .add_systems(OnEnter(ClientInstanceState::Game),
                (
                    setup_player_state,
                    setup_client_fw_reqs,
                )
            )
            .add_systems(OnExit(ClientInstanceState::Game), cleanup_at_game_end);
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
        // - connect to game and synchronize times
        app.configure_sets(
            Update,
            ClientSet::Init
                .run_if(in_state(ClientFwState::Init))
        );

        // Play systems.
        app.configure_sets(
            Update,
            ClientSet::Play
                .run_if(in_state(ClientFwState::Game))
                .run_if(in_state(ClientCoreState::Play))
        );

        // GameOver systems.
        app.configure_sets(
            Update,
            ClientSet::GameOver
                .run_if(in_state(ClientFwState::End))
                .run_if(in_state(ClientCoreState::GameOver))
        );

        // Admin
        app.add_systems(
            Update,
            (
                handle_player_inputs_init.in_set(ClientSet::Init),
                handle_player_inputs_play.in_set(ClientSet::Play),
                handle_player_inputs_gameover.in_set(ClientSet::GameOver),
            ).chain().in_set(GirkClientSet::Admin)
        );

        // Misc
        // Systems that should run when the client is fully initialized.
        app.add_systems(OnEnter(ClientInitState::Done), request_game_state);
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
