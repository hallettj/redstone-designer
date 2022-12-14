# Redstone Designer

This is an experimental pet project. Don't worry about it! But if you want to
see some Bevy code, and you're ok with looking at code written by someone just
getting started then please feel free to take a look.

## Prerequisites

To get the necessary Rust toolchain and system libraries automatically use Nix.
Follow the instructions in [flake.nix](./flake.nix). Otherwise install the
dependencies listed below. You need to get Minecraft assets manually either way.

If you are installing dependencies manually you will need:

- the Rust nightly toolchain
- OS dependencies listed [here](https://bevyengine.org/learn/book/getting-started/setup/#install-os-dependencies)

On Linux you also need these dependencies for Wayland support:

- libwayland-dev
- libxkbcommon-dev

Finally you will need a copy of Minecraft.

### Rust toolchain

Either use Nix or,

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
