# https://flake.parts/index.html

{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    systems.url = "github:nix-systems/default";
    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-substituters = [ "https://nix-community.cachix.org" ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs"
    ];
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      systems,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;
      perSystem =
        { pkgs, system, ... }:
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [ inputs.fenix.overlays.default ];
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              (pkgs.fenix.combine [
                pkgs.fenix.complete.cargo
                pkgs.fenix.complete.rustc
                pkgs.fenix.complete.rust-src
                pkgs.fenix.complete.clippy
                pkgs.fenix.complete.rustfmt
              ])
              rust-analyzer
            ];
          };
        };
    };
}
