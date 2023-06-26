# Qiner on Rust

## Deploy

### Download Rust  
  
#### Windows  
  
[x64](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)  
[x32](https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe)  
[Visual Studio C++ Build tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

#### Linux

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Building Qiner

1. Open the terminal in the end folder Qiner (the folder where the `Cargo.toml` file is)
2. `cargo build --release`

The built Qiner is `./root_directory/target/release/`

### Starting Qiner

#### .env

The options to run Qiner are in the `.env` file

1. Create a `.env' next to the built Qiner
2. Fill in the following options: RUST_LOG, NUMBER_OF_THREADS, ID, SERVER_IP, SERVER_PORT, VERSION

#### RUST_LOG

Set to `INFO` to see the output in the console
Read more at the [link](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging)

#### NUMBER_OF_THREADS

Number of threads to be mined on

##### ID

Qiner ID of 60 characters

#### SERVER_IP and SERVER_PORT

IP and Port to which Qiner will connect

#### VERSION

Qubic version


##### Example

```
RUST_LOG=INFO
NUMBER_OF_THREADS=8
ID=UBAZRCVPOZTDKGCBNPGYFUPLZXDDNHSEGJRTAJKWJBHJDKHMAKVVFAKCZGRI
SERVER_IP=176.9.28.103
SERVER_PORT=21841
VERSION=1.142.1
```
