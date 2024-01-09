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
    if !world.contains_resource::<ClientFWConfig>()
        { panic!("ClientFWConfig is missing on startup!"); }
    if !world.contains_resource::<Receiver<GamePacket>>()
        { panic!("Receiver<GamePacket> is missing on startup!"); }
    if !world.contains_resource::<Sender<ClientPacket>>()
        { panic!("Sender<ClientPacket> is missing on startup!"); }
    if !world.contains_resource::<Receiver<ClientFWCommand>>()
        { panic!("Receiver<ClientFWCommand> is missing on startup!"); }

    if !world.contains_resource::<Time>()
        { panic!("bevy::Time is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist after client startup.
fn poststartup_check(world: &World)
{
    if !world.contains_resource::<GameMessageHandler>()
        { panic!("GameMessageHandler is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Client startup plugin.
#[bevy_plugin]
pub fn ClientFWStartupPlugin(app: &mut App)
{
    app.add_state::<ClientInitializationState>()
        .add_state::<ClientFWMode>()
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
pub struct ClientFWSet;

/// Private client fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFWTickSetPrivate
{
    /// In schedule `PreUpdate`.
    FWStart,
    /// In schedule `PostUpdate`.
    FWEnd
}

/// Public client fw sets.
///
/// Client implementations should put game-related logic in these sets.
///
/// These sets are ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFWTickSet
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
pub struct ClientFWLoadingSet;

//-------------------------------------------------------------------------------------------------------------------

/// Client tick plugin.
#[bevy_plugin]
pub fn ClientFWTickPlugin(app: &mut App)
{
    app.add_plugins(
            ProgressPlugin::new(ClientInitializationState::InProgress)
                .continue_to(ClientInitializationState::Done)
                // we check progress in PostUpdate so initialization progress can be collected and networked immediately
                .check_progress_in(PostUpdate)
        );

    app.configure_sets(Update,
            (
                ClientFWTickSet::Admin,
                ClientFWTickSet::Start,
                ClientFWTickSet::PreLogic,
                ClientFWTickSet::Logic,
                ClientFWTickSet::PostLogic,
                ClientFWTickSet::End,
            ).chain().in_set(ClientFWSet)
        );
    app.configure_sets(Update,
            ClientFWLoadingSet
                .run_if(in_state(ClientInitializationState::InProgress))
                .in_set(ClientFWSet)
        );

    // FWSTART
    app.add_systems(PreUpdate,
            (
                reset_client_request_buffer,
                handle_commands,
                handle_game_incoming,
                apply_state_transition::<ClientInitializationState>,  //the client may have been commanded to reinitialize
                apply_state_transition::<ClientFWMode>,
            ).chain().in_set(ClientFWTickSetPrivate::FWStart)
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
                update_initialization_cache.run_if(in_state(ClientFWMode::Init)),
                send_initialization_progress_report.run_if(in_state(ClientFWMode::Init)),
                dispatch_client_packets,
            ).chain()
                .in_set(ClientFWTickSetPrivate::FWEnd)
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
pub fn ClientFWPlugin(app: &mut App)
{
    app.add_plugins(ClientFWStartupPlugin)
        .add_plugins(ClientFWTickPlugin);
}

//-------------------------------------------------------------------------------------------------------------------
