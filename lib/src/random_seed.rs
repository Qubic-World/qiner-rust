use std::env;
use crate::env_names::ENV_RANDOM_SEED;
use crate::types::{RANDOM_SEED_SPLIT_CHAR, Seed, SeedItem};

pub fn get_random_seed() -> Seed {
    let mut random_seed = Seed::default();
    let random_seed_string = env::var(ENV_RANDOM_SEED).unwrap();
    let split = random_seed_string.split(RANDOM_SEED_SPLIT_CHAR);
    for (split_item, seed_item) in split.zip(random_seed.as_mut()) {
        *seed_item = split_item.trim().parse::<SeedItem>().unwrap();
    }

    random_seed
}

#[test]
fn test_randomseed() {
    env::set_var(ENV_RANDOM_SEED, "  126, 27, 26, 27,    26, 27, 26, 27  ");
    let mut random_seed: Seed = Seed::default();
    random_seed[0] = 126;
    random_seed[1] = 27;
    random_seed[2] = 26;
    random_seed[3] = 27;
    random_seed[4] = 26;
    random_seed[5] = 27;
    random_seed[6] = 26;
    random_seed[7] = 27;

    assert_eq!(random_seed, get_random_seed());
}
