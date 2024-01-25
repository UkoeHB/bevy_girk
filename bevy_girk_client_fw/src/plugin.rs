//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use bevy_kot_utils::*;
use iyes_progress::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before client startup.
fn prestartup_check(world: &World)
{
    if !world.contains_resource::<ClientFwConfig>()            { panic!("ClientFwConfig is missing on startup!"); }
    if !world.contains_resource::<Receiver<ClientFwCommand>>() { panic!("Receiver<ClientFwCommand> is missing on startup!"); }

    if !world.contains_resource::<Time>() { panic!("bevy::Time is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist after client startup.
fn poststartup_check(world: &World)
{
    if !world.contains_resource::<GameMessageHandler>()  { panic!("GameMessageHandler is missing post startup!"); }
    if !world.contains_resource::<ClientRequestBuffer>() { panic!("ClientRequestBuffer is missing post startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns a Bevy run condition indicating if the client is initializing.
///
/// The run condition returns true when in state [`ClientFwMod::Connecting`], [`ClientFwMod::Syncing`],
/// and [`ClientFwMod::Init`]
pub fn client_is_initializing() -> impl FnMut(Res<State<ClientFwMode>>) -> bool + Clone
{
    |current_state: Res<State<ClientFwMode>>| -> bool
    {
        match **current_state
        {
            ClientFwMode::Connecting |
            ClientFwMode::Syncing |
            ClientFwMode::Init => true,
            _ => false,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client startup plugin.
#[bevy_plugin]
pub fn ClientFwStartupPlugin(app: &mut App)
{
    app.add_state::<ClientInitializationState>()
        .add_state::<ClientFwMode>()
        .add_systems(PreStartup,
            (
                prestartup_check,
            ).chain()
        )
        .add_systems(Startup,
            (
                setup_client_fw_state,
            ).chain()
        )
        .add_systems(PostStartup,
            (
                poststartup_check,
            ).chain()
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Umbrella set for client fw sets.
///
/// This set is ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientFwSet;

/// Private client fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFwTickSetPrivate
{
    /// In schedule `PreUpdate`.
    FwStart,
    /// In schedule `PostUpdate`.
    FwEnd
}

/// Public client fw sets.
///
/// Client implementations should put game-related logic in these sets.
///
/// These sets are ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFwTickSet
{
    Admin,
    Start,
    PreLogic,
    Logic,
    PostLogic,
    End
}

/// Runs when in [`ClientInitializationState::InProgress`].
///
/// This happens when initially connecting to the game, and whenever the client reconnects to the game.
///
/// This set is modal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientFwLoadingSet;

//-------------------------------------------------------------------------------------------------------------------

/// Client tick plugin.
#[bevy_plugin]
pub fn ClientFwTickPlugin(app: &mut App)
{
    app.add_plugins(
            ProgressPlugin::new(ClientInitializationState::InProgress)
                .continue_to(ClientInitializationState::Done)
                // we check progress in PostUpdate so initialization progress can be collected and networked immediately
                .check_progress_in(PostUpdate)
        );

    app.configure_sets(Update,
            (
                ClientFwTickSet::Admin,
                ClientFwTickSet::Start,
                ClientFwTickSet::PreLogic,
                ClientFwTickSet::Logic,
                ClientFwTickSet::PostLogic,
                ClientFwTickSet::End,
            ).chain().in_set(ClientFwSet)
        );
    app.configure_sets(Update,
            ClientFwLoadingSet
                .run_if(in_state(ClientInitializationState::InProgress))
                .in_set(ClientFwSet)
        );

    // FWSTART
    app.add_systems(PreUpdate,
            (
                reset_client_request_buffer,
                handle_commands,
                // The client may have been commanded to reinitialize.
                apply_state_transition::<ClientInitializationState>,
                // We want connection-related mode changes to be applied here since game mode changes will be ignored if
                // initializing.
                apply_state_transition::<ClientFwMode>,
                handle_game_incoming,
                // The game may have caused a mode change (will be ignored if in the middle of initializing).
                apply_state_transition::<ClientFwMode>,
            ).chain().in_set(ClientFwTickSetPrivate::FwStart)
        );

    // ADMIN

    // START

    // PRELOGIC

    // LOGIC

    // POSTLOGIC

    // END

    // FWEND
    app.add_systems(PostUpdate,
            (
                apply_state_transition::<ClientInitializationState>,
                update_initialization_cache.run_if(client_is_initializing()),
                send_initialization_progress_report.run_if(in_state(ClientFwMode::Init)),
                dispatch_client_packets,
            ).chain()
                .in_set(ClientFwTickSetPrivate::FwEnd)
                .after(iyes_progress::CheckProgressSet)
        );


    // MISC

    // Handle just disconnected.
    app.add_systems(OnEnter(ClientFwMode::Connecting), reset_init_progress);

    // Systems that should run when the client is fully initialized.
    app.add_systems(OnEnter(ClientInitializationState::Done), request_game_fw_mode);
}

//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
pub fn ClientFwPlugin(app: &mut App)
{
    app.add_plugins(ClientFwStartupPlugin)
        .add_plugins(ClientFwTickPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
