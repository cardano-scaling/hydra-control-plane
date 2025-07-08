{
  description = "Hydra control plane";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    cardano-node.url = "github:intersectmbo/cardano-node/10.1.4";
    hydra.url = "github:cardano-scaling/hydra/0.22.2";

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-inclusive.url = "github:input-output-hk/nix-inclusive";
  };

  nixConfig = {
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
    ];
    substituters = [
      "https://cache.nixos.org/"
      "https://cache.iog.io/"
    ];
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      cardano-node,
      hydra,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) makeRustPlatform mkShell rust-bin;
        inherit (pkgs.lib) optionals;
        inherit (pkgs.stdenv) isDarwin;

        rust = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        platform = makeRustPlatform {
          rustc = rust;
          cargo = rust;
        };
      in
      {
        packages.default = platform.buildRustPackage {
          name = "hydra-control-plane";
          src = inputs.nix-inclusive.lib.inclusive ./. [
            ./Cargo.lock
            ./Cargo.toml
            ./rust-toolchain.toml
            ./src
          ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs =
            with pkgs;
            (
              [
                openssl
              ]
              ++ optionals isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ]
            );
          cargoLock = {
            lockFile = ./Cargo.lock;

            outputHashes = {
              "pallas-0.29.0" = "sha256-P//R/17kMaqN4JGHFFTMy2gbo7k+xWUaqkF0LFVUxWQ=";
            };
          };
          meta.mainProgram = "hydra_control_plane";
        };

        devShells.default = mkShell {
          buildInputs =
            [
              # Runtime dependencies
              cardano-node.packages.${system}.cardano-cli
              hydra.packages.${system}.hydra-node

              # Rust Environment
              rust
              pkgs.pkg-config
              pkgs.openssl
            ]
            ++ optionals isDarwin [
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        };
      }
    );
}
