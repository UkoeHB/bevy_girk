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

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game startup.
fn build_precheck(world: &World)
{
    if !world.contains_resource::<ClickGameInitializer>()
        { panic!("ClickGameInitializer is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Game startup plugin.
pub struct GameStartupPlugin;

impl Plugin for GameStartupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<GameState>()
            .add_systems(PreStartup,
                (
                    build_precheck,
                ).chain()
            )
            .add_systems(Startup,
                (
                    setup_game_state,
                    setup_game_fw_reqs,
                ).chain()
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System sets that contain tick game logic. These don't run during initialization.
/// These sets are modal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet
{
    PostInit,
    Play,
    GameOver,
    End
}

//-------------------------------------------------------------------------------------------------------------------

/// Game tick plugin. Depends on [GameFwPlugin] and [GameStartupPlugin].
/// Configures system sets and adds basic administrative systems.
pub struct GameTickPlugin;

impl Plugin for GameTickPlugin
{
    fn build(&self, app: &mut App)
    {
        // GAME Tick systems (after initialization).
        app.configure_sets(Update,
                GameSet::PostInit
                    .run_if(not(in_state(GameFwState::Init)))
            );

        // GAME Play systems.
        app.configure_sets(Update,
                    GameSet::Play
                        .run_if(in_state(GameFwState::Game))
                        .run_if(in_state(GameState::Play))
                );

        // GAME GameOver systems.
        app.configure_sets(Update,
                    GameSet::GameOver
                        .run_if(in_state(GameState::GameOver))
                );


        // ADMIN
        app.add_systems(Update,
                (
                    // determine which game state the previous tick was in and set it
                    update_game_state.in_set(GameSet::PostInit),
                    //for GameState
                    {|w: &mut World| { let _ = w.try_run_schedule(StateTransition); }}.in_set(GameSet::PostInit),
                    // elapse the previous tick
                    advance_game_tick.in_set(GameSet::PostInit),
                    advance_play_tick.in_set(GameSet::Play),
                    advance_game_over_tick.in_set(GameSet::GameOver),
                ).chain()
            );


        // MISC

        // Respond to state transitions
        app.add_systems(PostStartup, notify_game_state_all);  // GameState::Init runs before startup systems
        app.add_systems(OnEnter(GameState::Play), notify_game_state_all);
        app.add_systems(OnEnter(GameState::GameOver),
                (
                    notify_game_state_all,
                    set_game_end_flag,
                )
                    .chain()
            );
    }
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
