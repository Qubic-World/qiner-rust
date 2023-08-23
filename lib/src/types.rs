use std::mem::size_of;

// Constants

pub const STATE_SIZE: usize = 200;
pub const STATE_SIZE_64: usize = 200 / size_of::<u64>();
pub const DATA_LENGTH: usize = 2000;
pub const INFO_LENGTH: usize = 1000;
pub const NUMBER_OF_INPUT_NEURONS: usize = 1000;
pub const NUMBER_OF_OUTPUT_NEURONS: usize = 1000;
pub const MAX_INPUT_DURATION: usize = 20;
pub const MAX_OUTPUT_DURATION: usize = 20;
pub const KECCAK_ROUND: usize = 12;
pub const SEED_ITEM_NUM: usize = 32;

pub(crate) const RANDOM_SEED_SPLIT_CHAR: char = ',';

pub const PORT: u16 = 21841u16;
#[cfg(debug_assertions)]
pub const STACK_SIZE: usize = 25165824 + 128 * 1024 * 1024;
#[cfg(not(debug_assertions))]
pub const STACK_SIZE: usize = 25165824;

pub const NUMBER_OF_NONCE: usize = 32;
pub const NUMBER_OF_NONCE_64: usize = NUMBER_OF_NONCE / size_of::<u64>();

pub const SYNAPSES_INPUT_LEN: usize = (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH);
pub const SYNAPSES_OUTPUT_LEN: usize = (NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH);

// Types
pub type SeedItem = u8;
pub type Seed = [SeedItem; SEED_ITEM_NUM];
pub type PublicKey = [u8; 32];
pub type Nonce = [u8; NUMBER_OF_NONCE];
pub type State = [u8; STATE_SIZE];
pub type MiningItemData = i32;
pub type MiningData = [MiningItemData; DATA_LENGTH];
pub type Id = [u8; 60];
pub type Signature = [u64; 8];
pub type Gamma = [u8; 32];

pub type SynapseItem = i8;
pub type NeuronItem = i32;

// 64
pub type Seed64 = [u64; SEED_ITEM_NUM / 8];
pub type PublicKey64 = [u64; 4];
pub type State64 = [u64; STATE_SIZE_64];
pub type Nonce64 = [u64; NUMBER_OF_NONCE_64];
pub type NeuronsInput = [NeuronItem; DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH];
pub type NeuronsOutput = [NeuronItem; INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH];
pub type SynapsesInput = [SynapseItem; SYNAPSES_INPUT_LEN];
pub type SynapsesOutput = [SynapseItem; SYNAPSES_OUTPUT_LEN];
pub type SynapsesLengths = [u16; MAX_INPUT_DURATION * (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + MAX_OUTPUT_DURATION * (NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH)];

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
    // assert_eq!(size_of::<NeuronValues>(), size_of::<NeuronValues64>());
    assert_eq!(size_of::<Nonce>(), size_of::<Nonce64>());
    assert_eq!(size_of::<State>(), size_of::<State64>());
    assert_eq!(size_of::<Seed>(), size_of::<Seed64>());
    assert_eq!(size_of::<PublicKey>(), size_of::<PublicKey64>());
    assert_eq!(size_of::<network::Key>(), size_of::<network::Key64>());
}
