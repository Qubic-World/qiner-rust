use std::arch::x86_64::{__m512i, _mm512_add_epi32, _mm512_load_epi32, _mm512_loadu_epi32, _mm512_mul_epi32, _mm512_or_epi32, _mm512_reduce_add_epi32, _mm512_set1_epi32, _mm512_srli_epi32, _rdrand64_step};
use crate::math::random_64_by_ref;
use lib::pointer::{as_const_ptr, as_mut_ptr, as_mut_slice};
use lib::solution_threshold::get_solution_threshold;
use lib::types::{MiningData, NeuronsInput, NeuronsOutput, Nonce64, PublicKey64, Seed64, SynapsesInput, SynapsesOutput, DATA_LENGTH, INFO_LENGTH, NUMBER_OF_INPUT_NEURONS, NUMBER_OF_OUTPUT_NEURONS, SynapsesLengths, MAX_INPUT_DURATION, NeuronItem, MAX_OUTPUT_DURATION};
use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::simd::{SimdInt};
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

#[derive(Debug, Clone, PartialEq)]
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

type Indexes = [u16; (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) * MAX_INPUT_DURATION];

#[derive(Debug, Clone)]
pub struct Miner {
    solution_threshold: usize,
    num_tasks: usize,

    mining_data: MiningData,
    computor_public_key: PublicKey64,

    score_counter: Arc<AtomicUsize>,
    iter_counter: Arc<AtomicUsize>,

    number_of_remaining_neurons_array: Indexes,
    neuron_indices_array: Indexes,

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

        let mut number_of_remaining_neurons_array: Indexes = unsafe { zeroed::<Indexes>() };
        number_of_remaining_neurons_array.iter_mut().enumerate().for_each(|(idx, item)| {
            *item = ((NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) - idx % (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH)) as u16;
        });

        let neuron_indices_array: Indexes = std::array::from_fn(|i| i as u16);

        Miner {
            solution_threshold: get_solution_threshold(),
            num_tasks: num_threads,
            mining_data,
            computor_public_key,
            score_counter: Arc::new(AtomicUsize::new(0)),
            iter_counter: Arc::new(AtomicUsize::new(0)),
            number_of_remaining_neurons_array,
            neuron_indices_array,
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

        let mut seed_64: Seed64 = Seed64::default();
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(as_const_ptr(&seed), as_mut_ptr(&mut seed_64), size_of::<Seed64>());
        }

