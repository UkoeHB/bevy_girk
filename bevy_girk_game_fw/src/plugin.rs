//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game startup.
fn prestartup_check(world: &World)
{
    if !world.contains_resource::<GameFwClients>() { panic!("GameFwClients is missing on startup!"); }
    if !world.contains_resource::<GameFwConfig>()  { panic!("GameFwConfig is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game begins.
fn poststartup_check(world: &World)
{
    if !world.contains_resource::<ClientRequestHandler>() { panic!("ClientRequestHandler is missing post startup!"); }
    if !world.contains_resource::<GameMessageType>()      { panic!("GameMessageType is missing post startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Game framework startup plugin.
#[bevy_plugin]
pub fn GameFwStartupPlugin(app: &mut App)
{
    app.init_state::<GameFwMode>()
        .add_systems(PreStartup,
            (
                prestartup_check,
            ).chain()
        )
        .add_systems(Startup,
            (
                setup_game_fw_state,
            ).chain()
        )
        .add_systems(PostStartup,
            (
                poststartup_check,
            ).chain()
        );
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
/// Each tick is assigned one game mode, represented by [`GameFwMode`].
/// We determine the game mode for the next tick at the start of the current tick, using the state of the current tick.
///
/// Transition logic can use the `OnEnter(GameFwMode::*)` and `OnExit(GameFwMode::*)` schedules.
/// Keep in mind that [`GameFwTick`] will equal the *current* tick (i.e. the first tick of the on-entered mode ) when
/// these schedules run.
///
/// In practice, since all our game logic is located at the end of a tick span in real time, the order of events in a
/// tick is:
/// 1) Elapse a time span (tick).
/// 2) Increment [`GameFwTick`] for the current tick.
/// 3) Determine mode for the current tick.
/// 4) Execute logic for the current tick.
///
/// Tick 1's game mode is always [`GameFwMode::Init`].
#[bevy_plugin]
pub fn GameFwTickPlugin(app: &mut App)
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
                update_game_fw_mode,
                apply_state_transition::<GameFwMode>,
            ).chain().in_set(GameFwSetPrivate::FwStart)
        );

    // ADMIN

    // START

    // FWHANDLEREQUESTS
    // note: we handle inputs after the game fw and game core have updated their ticks and modes (in their start sets)
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
    app.add_systems(OnEnter(GameFwMode::Init), notify_game_fw_mode_all)
        .add_systems(OnEnter(GameFwMode::Game), notify_game_fw_mode_all)
        .add_systems(OnEnter(GameFwMode::End),
            (
                notify_game_fw_mode_all,
                start_end_countdown,
            )
        )
        .add_systems(Last, try_exit_app.run_if(in_state(GameFwMode::End)));
}

//-------------------------------------------------------------------------------------------------------------------

/// Main game framework plugin.
///
/// Requires `TimePlugin` (`bevy`), `ClientCache` (`bevy_replicon`), and `VisibilityAttributesPlugin`
/// (`bevy_replicon_attributes`).
#[bevy_plugin]
pub fn GameFwPlugin(app: &mut App)
{
    app.add_plugins(GameFwStartupPlugin)
        .add_plugins(GameFwTickPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
