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

/// Readiness of a client (on a scale of [0.0, 1.0])
#[derive(Copy, Clone, Default)]
pub struct Readiness
{
    readiness: f32
}

impl Readiness
{
    pub fn new(readiness_raw: f32) -> Readiness
    {
        match readiness_raw.is_nan()
        {
            true  => Readiness{ readiness: 1.0 },  //assume NaN implies 0.0 / 0.0, which is a readiness of 100%
            false => Readiness{ readiness: readiness_raw.clamp(0.0, 1.0) }
        }
    }

    pub fn value(&self) -> f32 { self.readiness }
    pub fn is_ready(&self) -> bool { self.readiness >= 1.0 }
}

//-------------------------------------------------------------------------------------------------------------------
