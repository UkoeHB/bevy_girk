//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A game tick.
///
/// Represents a span of time where events occurred and logic was executed.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Default, Debug, Serialize, Deserialize, Deref, DerefMut)]
pub struct Tick(pub u32);

//-------------------------------------------------------------------------------------------------------------------
