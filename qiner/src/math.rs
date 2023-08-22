use std::mem::size_of;
use lib::types::{Nonce64, PublicKey64, State64, KECCAK_ROUND, STATE_SIZE_64};

pub fn random_64_by_ptr(
    public_key: &PublicKey64,
    nonce: &Nonce64,
    data_size: usize,
    data: *mut u64,
) {
    let mut state: State64 = State64::default();
    state[..public_key.len()].copy_from_slice(public_key);
    state[public_key.len()..public_key.len() + nonce.len()].copy_from_slice(nonce);

    let parts_mut = unsafe { std::slice::from_raw_parts_mut(data, data_size / size_of::<u64>()) };
    let chunks_mut = parts_mut.chunks_mut(STATE_SIZE_64);
    chunks_mut.for_each(|chunk|{
        keccak::p1600(&mut state, KECCAK_ROUND);
        chunk.clone_from_slice(&state[..chunk.len()]);
    });
}