        seed_64
    }

    pub fn find_solution(&self, nonce: &mut Nonce64, neuron_data: &mut NeuronData) -> bool {
        // Fill nonce with random values
        nonce.iter_mut().for_each(|item| {
            unsafe { _rdrand64_step(item) };
        });

        // Fill synapses with random values
        random_64_by_ref(&self.computor_public_key, nonce, &mut neuron_data.synapses);

        neuron_data.synapses.input.iter_mut().for_each(|item_input| {
            let item_value = ((*item_input as u8) % 3 - 1) as i8;
            *item_input = item_value;
        });

        neuron_data.synapses.output.iter_mut().for_each(|item_output| {
            let item_value = ((*item_output as u8) % 3 - 1) as i8;
            *item_output = item_value;
        });

        for input_neuron_index in 0..NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
            let idx = input_neuron_index * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + (DATA_LENGTH + input_neuron_index);
            unsafe { *neuron_data.synapses.input.get_unchecked_mut(idx) = 0 };
        }
        for output_neuron_index in 0..NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH {
            let idx = output_neuron_index * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) + (INFO_LENGTH + output_neuron_index);
            unsafe { *neuron_data.synapses.output.get_unchecked_mut(idx) = 0 };
        }

        // Copy with func
        neuron_data.neurons.input.iter_mut().zip(self.mining_data.iter()).for_each(|(neuron, data)| {
            *neuron = *data;
        });

        neuron_data.neurons.input.iter_mut().skip(size_of::<MiningData>() / size_of::<NeuronItem>()).for_each(|neuron| {
            *neuron = 0;
        });

        type Indexes = [u16; (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) * MAX_INPUT_DURATION];
        let mut neuron_indices_array: Indexes = self.neuron_indices_array;

        self.number_of_remaining_neurons_array.iter()
            .zip(neuron_data.synapses.lengths.iter())
            .for_each(|(number_of_remaining_neurons, length)| {
                unsafe {
                    let neuron_index_index = *length % *number_of_remaining_neurons;
                    let input_neuron_index = *neuron_indices_array.get_unchecked(neuron_index_index as usize);
                    *neuron_indices_array.get_unchecked_mut(neuron_index_index as usize) = *neuron_indices_array.get_unchecked((*number_of_remaining_neurons - 1) as usize);

                    let mut result_512_16 = _mm512_set1_epi32(neuron_data.neurons.input[DATA_LENGTH + input_neuron_index as usize]);
                    for another_input_neuron_index in (0..DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH).step_by(16) {
                        let neuron_inputs_16_512: __m512i = __m512i::from(std::simd::i32x16::from_slice(as_mut_slice(&mut neuron_data.neurons.input[another_input_neuron_index], 32)));

                        let mut value_16_512 = _mm512_srli_epi32::<31>(neuron_inputs_16_512);
                        value_16_512 = _mm512_or_epi32(value_16_512, _mm512_set1_epi32(1));
                        let neuron_inputs_32 = __m512i::from(std::simd::i32x16::from_slice(as_mut_slice(&mut neuron_data.synapses.input[(input_neuron_index as usize * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + another_input_neuron_index)], 32)));

                        value_16_512 = _mm512_mul_epi32(value_16_512, neuron_inputs_32);

                        result_512_16 = _mm512_add_epi32(result_512_16, value_16_512);

                        *neuron_data.neurons.input.get_unchecked_mut(DATA_LENGTH + input_neuron_index as usize) = _mm512_reduce_add_epi32(result_512_16);
                        /*                      let mut value: i32 = (*neuron_data.neurons.input.get_unchecked(another_input_neuron_index) >> 31) | 1;
                          
                                              value *= *neuron_data.synapses.input.get_unchecked((*input_neuron_index as usize * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + another_input_neuron_index)) as i32;
                                              neuron_input += value;
                                              *neuron_data.neurons.input.get_unchecked_mut(DATA_LENGTH + *input_neuron_index as usize) += value;
                        */
                    }

                    /*                    for another_input_neuron_index in 0..DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
                                            let mut value: i32 = (neuron_data.neurons.input.get_unchecked(another_input_neuron_index) >> 31) | 1;
                    
                                            value *= *neuron_data.synapses.input.get_unchecked((input_neuron_index as usize * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH)) + another_input_neuron_index) as i32;
                                            *neuron_data.neurons.input.get_unchecked_mut(DATA_LENGTH + input_neuron_index as usize) += value;
                                        }*/
                }
            });

        let mut length_index = 0u32;
        /*unsafe {
            for _tick in 0..MAX_INPUT_DURATION {
                let mut neuron_indices: [u16; NUMBER_OF_INPUT_NEURONS + INFO_LENGTH] = std::array::from_fn(|i| i as u16);
                let mut number_of_remaining_neurons = (NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) as u16;

                while number_of_remaining_neurons > 0 {
                    let neuron_index_index = *neuron_data.synapses.lengths.get_unchecked(length_index as usize) % number_of_remaining_neurons;
                    let input_neuron_index = *neuron_indices.get_unchecked(neuron_index_index as usize);

                    length_index += 1;
                    number_of_remaining_neurons -= 1;

                    *neuron_indices.get_unchecked_mut(neuron_index_index as usize) = *neuron_indices.get_unchecked(number_of_remaining_neurons as usize);

                    for another_input_neuron_index in 0..DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH {
                        let mut value: i32 = (neuron_data.neurons.input[another_input_neuron_index] >> 31) | 1;

                        value *= *neuron_data.synapses.input.get_unchecked((input_neuron_index as usize * (DATA_LENGTH + NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) + another_input_neuron_index)) as i32;
                        *neuron_data.neurons.input.get_unchecked_mut(DATA_LENGTH + input_neuron_index as usize) += value;
                    }
                }
            }
        }*/

        neuron_data.neurons.output.iter_mut().zip(neuron_data.neurons.input.iter_mut().skip(DATA_LENGTH + NUMBER_OF_INPUT_NEURONS))
            .take(INFO_LENGTH)
            .for_each(|(output, input)| {
                *output = *input;
            });

        length_index = ((NUMBER_OF_INPUT_NEURONS + INFO_LENGTH) * MAX_INPUT_DURATION) as u32;
        unsafe {
            for _tick in 0..MAX_OUTPUT_DURATION {
                let mut neuron_indices: [u16; NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH] = std::array::from_fn(|i| i as u16);
                let mut number_of_remaining_neurons = (NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) as u16;

                while number_of_remaining_neurons > 0 {
                    let neuron_index_index = *neuron_data.synapses.lengths.get_unchecked(length_index as usize) % number_of_remaining_neurons;
                    let output_neuron_index = *neuron_indices.get_unchecked(neuron_index_index as usize);

                    length_index += 1;
                    number_of_remaining_neurons -= 1;

                    *neuron_indices.get_unchecked_mut(neuron_index_index as usize) = *neuron_indices.get_unchecked(number_of_remaining_neurons as usize);

                    let mut result_32 = std::simd::i32x32::splat(neuron_data.neurons.output[INFO_LENGTH + output_neuron_index as usize]);
                    for another_output_neuron_index in (0..INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH).step_by(32) {
                        let neurons_output_32 = std::simd::i32x32::from_slice(as_mut_slice(&mut neuron_data.neurons.output[another_output_neuron_index], 32));
                        let mut value_32 = (neurons_output_32 >> std::simd::i32x32::splat(31)) | std::simd::i32x32::splat(1);
                        let neurons_output_32 = std::simd::i32x32::from_slice(as_mut_slice(&mut neuron_data.synapses.output[(output_neuron_index as usize * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) + another_output_neuron_index)], 32));
                        value_32 *= neurons_output_32;
                        result_32 += value_32;

                        *neuron_data.neurons.output.get_unchecked_mut(INFO_LENGTH + output_neuron_index as usize) = result_32.reduce_sum();

                        /*                        let mut value: i32 = (*neuron_data.neurons.output.get_unchecked(another_output_neuron_index) >> 31) | 1;
                        
                                                value *= *neuron_data.synapses.output.get_unchecked((output_neuron_index as usize * (INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS + DATA_LENGTH) + another_output_neuron_index)) as i32;
                                                *neuron_data.neurons.output.get_unchecked_mut(INFO_LENGTH + output_neuron_index as usize) += value;
                        */
                    }
                }
            }
        }

        let score = self.mining_data.iter()
            .zip(neuron_data.neurons.output.iter().skip(INFO_LENGTH + NUMBER_OF_OUTPUT_NEURONS))
            .filter(|(data_item, neuron_item)| {
                (**data_item >= 0) == (**neuron_item >= 0)
            })
            .count();

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