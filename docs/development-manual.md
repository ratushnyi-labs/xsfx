# Development Manual

## Prerequisites

- Docker (required for running tests)
- Rust stable toolchain (1.76+, only for native builds without Docker)
- `pkg-config` and `liblzma-dev` (Linux, for `native-compress` feature)
- `musl-tools` (Linux, for building the static stub)

## Repository Structure

```text
src/
  lib.rs            Re-exports: common, compress, decompress, pe_loader, macho_loader
  common.rs         Trailer struct, MAGIC, TRAILER_SIZE
  compress.rs       compress_lzma (XZ compression)
  decompress.rs     decompress_payload (XZ decompression)
  pe_loader.rs      Windows PE in-memory loading (compiled only on Windows)
  macho_loader.rs   macOS Mach-O in-memory loading (compiled only on macOS)
  bin/
    packer.rs       CLI entry point for packing
    stub.rs         SFX runtime (self-extract and execute)
tests/
  integration.rs    Integration tests (round-trip, format, security)
docs/
  spec.md           Business specification
  rules.md          Development rules
  specks/           Speck-driven development task files
```

## Building

### Build the stub (must be built first)

#### Linux (static via musl -- recommended)

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --bin stub --target x86_64-unknown-linux-musl
```

#### Windows (static via crt-static)

```bash
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release --bin stub --target x86_64-pc-windows-msvc
```

#### macOS

```bash
cargo build --release --bin stub --target aarch64-apple-darwin
```

### Build the packer (requires stub path)

```bash
export XSFX_STUB_PATH="$(pwd)/target/x86_64-unknown-linux-musl/release/stub"
cargo build --release --bin xsfx
```

### Build with native compression (better ratios)

```bash
export XSFX_STUB_PATH="$(pwd)/target/x86_64-unknown-linux-musl/release/stub"
cargo build --release --bin xsfx --features native-compress
```

## Platform-Specific Execution Strategies

The stub uses platform-specific in-memory execution on all platforms:

| Platform | Strategy | Mechanism |
|----------|----------|-----------|
| Linux | memfd_create | Anonymous in-memory file via syscall, executed via /proc/self/fd |
| Windows | In-process PE loading | Parse PE headers, VirtualAlloc, map sections, resolve imports, call entry point |
| macOS | NSObjectFileImage API | Patch MH_EXECUTE to MH_BUNDLE, load via NSCreateObjectFileImageFromMemory |

No temp files are used on any platform.

## Testing (Docker -- primary method)

All tests run inside Docker containers. No local Rust toolchain required.

### Run the full test suite (fmt + clippy + tests + coverage + audit)

```bash
docker compose run --build test
```

This single command executes, in fail-fast order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --lib --tests -- -D warnings`
3. `cargo llvm-cov` with 100% line/function coverage enforcement
4. `cargo audit`

### Build the binary via Docker (includes static musl stub)

```bash
docker build --target build -t xsfx-build .
```

The Docker build compiles the stub with `x86_64-unknown-linux-musl` for a fully
static binary, then embeds it into the packer.

## Testing (native -- without Docker)

Requires local Rust toolchain and `cargo-llvm-cov`, `cargo-audit`.

```bash
cargo test --lib --tests
cargo llvm-cov --lib --tests
cargo fmt --all -- --check
cargo clippy --lib --tests -- -D warnings
cargo audit
```

## Cross-Compilation

See `.github/workflows/ci.yml` for the full target matrix. Key targets:

| Target                        | OS      | Stub Target                     | Notes              |
|-------------------------------|---------|---------------------------------|--------------------|
| `x86_64-unknown-linux-gnu`    | Linux   | `x86_64-unknown-linux-musl`     | native-compress    |
| `aarch64-unknown-linux-gnu`   | Linux   | `aarch64-unknown-linux-musl`    | cross, native      |
| `x86_64-unknown-linux-musl`   | Linux   | `x86_64-unknown-linux-musl`     | static, pure Rust  |
| `aarch64-apple-darwin`        | macOS   | `aarch64-apple-darwin`          | native-compress    |
| `x86_64-apple-darwin`         | macOS   | `x86_64-apple-darwin`           | native-compress    |
| `x86_64-pc-windows-msvc`     | Windows | `x86_64-pc-windows-msvc`        | native-compress    |
| `aarch64-pc-windows-msvc`    | Windows | `aarch64-pc-windows-msvc`       | native-compress    |

Linux stubs always use musl for static linking. Windows stubs use `crt-static`.

Cross-compilation requires the appropriate linker and target installed:

```bash
rustup target add <target>
```
