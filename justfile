# Variables
perf_data := justfile_directory() / "perf.data"
profile_json := justfile_directory() / "profile.json.gz"
debug_dir := justfile_directory() / "target" / "debug"
release_dir := justfile_directory() / "target" / "release"
bin := "tinyassist"

_default:
  @just --list --unsorted

# Format, check, lint, and test
check:
  cargo fmt
  cargo check --quiet
  cargo clippy --quiet
  cargo test --quiet --tests

# Update dependencies
update:
  cargo update -Z unstable-options --breaking

# Build
build *args:
  @cargo build {{args}}

# Build for windows
build-windows:
  @cargo build --target x86_64-pc-windows-gnu --release

# Build with timing output
build-timing:
  cargo clean --profile dev
  cargo build --timings

install:
  cargo install --path .

du:
  @just build --quiet --release
  @du -h {{release_dir}}/tinyassist

# Run in debug mode
[no-cd]
run *args:
  @just build --quiet --bin {{bin}}
  @{{debug_dir}}/{{bin}} {{args}}

# Run in release mode
[no-cd]
run-release *args:
  @just build --quiet --bin {{bin}} --release
  @{{release_dir}}/{{bin}} {{args}}

# Run with strace
[no-cd]
strace *args:
  @just build --quiet --bin {{bin}}
  @strace -c -f {{debug_dir}}/{{bin}} {{args}}

# Run with time
[no-cd]
time *args:
  @just build --quiet --bin {{bin}}
  @/usr/bin/time -v {{debug_dir}}/{{bin}} {{args}}

# Run with samply profiling
[no-cd]
samply *args:
  @just build --quiet --bin {{bin}}
  @samply record -o {{perf_data}} {{debug_dir}}/{{bin}} {{args}}

set-perf-freq rate:
  sudo sysctl kernel.perf_event_max_sample_rate={{rate}}
