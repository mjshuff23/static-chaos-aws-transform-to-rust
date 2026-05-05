# Static Chaos -- C-to-Rust Incremental Migration

A legacy C MUD server (~70k lines) being incrementally migrated to safe,
idiomatic Rust. The migration preserves all observable behavior and existing
game data formats while replacing subsystems one at a time behind a build flag.

## Prerequisites

| Tool                              | Purpose                          |
| --------------------------------- | -------------------------------- |
| **gcc** + **make**                | Build the C server               |
| **Rust toolchain** (rustc, cargo) | Build the Rust library crate     |
| **libcrypt-dev**                  | Required by the C linker         |
| (Optional) AWS Transform CLI      | Transformation orchestration     |

Install Rust via [rustup](https://rustup.rs/) if not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Setup

```bash
# Clone the repository
git clone git@github.com:mjshuff23/static-chaos-aws-transform-to-rust.git
cd static-chaos-aws-transform-to-rust

# Create lowercase area.lst symlinks (required for run-dev.sh/run-env.sh startup)
ln -sf AREA.LST env/dev/area/area.lst
ln -sf AREA.LST env/prod/area/area.lst

# Ensure launcher scripts are executable
chmod +x scripts/run-dev.sh scripts/run-env.sh
```

## Build Commands

### Pure C build (no Rust required)

```bash
make -C src chaosium
```

Produces `src/chaosium`. This is the default and requires no Rust toolchain.

### Rust-integrated build

```bash
make -C src chaosium USE_RUST=1
```

Builds the Rust static library (`target/release/libchaos_rust.a`) and links it
in place of `envconfig.o`. The `CARGO` variable defaults to `cargo` but can be
overridden:

```bash
make -C src chaosium USE_RUST=1 CARGO=$HOME/.cargo/bin/cargo
```

### Run Rust tests

```bash
cargo test --manifest-path chaos-rust/Cargo.toml
```

Runs 48 tests total:

- 27 unit tests (config path resolution + xrand PRNG)
- 8 integration tests (full init/getter cycle)
- 13 parity tests (C-vs-Rust output comparison)

## Runtime -- Dev Server

```bash
scripts/run-dev.sh [port]
```

Starts the server on port 4000 (default) with environment variables pointing
to `env/dev/`. Requires:

1. The `area.lst` symlink in `env/dev/area/` (see Setup above).
2. The `src/chaosium` binary built via either build command.
3. The script sets `CHAOS_ENV_ROOT`, `CHAOS_PLAYER_DIR`, `CHAOS_AREA_DIR`,
   `CHAOS_FINGER_DIR`, `CHAOS_NOTES_DIR` and creates any missing player
   subdirectories.

Logs are written to `env/dev/log/<timestamp>.log`.

## Current Migration Status

| Module           | Status      | Notes                                                        |
| ---------------- | ----------- | ------------------------------------------------------------ |
| `src/envconfig.c`| Migrated    | Behind `USE_RUST=1`; path resolution via `chaos-rust/src/config/` |
| `src/xrand.c`   | Implemented | Rust impl + tests in `util/rand.rs`; not yet linked (symbols in db.c) |
| `src/bit.c`     | Planned     | Phase 1b optional pilot                                      |
| All other modules| Not started | Deferred to later phases                                     |

The Rust workspace lives in `chaos-rust/` with modules:

- `config/mod.rs` -- Path resolution logic (replaces envconfig.c)
- `config/ffi.rs` -- C-compatible FFI boundary with `catch_unwind` hardening
- `util/rand.rs` -- Lagged Fibonacci PRNG (`number_mm`, `number_range`, `isquare`)

## What's Next

1. **xrand symbol replacement** -- Add `#ifndef USE_RUST_RAND` guards around
   `number_mm`/`number_range`/`isquare` in `src/db.c`, then export the Rust
   versions under the original C names.
2. **bit.c pilot** -- Flag lookup/value/string utilities with static flag
   tables. Depends on `str_cmp` and `one_argument` wrappers.
3. **Area file parser** (`db.c`) -- Complex, requires fixture testing against
   actual `.are` files.
4. **Player save/load** (`save.c`) -- Must preserve exact file format
   byte-for-byte.

## Project Layout

```
.
+-- Cargo.toml                    # Workspace root
+-- Cargo.lock                    # Tracked (app/server workspace)
+-- chaos-rust/                   # Rust library crate
|   +-- Cargo.toml
|   +-- src/
|   |   +-- lib.rs
|   |   +-- config/
|   |   |   +-- mod.rs            # Path resolution implementation
|   |   |   +-- ffi.rs            # C FFI boundary (catch_unwind hardened)
|   |   +-- util/
|   |       +-- mod.rs
|   |       +-- rand.rs           # Lagged Fibonacci PRNG
|   +-- tests/
|       +-- config_integration_test.rs
|       +-- parity_test.rs        # C/Rust output comparison harness
+-- src/                          # C source
|   +-- Makefile                  # USE_RUST=1 flag for Rust integration
|   +-- envconfig.c               # Original (used when USE_RUST not set)
|   +-- xrand.c                   # Standalone PRNG test (not in build)
|   +-- bit.c                     # Flag utilities
|   +-- merc.h                    # Header declaring envconfig API
|   +-- ...
+-- env/
|   +-- dev/                      # Development runtime data
|   +-- prod/                     # Production runtime data
+-- scripts/
    +-- run-dev.sh                # Dev launcher (port 4000)
    +-- run-env.sh                # Generic env launcher
```
