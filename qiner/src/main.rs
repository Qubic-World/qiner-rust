use lib::env_names::{ENV_ID, ENV_NUMBER_OF_THREADS, ENV_SERVER_IP, ENV_SERVER_PORT};
use lib::random_seed::get_random_seed;
use lib::solution_threshold::get_solution_threshold;
use lib::types::{
    PublicKey64,
    STACK_SIZE,
};
use qiner::miner::{Miner};

use std::sync::Arc;
use std::{env, thread};

fn get_number_of_thread() -> usize {
    env::var(ENV_NUMBER_OF_THREADS)
        .unwrap()
        .parse::<usize>()
        .unwrap()
}

fn get_server_ip() -> String {
    env::var(ENV_SERVER_IP).unwrap_or_default()
}

fn get_server_port() -> String {
    env::var(ENV_SERVER_PORT).unwrap_or_default()
}

fn get_id() -> String {
    env::var(ENV_ID).unwrap_or_default()
}

fn main() {
    // Init dotenv
    dotenv::dotenv().ok();
    pretty_env_logger::init_timed();

    let number_of_threads = get_number_of_thread();
    let ip_raw = get_server_ip();
    let port_raw = get_server_port();
    let id_raw = get_id();
    let random_seed = get_random_seed();
    let solution_threshold = get_solution_threshold();

    println!("Random seed: {:?}", random_seed);
    println!("Solution threshold: {:?}", solution_threshold);
    println!("Random seed: {:?}", random_seed);
    println!("IP address: {ip_raw}");
    println!("Port: {port_raw}");
    println!("Id: {id_raw}");
    println!("Available cores: {}", num_cpus::get());
    println!("Number of threads: {}", number_of_threads);

    let public_key = PublicKey64::default();
    let miner = Arc::new(Miner::new(public_key, 1));
    for _ in 0..number_of_threads {
        let miner_clone = miner.clone();
        thread::Builder::new()
            .stack_size(STACK_SIZE)
            .spawn(move || {
                Miner::run_in_thread(miner_clone);
            })
            .unwrap();
    }

    loop {
        let mut prev_iter_value: usize = 0;

        loop {
            println!("{} scores | {} it/s", miner.get_score(), miner.get_iter_counter() - prev_iter_value);
            prev_iter_value = miner.get_iter_counter();

            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}