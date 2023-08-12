use lib::types::{Nonce64, PublicKey64, State64, KECCAK_ROUND, STATE_SIZE_64};

pub(crate) fn random_64<const S: usize>(
    public_key: &PublicKey64,
    nonce: &Nonce64,
    output: &mut [u64; S],
) {
    let mut state: State64 = State64::default();
    state[..public_key.len()].copy_from_slice(public_key);
    state[public_key.len()..public_key.len() + nonce.len()].copy_from_slice(nonce);

    let mut chunks_mut = output.chunks_mut(STATE_SIZE_64);

    while let Some(chunk) = chunks_mut.next() {
        keccak::p1600(&mut state, KECCAK_ROUND);
        chunk.clone_from_slice(&state[..chunk.len()]);
    }
}

pub fn random_64_by_ptr(
    public_key: &PublicKey64,
    nonce: &Nonce64,
    data_size: usize,
    data: *mut u64,
) {
    let mut state: State64 = State64::default();
    state[..public_key.len()].copy_from_slice(public_key);
    state[public_key.len()..public_key.len() + nonce.len()].copy_from_slice(nonce);

    let parts_mut = unsafe { std::slice::from_raw_parts_mut(data, data_size) };
    let mut chunks_mut = parts_mut.chunks_mut(STATE_SIZE_64);
    while let Some(chunk) = chunks_mut.next() {
        keccak::p1600(&mut state, KECCAK_ROUND);
        chunk.clone_from_slice(&state[..chunk.len()]);
    }
}
