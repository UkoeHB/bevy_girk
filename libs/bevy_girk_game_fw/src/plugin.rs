//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game startup.
fn prestartup_check(world: &World)
{
    if !world.contains_resource::<GameFwConfig>()           { panic!("GameFwConfig is missing on startup!"); }
    if !world.contains_resource::<GameFwInitializer>()      { panic!("GameFwInitializer is missing on startup!"); }
    if !world.contains_resource::<Receiver<ClientPacket>>() { panic!("Receiver<ClientPacket> is missing on startup!"); }
    if !world.contains_resource::<Sender<GamePacket>>()     { panic!("Sender<GamePacket> is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game begins.
fn poststartup_check(world: &World)
{
    if !world.contains_resource::<ClientRequestHandler>() { panic!("ClientRequestHandler is missing on startup!"); }
    if !world.contains_resource::<GameMessageBuffer>()    { panic!("GameMessageBuffer is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Game framework startup plugin.
#[bevy_plugin]
pub fn GameFwStartupPlugin(app: &mut App)
{
    app.add_state::<GameFwMode>()
        .add_systems(PreStartup,
            (
                prestartup_check,
                //todo: set up basic replication rooms (client rooms, global room, ...?)
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

/// Umbrella set for game fw sets.
///
/// This set is ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameFwSet;

/// Private game fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameFwTickSetPrivate
{
    /// In schedule `PreUpdate`.
    FwStart,
    /// In schedule `Update`. Runs between [`GameFwTickSet::Start`] and [`GameFwTickSet::PreLogic`].
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
pub enum GameFwTickSet
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
/// happened in a span runs after the *end* of that span (after the span's tick counter increments). This means we think
/// in terms of a span (a tick) having 'elapsed'. The very first time our logic runs, no tick has elapsed yet. To handle
/// that, we imagine a 'virtual tick' that ends just before our first logic run.
///
/// Each tick is assigned one game mode, represented by [`GameFwMode`]. Game mode transitions occur **between** two ticks,
/// which means they occur after one tick's logic run and before the span of the tick with the new game mode begins.
///
/// In practice, the order of events in a tick is:
/// 1) Elapse a time span (tick).
/// 2) Increment 'elapsed tick' counter for the elapsed tick.
/// 3) Determine mode for the elapsed tick.
/// 4) Execute logic for the elapsed tick.
///
/// Transition logic can use the `OnEnter(GameFwMode::*)` and `OnExit(GameFwMode::*)` schedules.
///
/// Note that 'before the virtual tick' is assumed to be [`GameFwMode::Init`], which means the `OnEnter(GameFwMode::Init)`
/// schedule will always be invoked in the app's first update, even if the virtual tick is [`GameFwMode::Game`] (in which
/// case `OnExit(GameFwMode::Init)` and `OnEnter(GameFwMode::Game)` will also be invoked in the first update).
#[bevy_plugin]
pub fn GameFwTickPlugin(app: &mut App)
{
    app.configure_sets(Update,
            (
                GameFwTickSet::Admin,
                GameFwTickSet::Start,
                GameFwTickSetPrivate::FwHandleRequests,
                GameFwTickSet::PreLogic,
                GameFwTickSet::Logic,
                GameFwTickSet::PostLogic,
                GameFwTickSet::End,
            ).chain().in_set(GameFwSet)
        );

    // FWSTART
    app.add_systems(PreUpdate,
            (
                // elapse the previous tick
                advance_game_fw_tick,
                reset_game_message_buffer,
                // determine which game framework mode the previous tick was in and set it
                update_game_fw_mode,
                apply_state_transition::<GameFwMode>,
            ).chain().in_set(GameFwTickSetPrivate::FwStart)
        );

    // ADMIN

    // START

    // FWHANDLEREQUESTS
    // note: we handle inputs after the game fw and game core have updated their ticks and modes (in their start sets)
    app.add_systems(Update,
            (
                handle_requests,
                refresh_game_init_progress,
            ).chain().in_set(GameFwTickSetPrivate::FwHandleRequests)
        );

    // PRELOGIC

    // LOGIC

    // POSTLOGIC

    // END

    // FWEND
    app.add_systems(PostUpdate,
            (
                dispatch_messages_to_client,
            ).chain().in_set(GameFwTickSetPrivate::FwEnd)
        );


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

/// Main game framework plugin
#[bevy_plugin]
pub fn GameFwPlugin(app: &mut App)
{
    app.add_plugins(GameFwStartupPlugin)
        .add_plugins(GameFwTickPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
