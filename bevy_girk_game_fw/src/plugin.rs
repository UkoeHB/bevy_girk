//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

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

/// Private game fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameFwSetPrivate
{
    /// In schedule `PreUpdate`.
    FwStart,
    /// In schedule `Update`. Runs between [`GameFwSet::Start`] and [`GameFwSet::PreLogic`].
    FwHandleRequests,
    /// In schedule `PostUpdate`.
    FwEnd
}

/// Public game fw sets (exclusively ordered).
/// 
/// Game implementations should put game-related logic in these sets.
///
/// These sets are ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameFwSet
{
    Admin,
    Start,
    PreLogic,
    Logic,
    PostLogic,
    End
}

//-------------------------------------------------------------------------------------------------------------------

/// Game framework tick plugin. Depends on [`GameFwStartupPlugin`].
///
/// We treat a tick as a span of time in which events occur: |__stuff_happening__|. Our logic for handling the stuff that
/// happened in a span runs after all 'real-space' events of that span have occurred.
///
/// Each tick is assigned one game state, represented by [`GameFwState`].
/// We determine the game state for the next tick at the start of the current tick, using the state of the current tick.
///
/// Transition logic can use the `OnEnter(GameFwState::*)` and `OnExit(GameFwState::*)` schedules.
/// Keep in mind that [`GameFwTick`] will equal the *current* tick (i.e. the first tick of the on-entered state ) when
/// these schedules run.
///
/// In practice, since all our game logic is located at the end of a tick span in real time, the order of events in a
/// tick is:
/// 1) Elapse a time span (tick).
/// 2) Increment [`GameFwTick`] for the current tick.
/// 3) Determine state for the current tick.
/// 4) Execute logic for the current tick.
///
/// Tick 1's game state is always [`GameFwState::Init`].
pub struct GameFwTickPlugin;

impl Plugin for GameFwTickPlugin
{
    fn build(&self, app: &mut App)
    {
        app.configure_sets(Update,
                (
                    GameFwSet::Admin,
                    GameFwSet::Start,
                    GameFwSetPrivate::FwHandleRequests,
                    GameFwSet::PreLogic,
                    GameFwSet::Logic,
                    GameFwSet::PostLogic,
                    GameFwSet::End,
                ).chain()
            );

        // FWSTART
        app.add_systems(PreUpdate,
                (
                    // begin the current tick
                    advance_game_fw_tick,
                    update_game_fw_state,
                    // todo: states dependency needs to be moved to OnEnter/OnExit since this is global
                    // - GameFwState
                    |w: &mut World| { let _ = w.try_run_schedule(StateTransition); },
                ).chain().in_set(GameFwSetPrivate::FwStart)
            );

        // ADMIN

        // START

        // FWHANDLEREQUESTS
        // note: we handle inputs after the game fw and game core have updated their ticks and states (in their start sets)
        app.add_systems(Update,
                (
                    handle_requests,
                    refresh_game_init_progress,
                ).chain().in_set(GameFwSetPrivate::FwHandleRequests)
            );

        // PRELOGIC

        // LOGIC

        // POSTLOGIC

        // END

        // FWEND


        // MISC

        // Respond to state transitions
        app.add_systems(PostStartup, notify_game_fw_state_all)  // GameFwState::Init runs before startup systems
            .add_systems(OnEnter(GameFwState::Game), notify_game_fw_state_all)
            .add_systems(OnEnter(GameFwState::End),
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
