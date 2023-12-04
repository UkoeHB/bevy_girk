//! Plugins for core game logic.
//!
//! PRECONDITION: plugin dependencies
//! - bevy_replicon::core::ReplicationCorePlugin
//!
//! PRECONDITION: the following must be initialized by the user
//! - Res<ClickGameInitializer>
//!
//! INTERFACE: for client core
//! - plugin GameReplicationPlugin must be added to the client core app
//!

//local shortcuts
use crate::click_game_integration::click_game::*;
use bevy_girk_game_fw::*;

//third-party shortcuts
use bevy::{prelude::*, app::PluginGroupBuilder};
use bevy_fn_plugin::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game startup.
fn prestartup_check(world: &World)
{
    if !world.contains_resource::<ClickGameInitializer>()
        { panic!("ClickGameInitializer is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Game startup plugin.
#[bevy_plugin]
pub fn GameStartupPlugin(app: &mut App)
{
    app.add_state::<GameMode>()
        .add_systems(PreStartup,
            (
                prestartup_check,
            ).chain()
        )
        .add_systems(Startup,
            (
                setup_game_state,
                setup_game_input_handler,
            ).chain()
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// System sets that contain tick game logic. These don't run during initialization.
/// These sets are modal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet
{
    PostInit,
    Prep,
    Play,
    GameOver,
    End
}

//-------------------------------------------------------------------------------------------------------------------

/// Game tick plugin. Depends on [GameFWPlugin] and [GameStartupPlugin].
/// Configures system sets and adds basic administrative systems.
#[bevy_plugin]
pub fn GameTickPlugin(app: &mut App)
{
    // GAME Tick systems (after initialization).
    app.configure_sets(Update,
            GameSet::PostInit
                .run_if(not(in_state(GameFWMode::Init)))
        );

    // GAME Prep systems.
    app.configure_sets(Update,
                GameSet::Prep
                    .run_if(in_state(GameFWMode::Game))
                    .run_if(in_state(GameMode::Prep))
            );

    // GAME Play systems.
    app.configure_sets(Update,
                GameSet::Play
                    .run_if(in_state(GameFWMode::Game))
                    .run_if(in_state(GameMode::Play))
            );

    // GAME GameOver systems.
    //todo: this will only run in the short delay between entering 'game over' and the GameFWMode moving to 'End'
    app.configure_sets(Update,
                GameSet::GameOver
                    .run_if(in_state(GameFWMode::Game))
                    .run_if(in_state(GameMode::GameOver))
            );


    // ADMIN
    app.add_systems(Update,
            (
                // determine which game mode the previous tick was in and set it
                update_game_mode.in_set(GameSet::PostInit),
                apply_state_transition::<GameMode>.in_set(GameSet::PostInit),
                // elapse the previous tick
                advance_game_tick.in_set(GameSet::PostInit),
                advance_prep_tick.in_set(GameSet::Prep),
                advance_play_tick.in_set(GameSet::Play),
                advance_game_over_tick.in_set(GameSet::GameOver),
            ).chain().in_set(GameFWTickSet::Admin)
        );


    // MISC

    // Respond to state transitions
    app.add_systems(OnEnter(GameMode::Init), notify_game_mode_all);
    app.add_systems(OnEnter(GameMode::Prep), notify_game_mode_all);
    app.add_systems(OnEnter(GameMode::Play), notify_game_mode_all);
    app.add_systems(OnEnter(GameMode::GameOver),
            (
                notify_game_mode_all,
                set_game_end_flag,
            )
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Main game plugins
pub struct GamePlugins;

impl PluginGroup for GamePlugins
{
    fn build(self) -> PluginGroupBuilder
    {
        PluginGroupBuilder::start::<Self>()
            .add(GameReplicationPlugin)
            .add(GameStartupPlugin)
            .add(GameTickPlugin)
    }
}

//-------------------------------------------------------------------------------------------------------------------
