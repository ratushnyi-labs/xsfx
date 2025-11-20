Rust SFX Packer
================

Self-extracting executable builder written in Rust. It embeds a small per-platform stub that unpacks and runs a compressed payload at runtime. By default builds are pure Rust; enabling the optional `native-compress` feature uses `liblzma` via `xz2` for smaller payloads during packing.

Features
- Pure-Rust stub (lzma-rs) for macOS, Linux, and Windows.
- Optional native compression for better ratios when building the packer.
- Single-command packing: `xsfx <payload> <output_sfx>`; stub is embedded at build time.

Prerequisites
- Rust toolchain (stable).
- Docker (for cross-building Linux/Windows stubs and packers from macOS).
- Payload binary built for the same target as the stub (e.g., Linux payload for Linux stub).

Packer CLI
```bash
xsfx <payload_path> <output_sfx>
```
The stub is chosen at build time via `include_bytes!`, so use the packer built for the target you want to produce.

Compression
- Default: pure-Rust lzma-rs encoder (slower/weaker compression, smallest vector of dependencies).
- Better ratios: build packer with `--features native-compress` to use liblzma via `xz2` during packing; the stub remains pure Rust.

Troubleshooting
- “Permission denied” when running SFX on Unix: `chmod +x dist/your-sfx`.
- Payload fails to launch: ensure the payload matches the stub/packer target (architecture + OS).
- Windows SFX creation: must run the Windows packer on Windows.
