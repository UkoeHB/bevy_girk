//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::apply_state_transitions;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game startup.
fn build_precheck(world: &World)
{
    if !world.contains_resource::<GameFwClients>() { panic!("GameFwClients is missing on startup!"); }
    if !world.contains_resource::<GameFwConfig>()  { panic!("GameFwConfig is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game begins.
fn startup_postcheck(world: &World)
{
    if !world.contains_resource::<ClientRequestHandler>() { panic!("ClientRequestHandler is missing post startup!"); }
    if !world.contains_resource::<GameMessageType>()      { panic!("GameMessageType is missing post startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Game framework startup plugin.
pub struct GameFwStartupPlugin;

impl Plugin for GameFwStartupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<GameFwState>()
            .enable_state_scoped_entities::<GameFwState>()
            .add_systems(PreStartup,
                (
                    build_precheck,
                ).chain()
            )
            .add_systems(Startup,
                (
                    setup_game_fw_state,
                ).chain()
            )
            .add_systems(PostStartup,
                (
                    startup_postcheck,
                ).chain()
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System sets for the girk game framework.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameFwSet
{
    /// In schedule `PreUpdate`.
    Start,
    /// In schedule `PostUpdate`.
    End
}

//-------------------------------------------------------------------------------------------------------------------

/// Game framework tick plugin. Depends on [`GameFwStartupPlugin`].
pub struct GameFwTickPlugin;

impl Plugin for GameFwTickPlugin
{
    fn build(&self, app: &mut App)
    {
        // FWSTART
        app.add_systems(PreUpdate,
            (
                // handle requests that showed up before this tick started (i.e. at the end of the previous tick)
                handle_requests,
                refresh_game_init_progress,
                // begin the current tick
                advance_game_fw_tick,
                update_game_fw_state,
                // todo: states dependency needs to be moved to OnEnter/OnExit since this is global
                // - GameFwState
                apply_state_transitions,
            ).chain().in_set(GameFwSet::Start)
        );

        // FWEND

        // MISC

        // Respond to state transitions
        app.add_systems(PostStartup, notify_game_fw_state_all)  // GameFwState::Init runs before startup systems
            .add_systems(OnEnter(GameFwState::Game), notify_game_fw_state_all)
            .add_systems(
                OnEnter(GameFwState::End),
                (
                    notify_game_fw_state_all,
                    start_end_countdown,
                )
            )
            .add_systems(Last, try_exit_app.run_if(in_state(GameFwState::End)));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Main game framework plugin.
///
/// Requires `TimePlugin` (`bevy`), `ClientCache` (`bevy_replicon`), and `VisibilityAttributesPlugin`
/// (`bevy_replicon_attributes`).
pub struct GameFwPlugin;

impl Plugin for GameFwPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(GameFwStartupPlugin)
            .add_plugins(GameFwTickPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
