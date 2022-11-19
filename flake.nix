# This is a Nix flake. It defines exact versions for project dependencies such
# as the rust compiler, cargo, and system libraries. This flake also acts as
# a build script. You can use Nix to install dependencies automatically.
# Dependencies are scoped to this project so they don't interfere with your
# system, and there is nothing to uninstall later. You can optionally use Nix to
# build the project for you, or you can run the usual Cargo commands yourself.
#
# For instructions on how to install Nix with support for flakes, and for some
# background on how to work with flakes see
# https://serokell.io/blog/practical-nix-flakes#getting-started-with-nix

{
  # Inputs are libraries and package sets that are loaded by URL. `inputs` is an
  # "attribute set" which is the Nix term for a dictionary.
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      # This says we want the code from fenix to use the same version of nixpkgs
      # that we use in other parts of this flake.
      # `inputs.nixpkgs.follows = "nixpkgs"` is syntactic suger for `inputs = { nixpkgs = { follows = "nixpkgs" }}`
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # `flake-utils.url = ...` is syntactic sugar for `flake-utils = { url = ... }`
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  # `outputs` is a function that returns another attribute set (dictionary).
  # Attributes in the attribute set define things like build instructions for an
  # installable package based on this project (`defaultPackage`), and
  # instructions for setting up a reproducible development environment
  # (`devShell`). To see exactly what `output` returns you can run,
  #
  #     $ nix flake show
  #
  # The arguments to `output` are results from processing `inputs`. Inputs are
  # processed by fetching the given URL for each input, (subject to locked
  # revisions specified in flake.lock), and evaluating a `flake.nix` file in the
  # downloaded file set. Properties on each input here come from outputs from
  # those evaluated flakes. For example, `fenix.packages` corresponds to
  # the `packages` attribute in the outputs of the flake at
  # https://github.com/nix-community/fenix/blob/main/flake.nix
  #
  # `self` is self-reference to the return value of `outputs`. Thanks to lazy
  # evaluation it is possible to refer to that return value before it is
  # returned.
  outputs = { self, fenix, flake-utils, naersk, nixpkgs }:
    # `eachDefaultSystem` iterates over these strings to produce outputs for
    # each corresponding system with minimal boilerplate:
    # - aarch64-linux
    # - aarch64-darwin 
    # - x86_64-darwin 
    # - x86_64-linux
    #
    # For a list of other systems we could potentially build outputs for see,
    # https://github.com/numtide/flake-utils/blob/master/default.nix 
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Nixpkgs is a repository of general-purpose software packages. Think of
        # it as like the official Debian or Arch repositories, or like Homebrew.
        #
        # See the note at the end of this file to understand what the `import`
        # operator does here, and why we have to import this input but not the
        # others. The upshot is that this code imports the default export (a
        # function) from the nixpkgs "derivation", and calls that function. The
        # return value of the function is an attribute set (dictionary) with
        # attributes for every package available in nixpkgs.
        pkgs = (import nixpkgs) {
          # `inherit system` is syntactic sugar for `system = system`. In other
          # words this takes the `system` variable above, and passes it as an
          # argument to nixpkgs. That is how we get the set of packages for the
          # specific system we are working on.
          inherit system;

          # Overlays override packages from nixpkgs in case we need a package
          # version that is different than what is provided in nixpkgs, or we
          # need a package that isn't in the base package set.
          overlays = [ ];
        };

        # Get the Rust toolchain from the nightly channel. To update the
        # toolchain, update the flake.lock entry for fenix with,
        #
        #     $ flake lock --update-input fenix
        #
        # `toolchain` evaluates to a single Nix package that includes all of the
        # requested components.
        toolchain = fenix.packages.${system}.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src" # used by rust-analyzer
          "rustc"
          "rustfmt"
        ];

        # Naersk automatically generates instructions for fetching and building
        # Rust crates based on the contents of Cargo.toml and Cargo.lock. This
        # call configures a `buildPackage` function that is called below.
        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        # Required system libraries and build tools are listed here. Versions
        # are whatever is in the locked version of nixpkgs. (The locked git rev
        # is written into flake.lock). We can update all of these at once by
        # updating nixpkgs with:
        #
        #     $ nix flake lock --update-input nixpkgs
        #
        # To install an input at a version other than what is in nixpkgs it is
        # necessary to add to the `overlays` attribute in the argument to
        # `nixpkgs` above.
        #
        # The syntax `with pkgs` brings all of the attributes of `pkgs` into
        # scope for the rest of the expression. That lets us write, e.g.,
        # `alsa-lib` instead of `pkgs.alsa-libs`.
        nativeBuildInputs = with pkgs; [
          alsa-lib
          pkg-config
          udev
        ] ++ lib.optionals stdenv.isLinux [
          # These packages are only installed when building for Linux.
          libxkbcommon # required with bevy's `wayland` feature
          wayland # required with bevy's `wayland` feature
        ];

      in
      {
        # Build the package with `nix build`, or run the built binary with `nix
        # run`. Both commands read this `defaultPackage` output for the system
        # they are run on. The build is created in a temp directory; the built
        # package is written to a directory under /nix/store/
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          inherit nativeBuildInputs;
        };

        # Run `nix develop`, or use direnv with nix-direnv, to get a development
        # shell with the exact Rust version and dependency versions defined in
        # this file.
        #
        # Because Naersk reads from Cargo.lock, building with `cargo build
        # --release` in the development shell should get exactly the same result
        # as running `nix build`. (Except that `cargo build` outputs to target/,
        # while `nix build` outputs to the Nix store.)
        #
        # During development you can use the normal Cargo workflow in the
        # development shell.
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ toolchain ] ++ nativeBuildInputs;
        };
      }
    );
}

