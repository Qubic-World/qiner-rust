use std::env;
use crate::env_names::VERSION;
use crate::types::Version;

pub fn get_version() -> Version {
	let found_version = env::var(VERSION).unwrap();
	let splited = found_version.split('.');

	let mut version: Version = Version::default();
	splited.into_iter().enumerate().for_each(|(idx, item)| {
		version[idx] = item.parse::<u8>().unwrap();
	});

	version
}