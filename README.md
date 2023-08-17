# Qiner on Rust

## !!!Important!!!

Sending solutions over the network is **disabled** in the **new_algo** branch

## Deploy

### Download Rust

#### Windows

1. [Rust x64](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) / [Rust x32](https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe)  
2. [Visual Studio C++ Build tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

#### Linux

```bash
sudo apt update
sudo apt install build-essential cargo -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo sh
```

### Building Qiner

1. Open the terminal in the end folder Qiner (the folder where the `Cargo.toml` file is)
2. `cargo build --release`

The built Qiner is `./root_directory/target/release/`

### Starting Qiner

#### .env

The options to run Qiner are in the `.env` file

1. Create a `.env' next to the built Qiner
2. Fill in the following options: `NUMBER_OF_THREADS`, `ID`, `SERVER_IP`, `SERVER_PORT`, `RANDOM_SEED`, `SOLUTION_THRESHOLD`

#### NUMBER_OF_THREADS

Number of threads to be mined on

##### ID

Qiner ID of 60 characters

#### SERVER_IP and SERVER_PORT

IP and Port to which Qiner will connect

##### Example

```
NUMBER_OF_THREADS=8
ID=UBAZRCVPOZTDKGCBNPGYFUPLZXDDNHSEGJRTAJKWJBHJDKHMAKVVFAKCZGRI
SERVER_IP=176.9.28.103
SERVER_PORT=21841
RANDOM_SEED=0,0,0,0,0,0,0,0
SOLUTION_THRESHOLD=600
```
