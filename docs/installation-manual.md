# Installation Manual

## Download Pre-built Binaries

Download the latest release for your platform from the
[GitHub Releases](../../releases) page.

Available archives:

| Platform              | Archive                          |
|-----------------------|----------------------------------|
| Linux x64 (glibc)    | `xsfx-linux-x64-gnu.tar.gz`     |
| Linux ARM64 (glibc)  | `xsfx-linux-arm64-gnu.tar.gz`   |
| Linux x64 (musl)     | `xsfx-linux-x64-musl.tar.gz`    |
| macOS ARM64           | `xsfx-macos-arm64.tar.gz`       |
| macOS x64             | `xsfx-macos-x64.tar.gz`         |
| Windows x64           | `xsfx-windows-x64.zip`          |
| Windows ARM64         | `xsfx-windows-arm64.zip`        |

### Linux / macOS

```bash
tar -xzf xsfx-<platform>.tar.gz
chmod +x xsfx-packed   # or xsfx, depending on archive contents
mv xsfx-packed /usr/local/bin/xsfx
```

### Windows

Extract `xsfx-windows-x64.zip` and place the `.exe` in a directory on your PATH.

## Build from Source

### Prerequisites

- Rust stable toolchain (1.76+)
- Linux: `pkg-config`, `liblzma-dev` (for `native-compress`)

### Steps

```bash
# 1. Clone
git clone <repo-url>
cd xsfx

# 2. Build stub
cargo build --release --bin stub

# 3. Build packer with embedded stub
export XSFX_STUB_PATH="$(pwd)/target/release/stub"
cargo build --release --bin xsfx --features native-compress

# 4. Binary is at target/release/xsfx
```

## Usage

```bash
xsfx <input_payload> <output_sfx>
```

**Example:**

```bash
xsfx my-app my-app-sfx
chmod +x my-app-sfx    # Unix only
./my-app-sfx --some-flag
```

The output SFX binary will decompress and execute the original payload, forwarding
all CLI arguments.

## Verification

Run the packed SFX to verify it works:

```bash
./my-app-sfx --version
```

The output should match running the original payload directly.

## Troubleshooting

- **"Permission denied"** on Unix: `chmod +x <sfx-binary>`
- **Payload fails to launch:** Ensure the payload matches the stub target (same
  OS and architecture).
- **Windows SFX creation:** Must use the Windows-built packer on Windows.
