# Static Chaos — C-to-Rust Incremental Migration

A legacy C MUD server (~70k lines) being incrementally migrated to safe, idiomatic Rust. The migration preserves all observable behavior and existing game data formats while replacing subsystems one at a time behind a build flag.

## Prerequisites

| Tool | Purpose |
| ------ | --------- |
| **gcc** + **make** | Build the C server |
| **Rust toolchain** (rustc, cargo) | Build the Rust library crate |
| **libcrypt-dev** | Required by the C linker |
| (Optional) AWS Transform CLI | Transformation orchestration |

Install Rust via [rustup](https://rustup.rs/) if not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Setup

```bash
# Clone the repository
git clone <repo-url> && cd static-chaos

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

Builds the Rust static library (`target/release/libchaos_rust.a`) and links it in place of `envconfig.o`. The `CARGO` variable defaults to `cargo` but can be overridden:

```bash
make -C src chaosium USE_RUST=1 CARGO=$HOME/.cargo/bin/cargo
```

### Run Rust tests

```bash
cargo test --manifest-path chaos-rust/Cargo.toml
```

Runs 20 unit tests and 8 integration tests covering path resolution, trailing-slash enforcement, environment variable cascading, and area file path construction.

## Runtime — Dev Server

```bash
scripts/run-dev.sh [port]
```

Starts the server on port 4000 (default) with environment variables pointing to `env/dev/`. Requires:

1. The `area.lst` symlink in `env/dev/area/` (see Setup above).
2. The `src/chaosium` binary built via either build command.
3. The script sets `CHAOS_ENV_ROOT`, `CHAOS_PLAYER_DIR`, `CHAOS_AREA_DIR`, `CHAOS_FINGER_DIR`, `CHAOS_NOTES_DIR` and creates any missing player subdirectories.

Logs are written to `env/dev/log/<timestamp>.log`.

## Current Migration Status

| Module | Status | Notes |
| -------- | -------- | ------- |
| `src/envconfig.c` | ✅ Migrated | Behind `USE_RUST=1`; path resolution via `chaos-rust/src/config/` |
| `src/xrand.c` | ⏳ Planned | Phase 1b optional pilot |
| `src/bit.c` | ⏳ Planned | Phase 1b optional pilot |
| All other modules | ❌ Not started | Deferred to later phases |

The Rust workspace lives in `chaos-rust/` with modules:

- `config/mod.rs` — Path resolution logic (replaces envconfig.c)
- `config/ffi.rs` — C-compatible FFI boundary (`#[no_mangle] extern "C"` wrappers)

## What's Next

1. **Better C/Rust parity tests** — Add a test harness that invokes both C and Rust implementations with identical environment variable combinations and compares outputs directly.
2. **FFI hardening** — Expand poison-safe Mutex handling, add `catch_unwind` around FFI entry points to prevent Rust panics from unwinding into C.
3. **Optional next modules:**
   - `xrand.c` — Lagged Fibonacci PRNG (`number_mm`, `number_range`, `isquare`)
   - `bit.c` — Flag lookup/value/string utilities with static flag tables
4. **Area file parser** (`db.c`) — Complex, requires fixture testing against actual `.are` files.
5. **Player save/load** (`save.c`) — Must preserve exact file format byte-for-byte.

## Project Layout

```bash
.
├── Cargo.toml              # Workspace root
├── Cargo.lock              # Tracked (app/server workspace)
├── chaos-rust/             # Rust library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   └── config/
│   │       ├── mod.rs      # Path resolution implementation
│   │       └── ffi.rs      # C FFI boundary
│   └── tests/
│       └── config_integration_test.rs
├── src/                    # C source
│   ├── Makefile            # USE_RUST=1 flag for Rust integration
│   ├── envconfig.c         # Original (used when USE_RUST not set)
│   ├── merc.h              # Header declaring envconfig API
│   └── ...
├── env/
│   ├── dev/                # Development runtime data
│   └── prod/               # Production runtime data
└── scripts/
    ├── run-dev.sh          # Dev launcher (port 4000)
    └── run-env.sh          # Generic env launcher
```
