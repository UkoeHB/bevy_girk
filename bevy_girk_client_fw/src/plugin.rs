//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_girk_utils::apply_state_transitions;
use bevy_replicon::prelude::Replicated;
use iyes_progress::prelude::ProgressPlugin;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_progress_plugin(app: &mut App)
{
    let progress_plugin = ProgressPlugin::<ClientInitState>::new()
        .with_state_transition(ClientInitState::InProgress, ClientInitState::Done)
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
        |current_state: Option<Res<State<ClientFwState>>>| -> bool
        {
            let Some(current_state) = current_state else { return false };
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
        app.init_state::<ClientInstanceState>()
            .enable_state_scoped_entities::<ClientInstanceState>()
            .add_sub_state::<ClientInitState>()
            .add_sub_state::<ClientFwState>()
            .register_required_components_with::<Replicated, StateScoped<ClientInstanceState>>(
                || StateScoped(ClientInstanceState::Game)
            )
            .add_systems(OnEnter(ClientFwState::Setup), (build_precheck, setup_client_fw_state).chain())
            .add_systems(OnEnter(ClientFwState::Connecting), startup_postcheck)
            .add_systems(OnExit(ClientInstanceState::Game), cleanup_client_fw_state);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Private client fw sets, these sandwich the public sets.
///
/// These sets are ordinal per-schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientFwSet
{
    /// In schedule `PreUpdate`.
    Start,
    /// System set for all client logic.
    /// - `PreUpdate`: Runs after [`Self::Start`].
    /// - `FixedUpdate`
    /// - `Update`
    /// - `PostUpdate`: Runs before [`Self::End`].
    ///
    /// Only runs in [`ClientInstanceState::Game`].
    /// Client implementations should put game-related logic in this set.
    Update,
    /// In schedule `PostUpdate`.
    End
}

/// Runs in [`Update`] when in [`ClientInitState::InProgress`].
///
/// This happens when initially connecting to the game, and whenever the client reconnects to the game.
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
                ClientFwSet::Start
                    .run_if(in_state(ClientInstanceState::Game))
            )
            .configure_sets(FixedUpdate, ClientFwSet::Update.run_if(in_state(ClientInstanceState::Game)))
            .configure_sets(PreUpdate, ClientFwSet::Update.run_if(in_state(ClientInstanceState::Game)).after(ClientFwSet::Start))
            .configure_sets(Update, ClientFwSet::Update.run_if(in_state(ClientInstanceState::Game)))
            .configure_sets(
                PostUpdate,
                ClientFwSet::Update
                    .run_if(in_state(ClientInstanceState::Game))
                    .before(iyes_progress::CheckProgressSet)
                    .before(ClientFwSet::End)
            );

        app.configure_sets(Update, ClientFwLoadingSet.run_if(in_state(ClientInitState::InProgress)));
        app.configure_sets(PostUpdate,
                ClientFwSet::End
                    .after(iyes_progress::CheckProgressSet)
                    .run_if(in_state(ClientInstanceState::Game))
            );

        // FWSTART
        app.add_systems(PreUpdate,
                (
                    // - We want connection-related state changes to be applied here since game state changes will be ignored if
                    //   initializing.
                    // todo: states dependency needs to be moved to OnEnter/OnExit since this is global
                    // - ClientInitState, ClientFwState
                    apply_state_transitions,
                    handle_game_incoming,
                    // The game may have caused a state change (will be ignored if in the middle of initializing).
                    // todo: states dependency needs to be moved to OnEnter/OnExit since this is global
                    // - ClientFwState
                    apply_state_transitions,
                ).chain().in_set(ClientFwSet::Start)
            );

        // FWEND
        app.add_systems(PostUpdate,
                (
                    // capture ClientInitState changes
                    apply_state_transitions,
                    update_initialization_cache.run_if(client_is_initializing()),
                    send_initialization_progress_report.run_if(in_state(ClientFwState::Init)),
                ).chain()
                    .in_set(ClientFwSet::End)
            );


        // MISC

        // Systems that should run when the client is fully initialized.
        app.add_systems(OnEnter(ClientInitState::Done), request_game_fw_state);
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
