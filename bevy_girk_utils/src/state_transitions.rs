//local shortcuts

//third-party shortcuts
use bevy::{prelude::*, state::state::FreelyMutableState};

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------

pub fn apply_state_transitions(w: &mut World)
{
    let _ = w.try_run_schedule(StateTransition);
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets state `state` and runs the `StateTransition` schedule to apply it to the world.
pub fn set_and_apply_state<S: FreelyMutableState>(world: &mut World, state: S)
{
    world.resource_mut::<NextState<S>>().set(state);
    if let Err(err) = world.try_run_schedule(StateTransition) {
        tracing::error!("failed running schedule StateTransition (err={err:?}) after setting state {}; state \
            transitions cannot be executed recursively, or maybe you're missing the States plugin",
            std::any::type_name::<S>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
