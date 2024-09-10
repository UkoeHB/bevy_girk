//local shortcuts

//third-party shortcuts
use bevy::{prelude::*, state::state::FreelyMutableState};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Returns a run condition that checks if state `state` was entered since the condition system lat ran.
pub fn just_entered_state<S: FreelyMutableState>(state: S) -> impl Condition<()>
{
    IntoSystem::into_system(
        move |mut last: Local<Option<S>>, current: Option<Res<State<S>>>| -> bool
        {
            let current = current.map(|s| s.into_inner().get());
            if last.as_ref() == current { return false; }
            *last = current.cloned();
            current == Some(&state)
        }
    )
}

//-------------------------------------------------------------------------------------------------------------------
