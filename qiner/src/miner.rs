use crate::math::random_64_by_ptr;
use lib::pointer::{as_const_ptr, as_mut_ptr};
use lib::solution_threshold::get_solution_threshold;
use lib::types::{
    MiningData, NeuronsInput,
    NeuronsOutput, Nonce64, PublicKey64, Seed, Seed64, SynapsesInput, SynapsesOutput,
    DATA_LENGTH, INFO_LENGTH, MAX_INPUT_DURATION, MAX_OUTPUT_DURATION,
    NUMBER_OF_INPUT_NEURONS, NUMBER_OF_OUTPUT_NEURONS,
};
use std::arch::x86_64::_rdrand64_step;
use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::ThreadId;

type Counters = [u32; 2];

#[derive(Debug, Clone)]
pub struct NeuronContainer {
    neuron_data: HashMap<ThreadId, NeuronData>,
}

impl NeuronContainer {
    pub fn get_mut_data(&mut self, thread_id: &ThreadId) -> &mut NeuronData {
        if !self.neuron_data.contains_key(thread_id) {
            self.neuron_data
                .insert(thread_id.clone(), NeuronData::default());
        }

        self.neuron_data.get_mut(thread_id).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Neurons {
    input: NeuronsInput,
    output: NeuronsOutput,
}

impl Default for Neurons {
    fn default() -> Self {
        unsafe { zeroed::<Neurons>() }
    }
}

#[derive(Debug, Clone)]
pub struct Synapses {
    pub input: SynapsesInput,
    pub output: SynapsesOutput,
}

impl Default for Synapses {
    fn default() -> Self {
        unsafe { zeroed::<Synapses>() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NeuronData {
    neurons: Neurons,
    synapses: Synapses,
    /*	neuron_links: NeuronLinks64,
        neuron_values: NeuronValues,
    */
}
/*
impl Default for NeuronData {
    fn default() -> Self {
        NeuronData {
            neuron_links: [0; NUMBER_OF_NEURONS_64 * 2],
            neuron_values: [NeuronValue::MAX; NUMBER_OF_NEURONS],
        }
    }
}

*/
#[derive(Debug, Clone)]
pub struct Miner {
    solution_threshold: usize,
    num_tasks: usize,

    mining_data: MiningData,
    computor_public_key: PublicKey64,

    score_counter: Arc<AtomicUsize>,
    iter_counter: Arc<AtomicUsize>,

    pub found_nonce: Arc<std::sync::Mutex<Vec<Nonce64>>>,
}

impl Miner {
    pub fn new(computor_public_key: PublicKey64, num_threads: usize) -> Self {
        // Random Seed
        let random_seed = Miner::get_random_seed_64();

        // Zeroed mining data
        let mut mining_data: MiningData;
        unsafe {
            mining_data = zeroed::<MiningData>();
        }

        // Generate Mining data
        crate::math::random_64(&random_seed, &random_seed, &mut mining_data);

        Miner {
            solution_threshold: get_solution_threshold(),
            num_tasks: num_threads,
            mining_data,
            computor_public_key,
            score_counter: Arc::new(AtomicUsize::new(0)),
            iter_counter: Arc::new(AtomicUsize::new(0)),
            found_nonce: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_score(&self) -> usize {
        self.score_counter.load(Ordering::SeqCst)
    }

    pub fn get_iter_counter(&self) -> usize {
        self.iter_counter.load(Ordering::SeqCst)
    }

    fn get_random_seed_64() -> Seed64 {
        let seed = lib::random_seed::get_random_seed();

        let seed_64: Seed64;
        unsafe {
            seed_64 = std::mem::transmute::<Seed, Seed64>(seed);
        }

        seed_64
    }

    pub fn find_solution(&self, nonce: &mut Nonce64, neuron_data: &mut NeuronData) -> bool {
        // Fill nonce with random values
        nonce.iter_mut().for_each(|item| {
            unsafe { _rdrand64_step(item) };
        });

        // Fill synapses with random values
        let synapses_ptr: *mut u64 = as_mut_ptr(&mut neuron_data.synapses);
        let data_size = size_of::<Synapses>() / size_of::<u64>();
        random_64_by_ptr(&self.computor_public_key, nonce, data_size, synapses_ptr);

        // Synapses input
        for input_neuron_index in 0..NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
            let offset = input_neuron_index * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH)
                + (DATA_LENGTH + input_neuron_index);
            neuron_data.synapses.input[offset >> 5] &= !(3u64 << (offset & 31));
        }

        // Synapses output
        for output_neuron_index in 0..NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
            let offset = output_neuron_index
                * (DATA_LENGTH + NUMBER_OF_OUTPUT_NEURONS + INFO_LENGTH)
                + (INFO_LENGTH + output_neuron_index);
            neuron_data.synapses.output[offset >> 5] &= !(3u64 << (offset & 31));
        }

        unsafe {
            std::ptr::copy::<u8>(
                as_const_ptr(&self.mining_data),
                as_mut_ptr(&mut neuron_data.neurons.input),
                size_of::<MiningData>(),
            );
        }

        let mut offset = 0usize;
        let mut counters: Counters = Default::default();

        for _tick in 0..MAX_INPUT_DURATION {
            offset = 0usize;
            for input_neuron_index in 0..NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
                counters = Default::default();
                for another_input_neuron_index in
                0..DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH
                {
                    // let offset = input_neuron_index * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + another_input_neuron_index;
                    let synapse = neuron_data.synapses.input[offset >> 5] >> ((offset & 31) << 1);
                    offset += 1;
                    if synapse & 1 != 0 {
                        // 
                        let idx = (((neuron_data.neurons.input[another_input_neuron_index >> 6] >> (another_input_neuron_index & 63)) ^ (synapse >> 1)) & 1) as usize;
                        counters[idx] += 1;
                    }
                }
                {
                    let idx = DATA_LENGTH / 64 + (input_neuron_index >> 6);
                    let value = 1u64 << (input_neuron_index & 63);

                    if counters[1] > counters[0] {
                        neuron_data.neurons.input[idx] &= !value;
                    } else {
                        neuron_data.neurons.input[idx] |= value;
                    }
                }
            }
        }

        {
            let src_ptr = as_const_ptr(
                &neuron_data.neurons.input[(DATA_LENGTH + NUMBER_OF_INPUT_NEURONS) / 64],
            );
            let dst_ptr = as_mut_ptr(&mut neuron_data.neurons.output);
            unsafe {
                std::ptr::copy::<u8>(src_ptr, dst_ptr, INFO_LENGTH / 8);
            }
        }

        for _tick in 0..MAX_OUTPUT_DURATION {
            offset = 0usize;
            for output_neuron_index in 0..NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
                counters = Default::default();
                for another_output_neuron_index in
                0..INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH
                {
                    let synapse = neuron_data.synapses.output[offset >> 5] >> ((offset & 31) << 1);
                    offset += 1;
                    if synapse & 1 != 0 {
                        let idx = (((neuron_data.neurons.output[another_output_neuron_index >> 6] >> (another_output_neuron_index & 63)) ^ (synapse >> 1)) & 1) as usize;
                        counters[idx] += 1;
                    }
                }

                {
                    let idx = INFO_LENGTH / 64 + (output_neuron_index >> 6);
                    let value = 1u64 << (output_neuron_index & 63);
                    if counters[1] > counters[0] {
                        neuron_data.neurons.output[idx] &= !value;
                    } else {
                        neuron_data.neurons.output[idx] |= value;
                    }
                }
            }
        }

        let mut score: usize = 0;
        for idx in 0..DATA_LENGTH / 64 {
            let value = self.mining_data[idx]
                ^ neuron_data.neurons.output[(INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS) / 64 + idx];
            let one_bit_counter = value.count_ones();
            score += (64 - one_bit_counter) as usize;
        }

        return score >= self.solution_threshold;
    }

    pub fn run_in_thread(miner: Arc<Miner>) {
        let mut nonce: Nonce64 = Nonce64::default();
        let mut neuron_data = NeuronData::default();
        let mut nonce_for_send: Vec<Nonce64> = Vec::new();

        loop {
            if miner.find_solution(&mut nonce, &mut neuron_data) {
                miner.score_counter.fetch_add(1, Ordering::Relaxed);
                nonce_for_send.push(nonce);
            }

            if !nonce_for_send.is_empty() {
                if let Ok(lock) = miner.found_nonce.try_lock().as_mut() {
                    lock.append(&mut nonce_for_send);
                }
            }

            miner.iter_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn run(miner: &Arc<Miner>) {
        for idx in 0..miner.num_tasks {
            let arc_miner_clone = miner.clone();

            let idx_clone = idx.clone();

            tokio::spawn(async move {
                let mut nonce: Nonce64 = Nonce64::default();
                let mut neuron_data = NeuronData::default();
                let mut nonce_for_send: Vec<Nonce64> = Vec::new();

                loop {
                    log::debug!(
                        "[{idx_clone}]Find solution in Thread Id ({:?})",
                        thread::current().id()
                    );

                    if arc_miner_clone.find_solution(&mut nonce, &mut neuron_data) {
                        arc_miner_clone
                            .score_counter
                            .fetch_add(1, Ordering::Relaxed);
                        nonce_for_send.push(nonce);
                    }

                    if !nonce_for_send.is_empty() {
                        if let Ok(lock) = arc_miner_clone.found_nonce.try_lock().as_mut() {
                            lock.append(&mut nonce_for_send);
                        }
                    }

                    arc_miner_clone.iter_counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    }
}
