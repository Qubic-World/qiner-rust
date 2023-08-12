use std::env;
use crate::env_names::ENV_VERSION;
use crate::types::{VERSION_SPLIT_CHAR, Version};

pub fn get_version() -> Version {
    let found_version = env::var(ENV_VERSION).unwrap();
    let split = found_version.split(VERSION_SPLIT_CHAR);

    let mut version: Version = Version::default();
    split.into_iter().enumerate().for_each(|(idx, item)| {
        version[idx] = item.trim().parse::<u8>().unwrap();
    });

    version
}

#[test]
fn test_get_version() {
    env::set_var(ENV_VERSION, "1. 141. 0");
    let version: Version = [1, 141, 0];

    assert_eq!(version, get_version());
}