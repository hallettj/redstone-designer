# Redstone Designer

This is an experimental pet project. Don't worry about it! But if you want to
see some Bevy code, and you're ok with looking at code written by someone just
getting started then please feel free to take a look.

## Prerequisites

To run this thing you basically need two things:

- the Rust nightly toolchain
- a copy of Minecraft's game assets
- OS dependencies listed [here](https://bevyengine.org/learn/book/getting-started/setup/#install-os-dependencies)

Wayland dependencies:
- libwayland-dev
- libxkbcommon-dev

### Rust toolchain

- install `rustup` by following [these instructions](https://www.rust-lang.org/tools/install)
- add the nightly channel by running `$ rustup install nightly`

### Minecraft assets

Assets should be unpacked under the `assets/minecraft/` directory in this
project. If you have Minecraft installed in the usual location you can do that
with this command:

    $ unzip ~/.minecraft/versions/1.19.2/1.19.2.jar 'assets/**/*' -d assets/minecraft/

## Running

After you have met the prerequisites, run the app with:

    $ cargo run
