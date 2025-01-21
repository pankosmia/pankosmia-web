# pankosmia_web
A web server for pankosmia desktop applications

## Installation
```
cd default_clients/dashboard
npm install
npm run build
cd ../settings
npm install
npm run build
cd ../..
cargo run # Creates a pankosmia_working directory at the root of the user directory
```

## Using other clients
- create or download a client
- build that client (compiled code should be in `build` directory)
- modify the `app_settings.json` file in the pankosmia_working directory
- restart the server

