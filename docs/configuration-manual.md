# Configuration Manual

## Build-time Configuration

xsfx has no runtime configuration. All settings are applied at build time.

### Environment Variables

| Variable         | Required | Description                                    |
|------------------|----------|------------------------------------------------|
| `XSFX_TARGETS` | No | Comma-separated list of targets to build stubs for. `all` for all targets. Defaults to host target. |
| `XSFX_PREBUILT_STUBS_DIR` | No | Directory of pre-built stubs (skips building). Used by Docker cross-build. |
| `XSFX_SKIP_STUB_BUILD` | No | Set to `1` to skip stub building and generate empty catalog. Used by test stage. |
| `XSFX_OUT_TARGET` | No | Override the default target triple when running the packer. |

### Cargo Features

| Feature            | Default | Description                                   |
|--------------------|---------|-----------------------------------------------|
| `native-compress`  | Off     | Use liblzma via xz2 for ultra compression (LZMA2 extreme preset 9 + x86 BCJ filter + 64 MiB dictionary). Produces smaller payloads. Requires `liblzma-dev` on Linux. The stub always uses pure-Rust lzma-rs regardless. |

### Release Profile

The release profile is optimized for minimal binary size:

| Setting        | Value   | Effect                                 |
|----------------|---------|----------------------------------------|
| `opt-level`    | `"z"`   | Optimize for size                      |
| `lto`          | `true`  | Link-time optimization                 |
| `codegen-units`| `1`     | Better cross-crate optimization        |
| `panic`        | `abort` | Remove unwinding code                  |
| `strip`        | `true`  | Remove debug symbols (Rust 1.76+)      |

### Stub Build Pipeline

The stub uses a multi-stage minification pipeline to stay under 100 KB:

| Stage | Technique | Effect |
|-------|-----------|--------|
| 1. Nightly build-std | `-Z build-std=std,panic_abort` | Rebuilds std optimized for size, dead code elimination |
| 2. Immediate abort | `-Cpanic=immediate-abort` | Eliminates all panic formatting machinery |
| 3. Format removal | No `eprintln!`/`format!` in stub code | Avoids pulling in `core::fmt` infrastructure |
| 4. UPX compression | `upx --best --lzma` post-build | LZMA-compressed executable with tiny decompressor header |

Result: ~170 KB pre-UPX -> ~77 KB post-UPX (Windows x64).

## Platform-specific Behavior

| Platform | Execution Strategy | Mechanism | Static Linking |
|----------|-------------------|-----------|----------------|
| Linux    | memfd_create (in-memory) | Anonymous memory fd via syscall | musl libc |
| Windows  | In-process PE loading | VirtualAlloc + import resolution + relocation fixing | crt-static |
| macOS    | NSObjectFileImage (in-memory) | Patch MH_EXECUTE to MH_BUNDLE, load via dyld API | System frameworks |

No temp files are used on any platform.

## Compression Settings

### Packer (with `native-compress`)

| Setting | Value | Description |
|---------|-------|-------------|
| Preset | 9 + EXTREME | Maximum compression effort |
| Dictionary | 64 MiB (capped to input size) | Larger dictionary = better compression for large files |
| Match finder | BinaryTree4 | Best compression ratio |
| Nice length | 273 | Maximum match length |
| Pre-filter | x86 BCJ | Improves compression of x86/x64 executables |
| Check | CRC-64 | Integrity verification |

### Packer (without `native-compress`)

Uses pure-Rust lzma-rs with default XZ settings. Produces larger output but has
no native dependency.

### Stub (decompression)

Always uses pure-Rust lzma-rs `xz_decompress`. Compatible with both standard and
ultra-compressed XZ streams.

## Cross-Build Architecture

All targets are built from a single Linux Docker container (`Dockerfile`, stage `cross`):

| Component | Toolchain |
|-----------|-----------|
| Linux gnu (x64/arm64) | gcc / aarch64-linux-gnu-gcc |
| Linux musl (x64/arm64) | musl-gcc / aarch64-linux-musl-gcc |
| macOS (x64/arm64) | osxcross (o64-clang / oa64-clang) |
| Windows gnu (x64) | x86_64-w64-mingw32-gcc |
| Windows MSVC (x64/arm64) | clang-cl + lld-link + xwin SDK |

Build pipeline: `./build.sh` builds the Docker image then runs the entrypoint
which cross-compiles all stubs (nightly + build-std + UPX) and all packers.

## CI Configuration

See `.github/workflows/ci.yml`. The CI pipeline runs:

1. **Test (Docker):** `docker compose run --build test` — fmt, clippy, tests
   with 100% line/function coverage, and `cargo audit` inside a lightweight container.
2. **Build (Docker):** Single ubuntu runner builds the cross-compilation image, then
   cross-compiles all 9 targets inside it. Stubs use nightly build-std + UPX.

Fail-fast order: test must pass before build starts.
