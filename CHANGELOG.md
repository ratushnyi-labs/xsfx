# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- stdin/stdout pipe support (`-` for input/output)
- Parallel CI/CD pipeline with matrix builds (9 targets in parallel)
- Self-compression step in CI (xsfx packs itself)
- Open-source governance files (CONTRIBUTING, SECURITY, CHANGELOG, issue/PR templates)

### Changed
- CI migrated from sequential Docker cross-build to parallel native runners
- Security tests renamed to `test_sec_ucXXX_*` format
- Refactored `parse_pe()` into smaller functions for maintainability
- Documentation regenerated from codebase

## [0.1.7] - 2026-03-25

### Added
- Vendored static liblzma for always-on ultra compression
- Security, adversarial, and memory leak tests across all modules
- `.dev-data/` and `.dev-temp/` directory conventions

### Changed
- Pinned all dependency versions
- Refactored oversized functions for rules compliance

## [0.1.0] - 2025-01-01

### Added
- Initial release
- LZMA/XZ compression with pure-Rust decompression in stub
- 9 target platforms (Linux gnu/musl, macOS, Windows)
- In-memory execution: memfd_create (Linux), PE loader (Windows), NSObjectFileImage (macOS)
- Multi-stub catalog embedding at build time
- Cross-compilation Docker build system
