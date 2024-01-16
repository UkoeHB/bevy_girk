//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A number of game ticks.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Ticks(pub u32);

//-------------------------------------------------------------------------------------------------------------------

/// A ticks counter.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct TicksElapsed
{
    ticks: Ticks
}

impl TicksElapsed
{
    /// Get the current number of elapsed ticks.
    pub fn ticks(&self) -> Ticks
    {
        self.ticks
    }
    /// Advance the elapsed ticks.
    pub fn advance(&mut self)
    {
        self.ticks.0 += 1;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Readiness of an entity (on a scale of [0.0, 1.0])
#[derive(Copy, Clone, Component, Default)]
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
