use std::arch::x86_64::_rdrand64_step;
use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::thread::ThreadId;
use crate::types::{
	MiningItemData,
	MiningData,
	NeuronLink,
	NeuronLinks64,
	NeuronValue,
	NeuronValues,
	Nonce64,
	PublicKey64,
	Seed,
	Seed64,
	SOLUTION_THRESHOLD,
	MINING_DATA_LENGTH,
	NEURON_MOD_BITS,
	NUMBER_OF_NEURONS,
	NUMBER_OF_NEURONS_64,
};

#[derive(Debug, Clone)]
pub struct NeuronContainer {
	neuron_data: HashMap<ThreadId, NeuronData>,
}

impl NeuronContainer {
	pub fn get_mut_data(&mut self, thread_id: &ThreadId) -> &mut NeuronData {
		if !self.neuron_data.contains_key(thread_id) {
			self.neuron_data.insert(thread_id.clone(), NeuronData::default());
		}

		self.neuron_data.get_mut(thread_id).unwrap()
	}
}

#[derive(Debug, Clone)]
pub struct NeuronData {
	neuron_links: NeuronLinks64,
	neuron_values: NeuronValues,
}

impl Default for NeuronData {
	fn default() -> Self {
		NeuronData {
			neuron_links: [0; NUMBER_OF_NEURONS_64 * 2],
			neuron_values: [NeuronValue::MAX; NUMBER_OF_NEURONS],
		}
	}
}


#[derive(Debug, Clone)]
pub struct Miner {
	solution_threshold: usize,
	num_tasks: usize,

	mining_data: MiningData,
	computor_public_key: PublicKey64,

	score_counter: Arc<AtomicUsize>,
	iter_counter: Arc<AtomicUsize>,

	pub found_nonce: Arc<tokio::sync::Mutex<Vec<Nonce64>>>,
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
			solution_threshold: SOLUTION_THRESHOLD,
			num_tasks: num_threads,
			mining_data,
			computor_public_key,
			score_counter: Arc::new(AtomicUsize::new(0)),
			iter_counter: Arc::new(AtomicUsize::new(0)),
			found_nonce: Arc::new(tokio::sync::Mutex::new(Vec::new())),
		}
	}

	pub fn get_score(&self) -> usize {
		self.score_counter.load(Ordering::SeqCst)
	}

	pub fn get_iter_counter(&self) -> usize {
		self.iter_counter.load(Ordering::SeqCst)
	}

	fn get_random_seed() -> Seed {
		let mut random_seed: Seed = Seed::default();
		random_seed[0] = 74;
		random_seed[1] = 27;
		random_seed[2] = 26;
		random_seed[3] = 27;
		random_seed[4] = 26;
		random_seed[5] = 27;
		random_seed[6] = 26;
		random_seed[7] = 27;

		random_seed
	}

	fn get_random_seed_64() -> Seed64 {
		let seed = Miner::get_random_seed();

		let seed_64: Seed64;
		unsafe {
			seed_64 = std::mem::transmute::<Seed, Seed64>(seed);
		}

		seed_64
	}

	pub fn find_solution(&self, nonce: &mut Nonce64, neuron_data: &mut NeuronData) -> bool {
		nonce.iter_mut().for_each(|item| { unsafe { _rdrand64_step(item) }; });

		crate::math::random_64(&self.computor_public_key, nonce, &mut neuron_data.neuron_links);

		for idx in 0..NUMBER_OF_NEURONS_64 {
			neuron_data.neuron_links[idx] &= NEURON_MOD_BITS;
			neuron_data.neuron_links[NUMBER_OF_NEURONS_64 + idx] &= NEURON_MOD_BITS;
		}

		let mut limit = MINING_DATA_LENGTH;
		let mut score: usize = 0;

		loop {
			let prev_value0 = neuron_data.neuron_values[NUMBER_OF_NEURONS - 1];
			let prev_value1 = neuron_data.neuron_values[NUMBER_OF_NEURONS - 2];

			for idx in 0..NUMBER_OF_NEURONS_64 {
				let idx_left = idx * 2; //[j][0]
				let idx_right = idx * 2 + 1; // [j][1]
				let value_idx = idx * 2;

				let nv_l0 = (neuron_data.neuron_links[idx_left] as NeuronLink) as usize; // neuronValues[neuronLinks[j][0]]
				let nv_r0 = ((neuron_data.neuron_links[idx_left] >> size_of::<NeuronLink>() * 8) as NeuronLink) as usize; // neuronValues[neuronLinks[j][1]]

				let nv_l1 = (neuron_data.neuron_links[idx_right] as NeuronLink) as usize; // neuronValues[neuronLinks[j + 1][0]]
				let nv_r1 = ((neuron_data.neuron_links[idx_right] >> size_of::<NeuronLink>() * 8) as NeuronLink) as usize; // neuronValues[neuronLinks[j + 1][1]]

				let and_result0 = neuron_data.neuron_values[nv_l0] & neuron_data.neuron_values[nv_r0];
				let and_result1 = neuron_data.neuron_values[nv_l1] & neuron_data.neuron_values[nv_r1];
				neuron_data.neuron_values[value_idx] = !(and_result0);
				neuron_data.neuron_values[value_idx + 1] = !(and_result1);
			}

			let current_value0 = neuron_data.neuron_values[NUMBER_OF_NEURONS - 1];
			let current_value1 = neuron_data.neuron_values[NUMBER_OF_NEURONS - 2];

			let data_of_mining = self.mining_data[score >> 6];
			let is_bit_set = ((data_of_mining >> (score & 63) as MiningItemData) & 1) as u8;
			if current_value0 != prev_value0 && current_value1 == prev_value1 {
				if is_bit_set == 0 {
					break;
				}

				score += 1;
			} else if current_value1 != prev_value1 && current_value0 == prev_value0 {
				if is_bit_set == 1 {
					break;
				}

				score += 1;
			} else {
				limit -= 1;

				if limit == 0 {
					break;
				}
			}
		}

		score >= self.solution_threshold
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
					log::debug!("[{idx_clone}]Find solution in Thread Id ({:?})", thread::current().id());

					if arc_miner_clone.find_solution(&mut nonce, &mut neuron_data) {
						arc_miner_clone.score_counter.fetch_add(1, Ordering::Relaxed);
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
