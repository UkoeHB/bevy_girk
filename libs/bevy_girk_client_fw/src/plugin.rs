//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;

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
    if !world.contains_resource::<Receiver<GamePacket>>()      { panic!("Receiver<GamePacket> is missing on startup!"); }
    if !world.contains_resource::<Sender<ClientPacket>>()      { panic!("Sender<ClientPacket> is missing on startup!"); }
    if !world.contains_resource::<Receiver<ClientFwCommand>>() { panic!("Receiver<ClientFwCommand> is missing on startup!"); }

    if !world.contains_resource::<Time>() { panic!("bevy::Time is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist after client startup.
fn poststartup_check(world: &World)
{
    if !world.contains_resource::<GameMessageHandler>() { panic!("GameMessageHandler is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
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

/// Runs when the client state is 'initializing in progress'.
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
                handle_game_incoming,
                apply_state_transition::<ClientInitializationState>,  //the client may have been commanded to reinitialize
                apply_state_transition::<ClientFwMode>,
            ).chain().in_set(ClientFwTickSetPrivate::FwStart)
        );

    // START

    // PRELOGIC

    // LOGIC

    // POSTLOGIC

    // END

    // FWEND
    app.add_systems(PostUpdate,
            (
                apply_state_transition::<ClientInitializationState>,
                update_initialization_cache.run_if(in_state(ClientFwMode::Init)),
                send_initialization_progress_report.run_if(in_state(ClientFwMode::Init)),
                dispatch_client_packets,
            ).chain()
                .in_set(ClientFwTickSetPrivate::FwEnd)
                .after(iyes_progress::CheckProgressSet)
        );


    // MISC

    // Systems that should run when the client is fully initialized.
    app.add_systems(OnEnter(ClientInitializationState::Done),
            (
                request_game_fw_mode,
            ).chain()
        );
}

//-------------------------------------------------------------------------------------------------------------------

#[bevy_plugin]
pub fn ClientFwPlugin(app: &mut App)
{
    app.add_plugins(ClientFwStartupPlugin)
        .add_plugins(ClientFwTickPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
