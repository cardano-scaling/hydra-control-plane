{
  description = "Hydra control plane";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    cardano-node.url = "github:intersectmbo/cardano-node/9.0.0";
    hydra.url = "github:cardano-scaling/hydra/0.17.0";

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        gitignore.follows = "gitignore";
      };
    };
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
      pre-commit-hooks,
      ...
    }:
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

        checks.pre-commit = pre-commit-hooks.lib.${system}.run {
          src = ./.;

          hooks = {
            nixfmt-rfc-style = {
              enable = true;
            };

            rustfmt = {
              enable = true;

              packageOverrides = {
                cargo = rust;
                rustfmt = rust;
              };
            };
          };
        };
      in
      {
        packages.default = platform.buildRustPackage {
          name = "hydra-control-plane";
          src = ./.;
          buildInputs =
            with pkgs;
            (
              [
                pkg-config
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
        };

        devShells.default = mkShell {
          inherit (checks.pre-commit) shellHook;

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
