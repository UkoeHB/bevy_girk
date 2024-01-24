//local shortcuts
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

/// Tracks ping requests in order to calculate latency and estimate the current game tick.
//todo: consider collecting stats/ping history
//todo: consider also tracking server clock time (it's possible this implementation can't accurately estimate the game tick)
#[derive(Default, Resource)]
pub struct PingTracker
{
    /// Configuration for the game's tick rate.
    tick_duration_ns: u64,

    /// The roundtrip time of the last recorded ping cycle, in nanoseconds.
    roundtrip_latency_ns: u64,
    /// The game tick where the last recorded ping cycle received a response.
    game_ticks_elapsed: Tick,
    /// The approximate time where the logic for the game tick of the last ping cycle executed.
    ///
    /// Calculated only using the local clock.
    game_tick_time_ns: u64,
}

impl PingTracker
{
    /// Makes a new ping tracker.
    pub fn new(ticks_per_sec: u32) -> PingTracker
    {
        if ticks_per_sec == 0 { panic!("PingTracker: ticks per second is zero!"); }
        PingTracker{
                tick_duration_ns     : Duration::as_nanos(&tps_to_duration(ticks_per_sec)) as u64,
                roundtrip_latency_ns : 0u64,
                game_ticks_elapsed   : Tick(0),
                game_tick_time_ns    : u64::MAX
            }
    }

    /// Adds a ping cycle to the tracker.
    pub fn add_ping_cycle(&mut self, game_ticks_elapsed: Tick, ping_timestamp_ns: u64, current_time_ns: u64)
    {
        self.roundtrip_latency_ns = current_time_ns.saturating_sub(ping_timestamp_ns);
        self.game_ticks_elapsed   = game_ticks_elapsed;
        self.game_tick_time_ns    = ping_timestamp_ns + (self.roundtrip_latency_ns / 2);
    }

    /// Gets the expected time to send a ping request and for it to be received.
    pub fn oneway_latency_ns(&self) -> u64
    {
        self.roundtrip_latency_ns / 2
    }

    /// Gets the expected time to send a ping request and then receive a response.
    pub fn roundtrip_latency_ns(&self) -> u64
    {
        self.roundtrip_latency_ns
    }

    /// Estimates the current game tick, with a fractional component for the currently-elapsing tick.
    pub fn estimate_game_tick(&self, current_time_ns: u64) -> (Tick, f32)
    {
        if self.tick_duration_ns == 0u64 { panic!("PingTracker: tick duration is zero!"); }

        let elapsed_time_ns        = current_time_ns.saturating_sub(self.game_tick_time_ns);
        let elapsed_ticks          = ((elapsed_time_ns / self.tick_duration_ns) as u32) + self.game_ticks_elapsed.0;
        let elapsing_tick_fraction = ((elapsed_time_ns % self.tick_duration_ns) as f64) / (self.tick_duration_ns as f64);

        return ( Tick(elapsed_ticks), elapsing_tick_fraction as f32 );
    }
}

//-------------------------------------------------------------------------------------------------------------------
