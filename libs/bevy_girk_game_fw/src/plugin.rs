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
    if !world.contains_resource::<GameFWConfig>()
        { panic!("GameFWConfig is missing on startup!"); }
    if !world.contains_resource::<GameFWInitializer>()
        { panic!("GameFWInitializer is missing on startup!"); }
    if !world.contains_resource::<Receiver<ClientPacket>>()
        { panic!("Receiver<ClientPacket> is missing on startup!"); }
    if !world.contains_resource::<Sender<GamePacket>>()
        { panic!("Sender<GamePacket> is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before game begins.
fn poststartup_check(world: &World)
{
    if !world.contains_resource::<ClientMessageHandler>()
        { panic!("ClientMessageHandler is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Game framework startup plugin.
#[bevy_plugin]
pub fn GameFWStartupPlugin(app: &mut App)
{
    app.add_state::<GameFWMode>()
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
pub struct GameFWSet;

/// Private game fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameFWTickSetPrivate
{
    /// In schedule `PreUpdate`.
    FWStart,
    /// In schedule `Update`. Runs between [`GameFWTickSet::Start`] and [`GameFWTickSet::PreLogic`].
    FWHandleRequests,
    /// In schedule `PostUpdate`.
    FWEnd
}

/// Public game fw sets (exclusively ordered).
/// 
/// Game implementations should put game-related logic in these sets.
///
/// These sets are ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameFWTickSet
{
    Admin,
    Start,
    PreLogic,
    Logic,
    PostLogic,
    End
}

//-------------------------------------------------------------------------------------------------------------------

/// Game framework tick plugin. Depends on [`GameFWStartupPlugin`].
///
/// We treat a tick as a span of time in which events occur: |__stuff_happening__|. Our logic for handling the stuff that
/// happened in a span runs after the *end* of that span (after the span's tick counter increments). This means we think
/// in terms of a span (a tick) having 'elapsed'. The very first time our logic runs, no tick has elapsed yet. To handle
/// that, we imagine a 'virtual tick' that ends just before our first logic run.
///
/// Each tick is assigned one game mode, represented by [`GameFWMode`]. Game mode transitions occur **between** two ticks,
/// which means they occur after one tick's logic run and before the span of the tick with the new game mode begins.
///
/// In practice, the order of events in a tick is:
/// 1) Elapse a time span (tick).
/// 2) Increment 'elapsed tick' counter for the elapsed tick.
/// 3) Determine mode for the elapsed tick.
/// 4) Execute logic for the elapsed tick.
///
/// Transition logic can use the `OnEnter(GameFWMode::*)` and `OnExit(GameFWMode::*)` schedules.
///
/// Note that 'before the virtual tick' is assumed to be [`GameFWMode::Init`], which means the `OnEnter(GameFWMode::Init)`
/// schedule will always be invoked in the app's first update, even if the virtual tick is [`GameFWMode::Game`] (in which
/// case `OnExit(GameFWMode::Init)` and `OnEnter(GameFWMode::Game)` will also be invoked in the first update).
#[bevy_plugin]
pub fn GameFWTickPlugin(app: &mut App)
{
    app.configure_sets(Update,
            (
                GameFWTickSet::Admin,
                GameFWTickSet::Start,
                GameFWTickSetPrivate::FWHandleRequests,
                GameFWTickSet::PreLogic,
                GameFWTickSet::Logic,
                GameFWTickSet::PostLogic,
                GameFWTickSet::End,
            ).chain().in_set(GameFWSet)
        );

    // TICK FWSTART
    app.add_systems(PreUpdate,
            (
                // elapse the previous tick
                advance_game_fw_tick,
                // determine which game framework mode the previous tick was in and set it
                update_game_fw_mode,
                apply_state_transition::<GameFWMode>,
            ).chain().in_set(GameFWTickSetPrivate::FWStart)
        );

    // TICK ADMIN

    // TICK START

    // TICK FWHANDLEREQUESTS
    // note: we handle inputs after the game fw and game core have updated their ticks and modes (in their start sets)
    app.add_systems(Update,
            (
                handle_requests,
                refresh_game_init_progress,
            ).chain().in_set(GameFWTickSetPrivate::FWHandleRequests)
        );

    // TICK PRELOGIC

    // TICK LOGIC

    // TICK POSTLOGIC

    // TICK END

    // TICK FWEND
    app.add_systems(PostUpdate,
            (
                dispatch_messages_to_client,
            ).chain().in_set(GameFWTickSetPrivate::FWEnd)
        );


    // MISC

    // Respond to state transitions
    app.add_systems(OnEnter(GameFWMode::Init), notify_game_fw_mode_all)
        .add_systems(OnEnter(GameFWMode::Game), notify_game_fw_mode_all)
        .add_systems(OnEnter(GameFWMode::End),
            (
                notify_game_fw_mode_all,
                start_end_countdown,
            )
        )
        .add_systems(Last, try_terminate_app.run_if(in_state(GameFWMode::End)));
}

//-------------------------------------------------------------------------------------------------------------------

/// Main game framework plugin
#[bevy_plugin]
pub fn GameFWPlugin(app: &mut App)
{
    app.add_plugins(GameFWStartupPlugin)
        .add_plugins(GameFWTickPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
