//local shortcuts
use bevy_girk_utils::*;
use crate::click_game_integration::click_game::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Produce a new PRNG for a specific player.
pub fn make_player_rand(domain_sep: &str, seed: u64, player_id: PlayerId) -> Rand64
{
    let shifted_seed = (seed as u128).checked_shl(64).unwrap();
    let player_seed  = shifted_seed + (player_id.id.raw() as u128);

    Rand64::new(domain_sep, player_seed)
}

//-------------------------------------------------------------------------------------------------------------------
