# pankosmia_web
A web server for Pankosmia applications

## Testing
Pankosmia-web expects various setup and resource files. These are typically provided by a Pankosmia "Product" such as https://github.com/pankosmia/desktop-app-pithekos . This repo includes a test config file that will use a locally-compiled linux server with resources from Pithekos within a typical Pankosmia dev configuration, ie all repos are within the same directory.

You first need to build the Pithekos app. Then
```
cargo build # Makes debug build
cd target/debug
./pankosmia_web --help # CLI options
./pankosmia_web --config-path ../../test/test_config.json # log file in the debug directory, other files under ~/pankosmia/pithekos and ~/pankosmia_repos
```
