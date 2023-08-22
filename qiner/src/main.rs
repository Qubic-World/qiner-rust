use lib::env_names::{ENV_ID, ENV_NUMBER_OF_THREADS, ENV_SERVER_IP, ENV_SERVER_PORT};
use lib::random_seed::get_random_seed;
use lib::solution_threshold::get_solution_threshold;
use lib::types::{Id, PublicKey64, STACK_SIZE};
use qiner::miner::{Miner};

use std::sync::Arc;
use std::{env};
use qiner::converters::get_public_key_64_from_id;

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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .disable_lifo_slot()
        .worker_threads(number_of_threads + 1)
        .enable_all()
        .thread_stack_size(number_of_threads * STACK_SIZE)
        .build().unwrap();


    runtime.block_on(
        async move {
            async_main().await;
        }
    );
}

async fn async_main() {
    log::info!("Async main thread id: {:?}", std::thread::current().id());

    let number_of_threads = get_number_of_thread();
    let ip_raw = get_server_ip();
    let port_raw = get_server_port();
    let id_raw = get_id();
    let random_seed = get_random_seed();
    let solution_threshold = get_solution_threshold();

    println!("Random seed: {:?}", random_seed);
    println!("Solution threshold: {:?}", solution_threshold);
    println!("IP address: {ip_raw}");
    println!("Port: {port_raw}");
    println!("Id: {id_raw}");
    println!("Available cores: {}", num_cpus::get());
    println!("Number of threads: {}", number_of_threads);

    // Convert ID
    let id: Id = id_raw.as_bytes().try_into().unwrap();

    // Get Public key
    let mut public_key: PublicKey64 = Default::default();
    if get_public_key_64_from_id(&id, &mut public_key) == false {
        log::error!("The Id is invalid!");
        return;
    }

    let arc_miner = Arc::new(Miner::new(public_key, number_of_threads));
    Miner::run(&arc_miner);

    // Display task
    let arc_miner_clone = arc_miner.clone();
    let display_info_future = async move {
        let mut prev_iter_value: usize = 0;

        loop {
            println!("{} scores {} it/s", arc_miner_clone.get_score(), arc_miner_clone.get_iter_counter() - prev_iter_value);
            prev_iter_value = arc_miner_clone.get_iter_counter();

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    };

    tokio::join!(
        display_info_future
    );

    println!("Qiner is shutting down");
}
