use std::mem::size_of;
use crate::types::network::{Key, Key64};

// Constants

pub const STATE_SIZE: usize = 200;
pub const STATE_SIZE_64: usize = 200 / size_of::<u64>();
pub const SOLUTION_THRESHOLD: usize = 21;
pub const NUMBER_OF_NEURONS: usize = 4194304;
pub const NUMBER_OF_NEURONS_64: usize = NUMBER_OF_NEURONS * size_of::<NeuronLink>() / size_of::<u64>();
pub const NEURON_MOD_BITS: u64 = (((NUMBER_OF_NEURONS - 1) << size_of::<NeuronLink>() * 8) | (NUMBER_OF_NEURONS - 1)) as u64;
pub const MINING_DATA_LENGTH: usize = 1024;
pub const KECCAK_ROUND: usize = 12;

pub const PORT: u16 = 21841u16;
pub const VERSION_A: u8 = 1u8;
pub const VERSION_B: u8 = 140u8;
pub const VERSION_C: u8 = 0u8;

pub const STACK_SIZE: usize = 40 * 1024 * 1024;

#[deprecated]
pub const NUMBER_OF_NEURON_VALUES_64: usize = size_of::<NeuronValues>() / size_of::<u64>();
pub const NUMBER_OF_NONCE: usize = 32;
pub const NUMBER_OF_NONCE_64: usize = NUMBER_OF_NONCE / size_of::<u64>();

// Types

pub type Seed = [u8; 32];
pub type PublicKey = [u8; 32];
pub type Nonce = [u8; NUMBER_OF_NONCE];
pub type State = [u8; STATE_SIZE];
pub type MiningItemData = u64;
pub type MiningData = [MiningItemData; MINING_DATA_LENGTH];
pub type NeuronLink = u32;
pub type NeuronLinks = [NeuronLink; NUMBER_OF_NEURONS * 2];
pub type NeuronValue = u8;
pub type NeuronValues = [NeuronValue; NUMBER_OF_NEURONS];
pub type Id = [u8; 60];
pub type Signature = [u64; 8];
pub type Gamma = [u8; 32];
pub type Version = [u8; 3];

// 64
pub type Seed64 = [u64; 4];
pub type PublicKey64 = [u64; 4];
pub type State64 = [u64; STATE_SIZE_64];
pub type Nonce64 = [u64; NUMBER_OF_NONCE_64];
pub type NeuronLink64 = u64;
pub type NeuronLinks64 = [NeuronLink64; NUMBER_OF_NEURONS_64 * 2];
pub type NeuronValue64 = u16;
pub type NeuronValues64 = [NeuronValue64; NUMBER_OF_NEURONS_64];

pub mod network {
	use std::mem::size_of;
	use crate::types::NUMBER_OF_NONCE;

	pub type Size = [u8; 3];
	pub type Protocol = u8;
	pub type Dejavu = [u8; DEJAVU_ITEM_NUM];
	pub type Type = u8;
	pub type Key = [u8; KEY_ITEM_NUM];
	pub type Key64 = [u64; KEY_ITEM_NUM_64];
	pub type KeyAndNonce = [u8; KEY_ITEM_NUM + NUMBER_OF_NONCE];

	// Constants
	pub const DEJAVU_ITEM_NUM: usize = 3;
	pub const KEY_ITEM_NUM: usize = 32;
	pub const KEY_ITEM_NUM_64: usize = KEY_ITEM_NUM / size_of::<u64>();

	pub mod protocols {
		use crate::types::network::Type;

		pub const BROADCAST_MESSAGE: Type = 1;
	}
}

#[test]
fn test_types() {
	assert_eq!(size_of::<NeuronLinks>(), size_of::<NeuronLinks64>());
	// assert_eq!(size_of::<NeuronValues>(), size_of::<NeuronValues64>());
	assert_eq!(size_of::<Nonce>(), size_of::<Nonce64>());
	assert_eq!(size_of::<State>(), size_of::<State64>());
	assert_eq!(size_of::<Seed>(), size_of::<Seed64>());
	assert_eq!(size_of::<PublicKey>(), size_of::<PublicKey64>());
	assert_eq!(size_of::<Key>(), size_of::<Key64>());
}
