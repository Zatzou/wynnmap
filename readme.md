# Wynnmap

A fast Wynncraft map. Currently available at [wynnmap.zatzou.com](https://wynnmap.zatzou.com).

## Running locally

There are two components as of right now, the backend server and the frontend. To get started, you will need to install [Rust](https://rustup.rs/) and [Trunk](https://trunkrs.dev/). Npm is also required to use the tailwindcss cli. The tailwind cli can be installed using `npm install tailwindcss @tailwindcss/cli`.

You can build and serve the frontend for development using `trunk serve` in the root of the project. The frontend will be available at `http://localhost:8080`. Note that by default the frontend will use the production backend server. If you wish to do backend development you will need to change the proxied urls in `Trunk.toml`.

To run the backend server, navigate to the `wynnmap-server` directory and run `cargo run`. The server will be available at the port configured in the config.toml file. An example configuration file is provided in the repository. If configured correctly the backend server should be able to serve the frontend as well.

## Building using release optimizations

In order to build release optimized binaries you will need to build the frontend using `trunk build --release` and the backend using `cargo build --release`. The frontend will be available in the `dist` directory and the backend will be available in the `target/release` directory. These can then be set up to run in a production environment.

### Note about the backend api url

By default the frontend assumes the backend server is available at `/api`. If you are running the backend server at a different location you will need to change the code in [datasource.rs](/src/datasource.rs) to use a different url. Specifically the `get_url` function can be edited to use a hardcoded url instead of using the `window.location` value.

# Acknowledgements

This project uses map tile assets and icons from the [Wynntils](https://github.com/Wynntils) project. These assets are used under the LGPL license.
