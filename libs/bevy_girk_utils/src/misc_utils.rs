//local shortcuts

//third-party shortcuts

//standard shortcuts
use std::path::PathBuf;
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------

/// Try to set the asset directory location based on its location relative to the package's cargo manifest.
pub fn try_set_bevy_asset_root(parent_dir_level: u8) -> Result<(), std::env::VarError>
{
    // 1. try to get the directory of the cargo manifest of the crate being built
    let mut manifest_directory = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

    // 2. back out of cargo manifest directory the specified number of levels
    for _ in 0..parent_dir_level
    { manifest_directory = manifest_directory.join(".."); }

    // 3. check if we found the assets directory
    if !manifest_directory.join("assets").is_dir()
    { return Err(std::env::VarError::NotPresent); }

    // 4. set the bevy asset root
    std::env::set_var("BEVY_ASSET_ROOT", manifest_directory.into_os_string());
    return Ok(())
}

//-------------------------------------------------------------------------------------------------------------------

/// Convert ticks/sec to tick duration.
///
/// Minimum = 1 tick per second.
pub fn tps_to_duration(ticks_per_sec: u32) -> Duration
{
    Duration::from_nanos(1_000_000_000u64.checked_div(ticks_per_sec as u64).unwrap_or(1_000_000_000u64))
}

//-------------------------------------------------------------------------------------------------------------------
