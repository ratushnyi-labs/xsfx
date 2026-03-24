# xsfx — User Manual

> **xsfx** — Self-extracting executable packer

## Prerequisites

- A supported platform (Linux, macOS, or Windows)
- No runtime dependencies required

## Installation

### Option 1: Pre-built Binary

Download from [GitHub Releases](https://github.com/ratushnyi-labs/xsfx/releases):

| Platform | File |
|----------|------|
| Linux x64 (static) | `xsfx-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (static) | `xsfx-aarch64-unknown-linux-musl.tar.gz` |
| macOS x64 | `xsfx-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (Apple Silicon) | `xsfx-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `xsfx-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `xsfx-aarch64-pc-windows-msvc.zip` |

```bash
# Linux / macOS
curl -sSfL https://github.com/ratushnyi-labs/xsfx/releases/latest/download/xsfx-x86_64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin
```

### Option 2: Build from Source

```bash
git clone https://github.com/ratushnyi-labs/xsfx.git
cd xsfx
cargo build --release --bin xsfx
```

## Usage

### Pack a binary

```bash
xsfx <input> <output> [--target <triple>]
```

- `input` — payload binary to pack (use `-` for stdin)
- `output` — output path for the SFX (use `-` for stdout)
- `--target` — target platform (defaults to host)

### Examples

```bash
# Basic packing
xsfx myapp myapp-sfx
chmod +x myapp-sfx
./myapp-sfx

# Pack for a different platform
xsfx myapp myapp-sfx.exe --target x86_64-pc-windows-msvc

# List available targets
xsfx
```

### Pipe support

Use `-` for stdin or stdout:

```bash
# Read payload from stdin
cat myapp | xsfx - myapp-sfx

# Write SFX to stdout
xsfx myapp - > myapp-sfx

# Full pipe
cat myapp | xsfx - - > myapp-sfx

# Pack and deploy over SSH
xsfx myapp - --target x86_64-unknown-linux-musl | ssh server 'cat > myapp && chmod +x myapp'
```

### Run the packed SFX

The output binary runs like any normal executable. All CLI arguments are forwarded to the payload:

```bash
./myapp-sfx --verbose --config /etc/myapp.conf
```

## Supported Targets

| Target | Arch | In-Memory Execution |
|--------|------|---------------------|
| `x86_64-unknown-linux-musl` | x64 | `memfd_create` + `execveat` |
| `aarch64-unknown-linux-musl` | ARM64 | `memfd_create` + `execveat` |
| `x86_64-apple-darwin` | x64 | `NSCreateObjectFileImageFromMemory` |
| `aarch64-apple-darwin` | ARM64 | `NSCreateObjectFileImageFromMemory` |
| `x86_64-pc-windows-msvc` | x64 | In-process PE loader |
| `aarch64-pc-windows-msvc` | ARM64 | In-process PE loader |

## Binary Format

The SFX binary consists of three parts:

```
+------------------------+
| Stub                   |  platform-specific loader (<100 KB)
+------------------------+
| Compressed payload     |  LZMA2/XZ stream
+------------------------+
| Trailer (16 bytes)     |  payload_len (u64 LE) + magic (u64 LE)
+------------------------+
```

No temporary files are written during extraction. The payload is decompressed and executed entirely in memory.

## Verification

After packing, verify the SFX works correctly:

```bash
# Pack
xsfx myapp myapp-sfx

# Run original
./myapp --version

# Run SFX — should produce identical output
./myapp-sfx --version
```

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| `"Invalid SFX magic marker"` | Corrupted SFX binary | Re-pack from original payload |
| `"File too small to contain trailer"` | Truncated SFX file | Re-download or re-pack |
| `Permission denied` | Missing execute permission | `chmod +x <sfx>` |
| `memfd_create: Operation not permitted` | Kernel restricts memfd in container | Ensure `SYS_PTRACE` cap or kernel >= 3.17 |
| Windows: `"Failed to load DLL"` | Missing runtime DLL | Install Visual C++ redistributable |
| macOS: `"Failed to create object file image"` | Code signing issue | Sign the SFX or allow unsigned execution |
