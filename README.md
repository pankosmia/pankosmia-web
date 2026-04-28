# se-wiki

Multi-tenant web server for collaborative scripture/translation projects.

> Forked from [pankosmia/pankosmia-web](https://github.com/pankosmia/pankosmia-web).
> See [LICENSE](./LICENSE) for original copyright. The fork diverges architecturally
> (Supabase-backed auth, per-project isolation, Docker deployment) and does not
> intend to send changes back upstream — upstream improvements may be cherry-picked
> in selectively.

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
- cmake 3.31.0 -- _Version 3 is required._ See https://cmake.org/download/

#### Tested on MacOS with:
- npm 10.7.0 (tested on Monterey)
- npm 10.8.2 (tested on Sequoia)
- node 18.20.4
- rustc 1.86.0 -- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- OpenSSL 3.5.0 -- `brew install openssl`

## Using within Tauri
See the Pithekos repo for example code.

## Usage
Connect to localhost:8000 to see the (extremely basic) default client

## Using other clients
- create or download a client
- build that client (compiled code should be in `build` directory)
- modify the `app_settings.json` file in the `settings` directory
- restart the server

