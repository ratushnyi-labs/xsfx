# SDD-007 — Two-Stage SFX for musl Stub Minification

**Impacted UCs:** UC-002
**Impacted BR/WF:** BR-006, BR-013, BR-015, WF-002

## Problem

The musl stub is ~96 KB after all build-time optimizations (nightly
build-std, panic=immediate-abort, opt-level=z, LTO, strip). UPX cannot
be used on musl stubs (see SDD-004). Binary-level dead code analysis
(xstrip) finds no dead code — all 293 functions are reachable because
Rust's LTO already eliminates unreachable code at IR level.

## Scope

- Implement a two-stage SFX format for `*-linux-musl` targets
- Stage0: `#![no_std]` loader < 10 KB with custom RFC 1951 inflate
- Stage1: standard SFX (stub + xz payload + trailer)
- Fix stub to open `/proc/self/exe` directly for memfd compatibility
- PoC with full build/test Dockerfile in `.dev-temp/xstrip-test/`

## Non-goals

- Replacing the standard SFX format for non-musl targets
- Modifying the packer to produce two-stage SFX (PoC uses assembler tool)
- aarch64 support for stage0 (x86_64 only in PoC, uses inline asm)

## Results

| Metric | Value |
|--------|-------|
| stage0 loader | 10,136 bytes (< 10 KB) |
| Traditional SFX | 122,420 bytes |
| Two-stage SFX | 72,533 bytes |
| Size reduction | 49,887 bytes (40%) |
| xstrip reduction | 0 bytes (0% — LTO already optimal) |

## Implementation

### Stage0 loader (`.dev-temp/xstrip-test/stage0/`)

- `#![no_std]`, `#![no_main]`, zero crate dependencies
- Raw x86_64 Linux syscalls via `core::arch::asm!` (open, read, write,
  lseek, mmap, munmap, memfd_create, fchmod, execveat, close, exit)
- Custom `_start` entry point via `global_asm!`
- Custom RFC 1951 (raw deflate) decompressor: bit reader, canonical
  Huffman decode, stored/fixed/dynamic block types (~200 lines)
- Build: nightly + `build-std=core,compiler_builtins` +
  `compiler-builtins-mem` + `-Clink-self-contained=no -Clinker=cc
  -Clink-arg=-nostdlib -Clink-arg=-static -Crelocation-model=static`

### Stub fix (`src/bin/stub.rs`)

On Linux, the stub now opens `/proc/self/exe` directly instead of
resolving the symlink via `env::current_exe()` then opening the
resulting path. This is required because when running from a memfd
(two-stage SFX), `readlink("/proc/self/exe")` returns a virtual path
like `/memfd:s (deleted)` that cannot be opened as a filesystem path.
Opening `/proc/self/exe` works because the kernel follows the symlink
to the underlying file descriptor.

### Assembler (`.dev-temp/xstrip-test/assembler/`)

Test tool with two modes:
- `stage1`: stub + xz_compress(payload) + trailer(16)
- `stage0`: stage0 + deflate_best(stage1_sfx) + trailer(24)

## Acceptance Criteria

- AC-1: Stage0 binary < 10,240 bytes (10 KB)
- AC-2: Two-stage SFX produces correct output (payload + args forwarded)
- AC-3: Traditional SFX still works unchanged
- AC-4: Stub works from both disk and memfd execution
- AC-5: Two-stage SFX is at least 30% smaller than traditional

## Security Acceptance Criteria (mandatory)

- SEC-1: Stage0 validates trailer magic before decompressing
- SEC-2: Stage0 validates compressed/uncompressed lengths are non-zero
- SEC-3: Stage0 inflate checks output bounds on every write
- SEC-4: Stage0 uses memfd + execveat (no temp files on disk)
- SEC-5: No file descriptor leaks (all fds closed or MFD_CLOEXEC)

## Failure Modes / Error Mapping

| Condition | Behavior |
|-----------|----------|
| No trailer / bad magic | Stage0 exits 1 silently |
| Inflate failure | Stage0 exits 1 silently |
| Size mismatch after inflate | Stage0 exits 1 silently |
| memfd_create fails | Stage0 exits 1 silently |
| execveat fails | Stage0 exits 1 silently |
| Stage1 stub error | Prints "SFX stub error", exits 1 |

## Test Matrix (mandatory)

| AC    | Unit | Integration | Curl Dev | Base UI | UI | Curl Prod API | Prod Fullstack |
|-------|------|-------------|----------|---------|----|---------------|----------------|
| AC-1  | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |
| AC-2  | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |
| AC-3  | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |
| AC-4  | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |
| AC-5  | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |
| SEC-1 | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |
| SEC-2 | N/A  | N/A         | N/A      | N/A     | N/A| N/A           | N/A            |

Notes:
- All criteria verified via the Docker-based PoC build
  (`.dev-temp/xstrip-test/Dockerfile`) which builds all components,
  assembles both SFX variants, and runs functional tests.
- The stage0 loader and assembler are nostd/test-only code not covered
  by the project's unit/integration test suite.
