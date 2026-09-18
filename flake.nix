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
      fenix,
      nixpkgs,
      systems,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;
      perSystem =
        { pkgs, system, ... }:
        let
          cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
          bin = cargoToml.package.name;

          muslCc = pkgs.pkgsStatic.stdenv.cc;
          mingwCc = pkgs.pkgsCross.mingwW64.stdenv.cc;

          rustToolchain = pkgs.fenix.combine [
            pkgs.fenix.complete.cargo
            pkgs.fenix.complete.rustc
            pkgs.fenix.complete.rust-src
            pkgs.fenix.complete.clippy
            pkgs.fenix.complete.rustfmt
            pkgs.fenix.targets.x86_64-unknown-linux-musl.latest.rust-std
            pkgs.fenix.targets.x86_64-pc-windows-gnu.latest.rust-std
          ];

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          staticRustPlatform = pkgs.pkgsStatic.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          mingwRustPlatform = pkgs.pkgsCross.mingwW64.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          common = {
            pname = bin;
            version = cargoToml.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          # `-Zbuild-std` also resolves the standard library's own dependencies,
          # so the crates pinned by rust-src's library have to be vendored in
          # addition to the ones from the project's Cargo.lock.
          cargoDeps = pkgs.runCommand "${bin}-cargo-deps" { } ''
            mkdir -p $out
            cp -rs ${rustPlatform.importCargoLock { lockFile = ./Cargo.lock; }}/. $out/
            chmod -R u+w $out
            for dep in ${
              rustPlatform.importCargoLock {
                lockFile = "${rustToolchain}/lib/rustlib/src/rust/library/Cargo.lock";
              }
            }/*/; do
              name="$(basename "$dep")"
              [ -e "$out/$name" ] || cp -r "$dep" "$out/$name"
            done
            # Let the cargo setup hook provide the vendored-sources config.
            rm -rf $out/.cargo
          '';

          muslPackage = staticRustPlatform.buildRustPackage (
            common
            // {
              doCheck = false;
              auditable = false;
              inherit cargoDeps;
              # nixpkgs only strips debug info from bin by default; strip the
              # whole symbol table too.
              stripAllList = [ "bin" ];
            }
          );

          windowsPackage = mingwRustPlatform.buildRustPackage (
            common
            // {
              doCheck = false;
              auditable = false;
              inherit cargoDeps;
              stripAllList = [ "bin" ];
              # AWS-LC's portable/JIT entropy backend includes POSIX headers
              # unavailable on mingw, and its assembly is pre-generated for
              # Windows so nasm is not needed when cross-compiling.
              AWS_LC_SYS_PREBUILT_NASM = "1";
              AWS_LC_SYS_NO_JITTER_ENTROPY = "1";
            }
          );
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [ inputs.fenix.overlays.default ];
          };

          packages.default = muslPackage;
          packages.musl = muslPackage;
          packages.windows = windowsPackage;

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustToolchain
              rust-analyzer
              pkg-config
              muslCc
              mingwCc
            ];

            shellHook = ''
              # Local cross builds do not go through a cross stdenv, so point
              # cargo and the C/C++ compilers at the target toolchains.
              export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=${muslCc}/bin/x86_64-unknown-linux-musl-cc
              export CC_x86_64_unknown_linux_musl=${muslCc}/bin/x86_64-unknown-linux-musl-gcc
              export CXX_x86_64_unknown_linux_musl=${muslCc}/bin/x86_64-unknown-linux-musl-g++
              export AR_x86_64_unknown_linux_musl=${muslCc}/bin/x86_64-unknown-linux-musl-ar
              export RANLIB_x86_64_unknown_linux_musl=${muslCc}/bin/x86_64-unknown-linux-musl-ranlib

              winPthreads=${pkgs.pkgsCross.mingwW64.windows.pthreads}
              export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=${mingwCc}/bin/${mingwCc.targetPrefix}cc
              export CC_x86_64_pc_windows_gnu=${mingwCc}/bin/${mingwCc.targetPrefix}gcc
              export CXX_x86_64_pc_windows_gnu=${mingwCc}/bin/${mingwCc.targetPrefix}g++
              export AR_x86_64_pc_windows_gnu=${mingwCc}/bin/${mingwCc.targetPrefix}ar
              export RANLIB_x86_64_pc_windows_gnu=${mingwCc}/bin/${mingwCc.targetPrefix}ranlib
              export AWS_LC_SYS_PREBUILT_NASM=1
              export AWS_LC_SYS_NO_JITTER_ENTROPY=1
              export CFLAGS_x86_64_pc_windows_gnu="-I$winPthreads/include"
              export CXXFLAGS_x86_64_pc_windows_gnu="-I$winPthreads/include"
              export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L native=$winPthreads/lib -C target-feature=+crt-static"
            '';
          };
        };
    };
}
