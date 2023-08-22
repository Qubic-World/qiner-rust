use std::arch::x86_64::_rdrand64_step;
use crate::math::random_64_by_ptr;
use lib::pointer::{as_const_ptr, as_mut_ptr};
use lib::solution_threshold::get_solution_threshold;
use lib::types::{MiningData, NeuronsInput, NeuronsOutput, Nonce64, PublicKey64, Seed, Seed64, SynapsesInput, SynapsesOutput, DATA_LENGTH, INFO_LENGTH, NUMBER_OF_INPUT_NEURONS, NUMBER_OF_OUTPUT_NEURONS, SynapsesLengths, MAX_INPUT_DURATION, NeuronItem, MAX_OUTPUT_DURATION, SYNAPSES_OUTPUT_LEN};
use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::ThreadId;

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
    pub lengths: SynapsesLengths,
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

}

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
        crate::math::random_64_by_ptr(&random_seed, &random_seed, size_of::<MiningData>(), as_mut_ptr(&mut mining_data));

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
        static SYNAPSES_SIZE: usize = size_of::<Synapses>();
        random_64_by_ptr(&self.computor_public_key, nonce, SYNAPSES_SIZE, synapses_ptr);

        for input_neuron_index in 0..NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
            for another_input_neuron_index in 0..DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
                let offset = (input_neuron_index * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH)) + another_input_neuron_index;
                neuron_data.synapses.input[offset] = ((neuron_data.synapses.input[offset]) as u8 % 3u8) as i8 - 1;
            }
        }

        for output_neuron_index in 0..NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
            for another_output_neuron_index in 0..INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
                let offset = (output_neuron_index * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH)) + another_output_neuron_index;
                neuron_data.synapses.output[offset] = ((neuron_data.synapses.output[offset]) as u8 % 3u8) as i8 - 1;
            }
        }

        for input_neuron_index in 0..NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
            let idx = input_neuron_index * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + (DATA_LENGTH + input_neuron_index);
            neuron_data.synapses.input[idx] = 0;
        }
        for output_neuron_index in 0..NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
            let idx = output_neuron_index * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) + (INFO_LENGTH + output_neuron_index);
            neuron_data.synapses.output[idx] = 0;
        }

        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                as_const_ptr(&self.mining_data),
                as_mut_ptr(&mut neuron_data.neurons.input),
                size_of::<MiningData>(),
            );

            std::ptr::write_bytes::<u8>(
                as_mut_ptr(&mut neuron_data.neurons.input[size_of::<MiningData>() / size_of::<NeuronItem>()]),
                0,
                size_of::<Neurons>() - size_of::<MiningData>(),
            );
        }

        let mut length_index = 0u32;
        for _tick in 0..MAX_INPUT_DURATION {
            let mut neuron_indices: [u16; NUMBER_OF_INPUT_NEURONS + INFO_LENGTH] = std::array::from_fn(|i| i as u16);
            let mut number_of_remaining_neurons = (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) as u16;

            while number_of_remaining_neurons > 0 {
                let neuron_index_index = neuron_data.synapses.lengths[length_index as usize] as u16 % number_of_remaining_neurons;
                let input_neuron_index = neuron_indices[neuron_index_index as usize];

                length_index += 1;
                number_of_remaining_neurons -= 1;

                neuron_indices[neuron_index_index as usize] = neuron_indices[number_of_remaining_neurons as usize];

                for another_input_neuron_index in 0..DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
                    let mut value: i32 = if neuron_data.neurons.input[another_input_neuron_index] >= 0 { 1 } else { -1 };

                    value *= neuron_data.synapses.input[(input_neuron_index as usize * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + another_input_neuron_index)] as i32;
                    neuron_data.neurons.input[DATA_LENGTH + input_neuron_index as usize] += value;
                }
            }
        }

        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                as_const_ptr(&neuron_data.neurons.input[DATA_LENGTH + NUMBER_OF_INPUT_NEURONS]),
                as_mut_ptr(&mut neuron_data.neurons.output),
                INFO_LENGTH * size_of::<NeuronItem>(),
            )
        }

        for _tick in 0..MAX_OUTPUT_DURATION {
            let mut neuron_indices: [u16; NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH] = std::array::from_fn(|i| i as u16);
            let mut number_of_remaining_neurons = (NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) as u16;

            while number_of_remaining_neurons > 0 {
                let neuron_index_index = neuron_data.synapses.lengths[length_index as usize] as u16 % number_of_remaining_neurons;
                let output_neuron_index = neuron_indices[neuron_index_index as usize];

                length_index += 1;
                number_of_remaining_neurons -= 1;

                neuron_indices[neuron_index_index as usize] = neuron_indices[number_of_remaining_neurons as usize];

                for another_output_neuron_index in 0..INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
                    let mut value: i32 = if neuron_data.neurons.output[another_output_neuron_index] >= 0 { 1 } else { -1 };

                    value *= neuron_data.synapses.output[(output_neuron_index as usize * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) + another_output_neuron_index)] as i32;
                    neuron_data.neurons.output[INFO_LENGTH + output_neuron_index as usize] += value;
                }
            }
        }

        let mut score = 0;
        for i in 0..DATA_LENGTH {
            if (self.mining_data[i] >= 0) == (neuron_data.neurons.output[INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + i] >= 0) {
                score += 1;
            }
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