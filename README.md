# pankosmia_web
A web server for pankosmia desktop applications

## Running in isolation
```
cargo run # Creates a pankosmia_working directory at the root of the user directory
```

#### Tested on Ubuntu 24.04 with:
- npm 9.2.0
- node 18.19.1
- rustc 1.83.0 -- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

#### Tested on Windows 11 with:
- npm 10.7.0
- node 18.20.4
- rustc 1.83.0 -- See https://www.rust-lang.org/tools/install
- cmake 3.31.0 -- See https://cmake.org/download/

## Using within Tauri
See the Pithekos repo for example code.

## Usage
Connect to localhost:8000 to see the (extremely basic) default client

## Using other clients
- create or download a client
- build that client (compiled code should be in `build` directory)
- modify the `app_settings.json` file in the `settings` directory
- restart the server

