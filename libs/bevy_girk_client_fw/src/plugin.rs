//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;
use iyes_progress::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before client startup.
fn prestartup_check(world: &World)
{
    if !world.contains_resource::<ClientFWConfig>()
        { panic!("ClientFWConfig is missing on startup!"); }
    if !world.contains_resource::<MessageReceiver<GamePacket>>()
        { panic!("MessageReceiver<GamePacket> is missing on startup!"); }
    if !world.contains_resource::<MessageSender<ClientPacket>>()
        { panic!("MessageSender<ClientPacket> is missing on startup!"); }
    if !world.contains_resource::<MessageReceiver<ClientFWCommand>>()
        { panic!("MessageReceiver<ClientFWCommand> is missing on startup!"); }

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

/// Manually check progress because iyes_progress does this check in the `Last` schedule...
fn manually_check_progress<S: States>(next_state: S) -> impl FnMut(Res<ProgressCounter>, ResMut<NextState<S>>)
{
    move |progress, mut state|
    {
        let progress_complete = progress.progress_complete();
        if progress_complete.done >= progress_complete.total {
            state.set(next_state.clone());
        }
    }
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
/// This set is ordinal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientFWSet;

/// Private client fw sets, these sandwich the public sets.
/// These sets are ordinal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFWTickSetPrivate
{
    /// PreUpdate
    FWStart,
    /// PostUpdate
    FWEnd
}

/// Public client fw sets (exclusively ordered). Client implementations should put game-related logic in these sets.
/// These sets are ordinal.
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

/// Runs when the client state is 'initializing in progress'. This happens when initially connecting to the game,
/// and whenever the client reconnects to the game.
/// This set is modal.
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
    app.configure_set(Update,
            ClientFWLoadingSet
                .run_if(in_state(ClientInitializationState::InProgress))
                .in_set(ClientFWSet)
        );

    // FWSTART
    app.add_systems(PreUpdate,
            (
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
                manually_check_progress::<ClientInitializationState>(ClientInitializationState::Done)
                    .run_if(in_state(ClientInitializationState::InProgress)),
                apply_state_transition::<ClientInitializationState>,
                update_initialization_cache.run_if(in_state(ClientFWMode::Init)),
                send_initialization_progress_report.run_if(in_state(ClientFWMode::Init)),
                dispatch_client_packets,
            ).chain().in_set(ClientFWTickSetPrivate::FWEnd)
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
