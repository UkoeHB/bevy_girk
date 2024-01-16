//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Tick counter utility for tick-based run conditions.
///
/// The `TickCounter` resource and `increment_tick_counter` system must be manually added to your bevy app.
///
/// ### Example
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_girk_utils::*;
/// # fn my_system() {}
/// # fn t()
/// # {
/// let mut app = App::default();
/// app.init_resource::<TickCounter>()
///     .add_systems(Update,
///         (
///             increment_tick_counter,
///             my_system.run_if(on_tick_counter(10u64)),
///         ).chain()
///     );
/// # }
///```
#[derive(Resource, Default, Debug)]
pub struct TickCounter
{
    ticks: u64,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn increment_tick_counter(mut tick_counter: ResMut<TickCounter>)
{
    (*tick_counter).ticks += 1;
}

//-------------------------------------------------------------------------------------------------------------------

pub fn on_tick_counter(tick_period: u64) -> impl FnMut(Res<TickCounter>) -> bool + Clone
{
    let mut last_tick: u64 = 0;
    move | tick_counter: Res<TickCounter> |
    {
        if (*tick_counter).ticks.saturating_sub(last_tick) < tick_period { return false; }
        last_tick = (*tick_counter).ticks;
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------
