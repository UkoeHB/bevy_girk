//local shortcuts

//third-party shortcuts
use siphasher::sip128::{Hash128, Hasher128, SipHasher24};

//standard shortcuts
use std::hash::Hasher;

//-------------------------------------------------------------------------------------------------------------------

/// Random 64-bit number generator (cryptographically secure, deterministic from seed).
pub struct Rand64
{
    cached_hasher : SipHasher24,
    state         : Hash128  //[prefix || value]
}

impl Rand64
{
    /// Makes a new PRNG.
    pub fn new(domain_sep: &str, seed: u128) -> Rand64
    {
        let mut hasher = SipHasher24::new_with_key(&seed.to_le_bytes());
        hasher.write(domain_sep.as_bytes());

        Rand64{
                cached_hasher : hasher,
                state         : hasher.finish128()
            }
    }

    /// Gets the next random number.
    pub fn next(&mut self) -> u64
    {
        self.advance_state();
        self.state.h2
    }

    /// Gets the next random number normalized to `[0.0, 1.0]`.
    pub fn next_normalized(&mut self) -> f64
    {
        self.next() as f64 / u64::MAX as f64
    }

    /// Injects additional entropy.
    pub fn inject(&mut self, extra_entropy: u128)
    {
        let mut hasher = self.cached_hasher.clone();
        hasher.write(&self.state.as_bytes());
        hasher.write(&extra_entropy.to_le_bytes());
        self.state = hasher.finish128();
    }

    /// Hashes the state to advance it.
    fn advance_state(&mut self)
    {
        // [new prefix || new value] = H([old prefix || old value])
        let mut hasher = self.cached_hasher.clone();
        hasher.write(&self.state.as_bytes());
        self.state = hasher.finish128();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Generates a random seed for [`Rand64`].
#[cfg(not(target_arch = "wasm32"))]
pub fn gen_rand64_seed() -> u128
{
    use rand::RngCore;
    let mut seed = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    return u128::from_le_bytes(seed);
}

//-------------------------------------------------------------------------------------------------------------------

/// Generates a random 64-bit integer.
#[cfg(not(target_arch = "wasm32"))]
pub fn gen_rand64() -> u64
{
    use rand::RngCore;
    let mut seed = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    return u64::from_le_bytes(seed);
}

//-------------------------------------------------------------------------------------------------------------------

/// Generates a random 128-bit integer.
#[cfg(not(target_arch = "wasm32"))]
pub fn gen_rand128() -> u128
{
    use rand::RngCore;
    let mut seed = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    return u128::from_le_bytes(seed);
}

//-------------------------------------------------------------------------------------------------------------------
