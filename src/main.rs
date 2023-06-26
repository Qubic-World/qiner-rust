use qiner::miner::{Miner};
use tokio;
use qiner::types::{Id, PublicKey64, STACK_SIZE};
use std::{env};
use std::mem::{size_of, transmute};
use std::sync::Arc;
use tokio::runtime::Builder;
use qiner::converters::get_public_key_64_from_id;
use qiner::env_names::{ID, NUMBER_OF_TASKS, NUMBER_OF_THREADS, SERVER_IP, SERVER_PORT};
use qiner::network::Packet;
use qiner::types::network::protocols::BROADCAST_MESSAGE;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use qiner::version::get_version;

fn get_number_of_thread() -> usize {
	env::var(NUMBER_OF_THREADS).unwrap().parse::<usize>().unwrap()
}

fn get_number_of_tasks() -> usize {
	env::var(NUMBER_OF_TASKS).unwrap_or_default()
							 .parse::<usize>().unwrap_or(1)
}

fn get_server_ip() -> String {
	env::var(SERVER_IP).unwrap_or_default()
}

fn get_server_port() -> String {
	env::var(SERVER_PORT).unwrap_or_default()
}

fn get_id() -> String {
	env::var(ID).unwrap_or_default()
}

fn main() {
	// Init dotenv
	dotenv::dotenv().ok();

	pretty_env_logger::init_timed();

	let number_of_threads = get_number_of_thread();
	let stack_size = STACK_SIZE * (number_of_threads + 3);

	Builder::new_multi_thread()
		.worker_threads(number_of_threads + 3)
		.thread_stack_size(stack_size)
		.enable_all()
		.build().unwrap()
		.block_on(async {
			async_main().await;
		});
}

async fn async_main() {

	// Grab info
	let number_of_threads = get_number_of_thread();
	let ip_raw = get_server_ip();
	let port_raw = get_server_port();
	let id_raw = get_id();

	// Display info
	log::info!("Version: {:?}", get_version());
	log::info!("IP address: {ip_raw}");
	log::info!("Port: {port_raw}");
	log::info!("Id: {id_raw}");
	log::info!("Available cores: {}", num_cpus::get());
	log::info!("Number of threads: {}", number_of_threads);

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
	let sent_score_counter = Arc::new(tokio::sync::Mutex::new(0usize));

	let arc_miner_clone = arc_miner.clone();
	let sent_score_counter_clone = sent_score_counter.clone();
	let handle = tokio::spawn(async move {
		let mut prev_iter_value: usize = 0;

		loop {
			log::info!("{} scores | sent scores {} | {} it/s", arc_miner_clone.get_score(), sent_score_counter_clone.lock().await, arc_miner_clone.get_iter_counter() - prev_iter_value);
			prev_iter_value = arc_miner_clone.get_iter_counter();

			tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
		}
	});

	// TCP client task
	let arc_miner_clone = arc_miner.clone();
	let sent_score_counter_clone = sent_score_counter.clone();
	tokio::spawn(async move {
		loop {
			let is_nonce_exists;
			{
				is_nonce_exists = !arc_miner_clone.found_nonce.lock().await.is_empty();
			}

			if is_nonce_exists {
				let addr = format!("{ip_raw}:{port_raw}");

				log::info!("Connecting to {addr}");
				let mut stream_result = TcpStream::connect(addr).await;

				match stream_result.as_mut() {
					Err(err) => {
						log::error!("Failed to connect: {:?}", err);
					}
					Ok(stream) => {
						// Wait for the socket to be writable
						if let Err(err) = stream.writable().await {
							log::error!("Writable: {:?}", err);
						} else {
							// Grab data
							let data_for_send;
							{
								let found_nonce = arc_miner_clone.found_nonce.lock().await;
								data_for_send = found_nonce.iter().map(|nonce| {
									let packet = Packet::new(&BROADCAST_MESSAGE, &public_key, &nonce);
									unsafe { transmute::<Packet, [u8; size_of::<Packet>()]>(packet) }
								}).collect::<Vec<[u8; size_of::<Packet>()]>>().into_iter().flatten().collect::<Vec<u8>>();
							}

							let packet_num = data_for_send.len() / size_of::<Packet>();
							log::info!("TCP: will be sent {packet_num} packets({} Bytes)", data_for_send.len());

							// Send data
							log::info!("TCP: send data...");
							let write_result = stream.write_all(data_for_send.as_slice()).await;
							if let Err(err) = write_result {
								log::error!("Failed to send data: {:?}", err);
							} else {
								let mut lock = sent_score_counter_clone.lock().await;
								*lock += packet_num;
							}

							// Deleting nonce that have been sent
							arc_miner_clone.found_nonce.lock().await.drain(0..packet_num);
						}
					}
				}
			}

			tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
		}
	});


	handle.await.unwrap();
	println!("End");
}
