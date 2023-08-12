use std::mem::size_of;

// Constants

pub const STATE_SIZE: usize = 200;
pub const STATE_SIZE_64: usize = 200 / size_of::<u64>();
pub const NUMBER_OF_NEURONS: usize = 4194304;
pub const NUMBER_OF_NEURONS_64: usize =
    NUMBER_OF_NEURONS * size_of::<NeuronLink>() / size_of::<u64>();
pub const NEURON_MOD_BITS: u64 =
    (((NUMBER_OF_NEURONS - 1) << size_of::<NeuronLink>() * 8) | (NUMBER_OF_NEURONS - 1)) as u64;
pub const DATA_LENGTH: usize = 1024;
pub const INFO_LENGTH: usize = 512;
pub const NUMBER_OF_INPUT_NEURONS: usize = 640;
pub const NUMBER_OF_OUTPUT_NEURONS: usize = 640;
pub const MAX_INPUT_DURATION: usize = 10;
pub const MAX_OUTPUT_DURATION: usize = 10;
pub const KECCAK_ROUND: usize = 12;
pub const SEED_ITEM_NUM: usize = 32;

pub(crate) const RANDOM_SEED_SPLIT_CHAR: char = ',';

pub const PORT: u16 = 21841u16;
#[cfg(debug_assertions)]
pub const STACK_SIZE: usize = 2097152 + 5 * 1024 * 1024;
#[cfg(not(debug_assertions))]
pub const STACK_SIZE: usize = 2097152;

#[deprecated]
pub const NUMBER_OF_NEURON_VALUES_64: usize = size_of::<NeuronValues>() / size_of::<u64>();
pub const NUMBER_OF_NONCE: usize = 32;
pub const NUMBER_OF_NONCE_64: usize = NUMBER_OF_NONCE / size_of::<u64>();

// Types
pub type SeedItem = u8;
pub type Seed = [SeedItem; SEED_ITEM_NUM];
pub type PublicKey = [u8; 32];
pub type Nonce = [u8; NUMBER_OF_NONCE];
pub type State = [u8; STATE_SIZE];
pub type MiningItemData = u64;
pub type MiningData = [MiningItemData; DATA_LENGTH / 64];
pub type NeuronLink = u32;
pub type NeuronLinks = [NeuronLink; NUMBER_OF_NEURONS * 2];
pub type NeuronValue = u8;
pub type NeuronValues = [NeuronValue; NUMBER_OF_NEURONS];
pub type Id = [u8; 60];
pub type Signature = [u64; 8];
pub type Gamma = [u8; 32];

pub type SynapseItem = u64;

// 64
pub type Seed64 = [u64; 4];
pub type PublicKey64 = [u64; 4];
pub type State64 = [u64; STATE_SIZE_64];
pub type Nonce64 = [u64; NUMBER_OF_NONCE_64];
pub type NeuronLink64 = u64;
pub type NeuronLinks64 = [NeuronLink64; NUMBER_OF_NEURONS_64 * 2];
pub type NeuronsInput = [u64; (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) / 64];
pub type NeuronsOutput = [u64; (DATA_LENGTH + NUMBER_OF_OUTPUT_NEURONS + INFO_LENGTH) / 64];
pub type SynapsesInput = [SynapseItem;
    (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH)
        / (64 / 2)];
pub type SynapsesOutput = [SynapseItem;
    (NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH)
        * (DATA_LENGTH + NUMBER_OF_OUTPUT_NEURONS + INFO_LENGTH)
        / (64 / 2)];
pub type NeuronValue64 = u16;
pub type NeuronValues64 = [NeuronValue64; NUMBER_OF_NEURONS_64];

pub mod network {
    use crate::types::NUMBER_OF_NONCE;
    use std::mem::size_of;

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
    assert_eq!(size_of::<network::Key>(), size_of::<network::Key64>());
}