# You might have noticed that the variables above, `lib` and `stdenv`, are not
# bound anywhere in this file. Those seem to be injected by some part of the
# Nix system. I think `lib` might be the Nix standard library - or possibly
# a flake-specific standard library. The flake wiki page doesn't say anything
# about either of these values.

# So why do we have to write `import nixpkgs`, but we don't need to import the
# other flake inputs? Every flake is a "derivation" which means that it is
# written to a path in the local filesystem in the Nix store (under
# /nix/store/), and it automatically gets a attribute called `.outPath` which is
# the store path where the derivation is unpacked. That applies the flake
# defined in this file, and to all of its inputs. So we could read
# `nixpkgs.outPath`, and it would evaluate to `/nix/store/<some
# hash>-nixpkgs/nixpkgs/`. The outPath attribute is special in two ways:
#
# 1. When you coerce a derivation to a string the result is its `.outPath`
#    attribute value.
# 2. You can `import` a value with an `.outPath`. (I think this is actually
#    a consequence of point 1 since `import` takes a string.)
#
# We already saw that when flake input are loaded the system finds a file called
# `flake.nix` and evualuates it. This is a special feature of flakes - that
# behavior is not intrinsic to the Nix language. Flake inputs are not imports.
#
# What is intrinsic to the language is the `import` keyword. When given a path
# to a Nix file, `import` evaluates to the result of evaluating that file. When
# given a path to a *directory* `import` evaluates the file in that directory
# named `default.nix`.
#
# It happens that the nixpkgs repo has both a `flake.nix`, and a `default.nix`.
# When bring nixpkgs as a flake input the system evaluates `flake.nix`, and puts
# the whole repo in a directory in the local Nix store. Then calling `import
# nixpkgs` evaluates `default.nix` from that directory which evaluates to the
# function we call to get references to all of the system packages.
#
# Why is nixpkgs designed so that we have to load `default.nix`? Why is the
# package attribute set not provided as an output in its `flake.nixe?`. I don't
# know. I think it might have to do with avoiding eager evaluate of the entire
# set of packages, but I'm not sure.
