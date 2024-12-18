//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::apply_state_transitions;
use iyes_progress::prelude::ProgressPlugin;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_progress_plugin(app: &mut App)
{
    let progress_plugin = ProgressPlugin::<ClientInitializationState>::new()
        .with_state_transition(ClientInitializationState::InProgress, ClientInitializationState::Done)
        // we check progress in PostUpdate so initialization progress can be collected and networked immediately
        .check_progress_in(PostUpdate)
        .with_asset_tracking();

    app.add_plugins(progress_plugin);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist before client startup.
fn build_precheck(world: &World)
{
    if !world.contains_resource::<ClientFwConfig>() { panic!("ClientFwConfig is missing on startup!"); }

    if !world.contains_resource::<Time>() { panic!("bevy::Time is missing on startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Validate resources that should exist after client startup.
fn startup_postcheck(world: &World)
{
    if !world.contains_resource::<GameMessageHandler>() { panic!("GameMessageHandler is missing post startup!"); }
    if !world.contains_resource::<ClientRequestType>()  { panic!("ClientRequestType is missing post startup!"); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns a Bevy run condition indicating if the client is initializing.
///
/// The run condition returns true when in state [`ClientFwState::Setup`], [`ClientFwState::Connecting`],
/// [`ClientFwState::Syncing`], or [`ClientFwState::Init`].
pub fn client_is_initializing() -> impl Condition<()>
{
    IntoSystem::into_system(
        |current_state: Res<State<ClientFwState>>| -> bool
        {
            match **current_state
            {
                ClientFwState::Setup |
                ClientFwState::Connecting |
                ClientFwState::Syncing |
                ClientFwState::Init => true,
                _ => false,
            }
        }
    )
}

//-------------------------------------------------------------------------------------------------------------------

/// Client startup plugin.
pub struct ClientFwStartupPlugin;

impl Plugin for ClientFwStartupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<ClientInitializationState>()
            .init_state::<ClientFwState>()
            .add_systems(OnExit(ClientFwState::Setup), (build_precheck, setup_client_fw_state).chain())
            .add_systems(OnEnter(ClientFwState::Connecting), startup_postcheck)
            .add_systems(OnExit(ClientInstanceState::Game), cleanup_client_fw_state);
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

        app.configure_sets(PreUpdate,
                ClientFwSetPrivate::FwStart
                    .run_if(in_state(ClientInstanceState::Game))
            );
        app.configure_sets(Update,
                (
                    ClientFwSet::Admin,
                    ClientFwSet::Start,
                    ClientFwSet::PreLogic,
                    ClientFwSet::Logic,
                    ClientFwSet::PostLogic,
                    ClientFwSet::End,
                )
                    .chain()
                    .run_if(in_state(ClientInstanceState::Game))
            );
        app.configure_sets(Update, ClientFwLoadingSet.run_if(in_state(ClientInitializationState::InProgress)));
        app.configure_sets(PostUpdate,
                ClientFwSetPrivate::FwEnd
                    .after(iyes_progress::CheckProgressSet)
                    .run_if(in_state(ClientInstanceState::Game))
            );

        // FWSTART
        app.add_systems(PreUpdate,
                (
                    prep_replicated_entities,
                    // - We want connection-related state changes to be applied here since game state changes will be ignored if
                    //   initializing.
                    // todo: states dependency needs to be moved to OnEnter/OnExit since this is global
                    // - ClientInitializationState, ClientFwState
                    apply_state_transitions,
                    handle_game_incoming,
                    // The game may have caused a state change (will be ignored if in the middle of initializing).
                    // todo: states dependency needs to be moved to OnEnter/OnExit since this is global
                    // - ClientFwState
                    apply_state_transitions,
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
                    // capture ClientInitializationState changes
                    apply_state_transitions,
                    update_initialization_cache.run_if(client_is_initializing()),
                    send_initialization_progress_report.run_if(in_state(ClientFwState::Init)),
                ).chain()
                    .in_set(ClientFwSetPrivate::FwEnd)
            );


        // MISC

        // Systems that should run when the client is fully initialized.
        app.add_systems(OnEnter(ClientInitializationState::Done), request_game_fw_state);
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
