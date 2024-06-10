//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::*;
use iyes_progress::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_progress_plugin(app: &mut App)
{
    let progress_plugin = ProgressPlugin::new(ClientInitializationState::InProgress)
        .continue_to(ClientInitializationState::Done)
        // we check progress in PostUpdate so initialization progress can be collected and networked immediately
        .check_progress_in(PostUpdate)
        .track_assets();

    app.add_plugins(progress_plugin);
}

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
    if !world.contains_resource::<GameMessageHandler>() { panic!("GameMessageHandler is missing post startup!"); }
    if !world.contains_resource::<ClientRequestType>()  { panic!("ClientRequestType is missing post startup!"); }
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
pub struct ClientFwStartupPlugin;

impl Plugin for ClientFwStartupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<ClientInitializationState>()
            .init_state::<ClientFwMode>()
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
}

//-------------------------------------------------------------------------------------------------------------------

/// Private client fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFwSetPrivate
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
pub enum ClientFwSet
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
pub struct ClientFwTickPlugin;

impl Plugin for ClientFwTickPlugin
{
    fn build(&self, app: &mut App)
    {
        add_progress_plugin(app);

        app.configure_sets(Update,
                (
                    ClientFwSet::Admin,
                    ClientFwSet::Start,
                    ClientFwSet::PreLogic,
                    ClientFwSet::Logic,
                    ClientFwSet::PostLogic,
                    ClientFwSet::End,
                ).chain()
            );
        app.configure_sets(Update,
                (
                    iyes_progress::TrackedProgressSet,
                    ClientFwLoadingSet,
                )
                    .run_if(in_state(ClientInitializationState::InProgress))
            );
        app.configure_sets(PostUpdate,
                ClientFwSetPrivate::FwEnd
                    .after(iyes_progress::CheckProgressSet)
            );

        // FWSTART
        app.add_systems(PreUpdate,
                (
                    handle_commands,
                    // The client may have been commanded to reinitialize.
                    apply_state_transition::<ClientInitializationState>,
                    // We want connection-related mode changes to be applied here since game mode changes will be ignored if
                    // initializing.
                    apply_state_transition::<ClientFwMode>,
                    handle_game_incoming,
                    // The game may have caused a mode change (will be ignored if in the middle of initializing).
                    apply_state_transition::<ClientFwMode>,
                ).chain().in_set(ClientFwSetPrivate::FwStart)
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
                ).chain()
                    .in_set(ClientFwSetPrivate::FwEnd)
            );


        // MISC

        // Handle just disconnected.
        app.add_systems(OnEnter(ClientFwMode::Connecting), reset_init_progress);

        // Systems that should run when the client is fully initialized.
        app.add_systems(OnEnter(ClientInitializationState::Done), request_game_fw_mode);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ClientFwPlugin;

impl Plugin for ClientFwPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ClientFwStartupPlugin)
            .add_plugins(ClientFwTickPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
