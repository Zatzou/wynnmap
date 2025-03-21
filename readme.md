# Wynnmap

A fast Wynncraft map (currently beats the competition). Currently available at [wynnmap.zatzou.com](https://wynnmap.zatzou.com).

# Running locally
There are two components as of right now, the backend server and the frontend. To get started, you will need to [install Rust-lang](https://www.rust-lang.org/tools/install) and then [Trunk](https://trunkrs.dev/).

1) Clone the Git repository using `git clone https://github.com/Zatzou/wynnmap`
2) `cd wynnmap` to change directory into the cloned directory
3) Move `wynnmap-server/config-example.toml` to `wynnmap-server/config.toml`.
4) Open two terminals.
5) On terminal #1, run the command `trunk serve`. This serves the frontend.
6) On terminal #2, run `cd wynnmap-server` then `cargo run`. This serves the backend.
7) The map should now be accessible at `localhost:8080`.

# Building using release optimizations
1) Run `trunk build --release` to generate an optimized frontend.
2) Run `cargo build --release` in `wynnmap-server` in order to generate a release binary,