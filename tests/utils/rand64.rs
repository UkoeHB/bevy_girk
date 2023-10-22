//local shortcuts
use bevy_girk_utils::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn test_rand64_shared_context(r1: &mut Rand64, r2: &mut Rand64) -> bool
{
    if r1.next() != r2.next() { return false; }
    if r1.next() != r2.next() { return false; }
    r1.inject(100u128);
    r2.inject(100u128);
    if r1.next() != r2.next() { return false; }
    let _ = r1.next();
    if r1.next() == r2.next() { return false; }

    return true;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn rand64_basic()
{
    // 1. null rand
    let mut r1 = Rand64::new("", 0u128);
    let mut r2 = Rand64::new("", 0u128);
    assert!(test_rand64_shared_context(&mut r1, &mut r2));

    // 2. basic rand
    let mut r1 = Rand64::new("basic", 1u128);
    let mut r2 = Rand64::new("basic", 1u128);
    assert!(test_rand64_shared_context(&mut r1, &mut r2));

    // 3. random seed
    #[cfg(not(target_arch = "wasm32"))]
    {
        let seed = gen_rand64_seed();
        let mut r1 = Rand64::new("random", seed);
        let mut r2 = Rand64::new("random", seed);
        assert!(test_rand64_shared_context(&mut r1, &mut r2));
    }
}

//-------------------------------------------------------------------------------------------------------------------
