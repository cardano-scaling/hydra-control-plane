{
  description = "Hydra control plane";
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";

    cardano-node.url = "github:intersectmbo/cardano-node/10.1.4";
    hydra.url = "github:cardano-scaling/hydra/0.19.0";
  };

  outputs = { self, nixpkgs, utils, naersk, cardano-node, hydra, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        packages.hydra-control-plane = naersk-lib.buildPackage ./.;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Runtime dependencies
            cardano-node.packages.${system}.cardano-cli
            hydra.packages.${system}.hydra-node
            # Rust build tools
            pkgs.rustc
            pkgs.cargo
            pkgs.rust-analyzer
            pkgs.pre-commit
            pkgs.rustPackages.clippy
            # Libraries
            pkgs.pkg-config
            pkgs.openssl
          ];
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      });
}
