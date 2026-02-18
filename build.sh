#!/usr/bin/env bash
set -euo pipefail

# build.sh — Build ALL xsfx variants using a single Linux Docker container.
#
# From any host (macOS/Linux/Windows+Docker), cross-compiles:
#   - Linux gnu+musl (x86_64 + aarch64)
#   - macOS (x86_64 + aarch64) via osxcross
#   - Windows (x86_64 gnu + msvc, aarch64 msvc) via mingw/xwin
#
# Usage:
#   ./build.sh                     # build all targets
#   PACKER_TARGETS="x86_64-unknown-linux-gnu" ./build.sh  # subset

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJECT_DIR="$SCRIPT_DIR"
DIST_DIR="$PROJECT_DIR/dist"
mkdir -p "$DIST_DIR"

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required. Please install Docker and retry." >&2
  exit 1
fi

DOCKER_PLATFORM="linux/amd64"
IMAGE_TAG="${XSFX_IMAGE_TAG:-xsfx-build}"

# Build the cross-compilation image (cached after first run)
if ! docker image inspect "$IMAGE_TAG" >/dev/null 2>&1; then
  echo "Building Docker image '$IMAGE_TAG' (this takes a while the first time)..."
  BUILD_ARG_SDK=""
  if [ -n "${XSFX_MAC_SDK_URL:-}" ]; then
    BUILD_ARG_SDK="--build-arg MAC_SDK_URL=${XSFX_MAC_SDK_URL}"
  fi
  docker build --platform "$DOCKER_PLATFORM" --target cross -t "$IMAGE_TAG" $BUILD_ARG_SDK .
fi

# Reuse cargo registry across runs
CARGO_VOLUME="xsfx_cargo_cache"
docker volume inspect "$CARGO_VOLUME" >/dev/null 2>&1 || docker volume create "$CARGO_VOLUME" >/dev/null

echo "Building xsfx inside Docker container..."

DOCKER_CMD="docker run --rm"
DOCKER_CMD+=" --platform $DOCKER_PLATFORM"
DOCKER_CMD+=" -v \"$PROJECT_DIR\":/project"
DOCKER_CMD+=" -v $CARGO_VOLUME:/usr/local/cargo/registry"
DOCKER_CMD+=" -w /project"
DOCKER_CMD+=" -e RUSTUP_HOME=/usr/local/rustup"
DOCKER_CMD+=" -e CARGO_HOME=/usr/local/cargo"
DOCKER_CMD+=" -e PROJECT_DIR=/project"
if [ -n "${ALL_STUBS:-}" ]; then
  DOCKER_CMD+=" -e ALL_STUBS=\"$ALL_STUBS\""
fi
if [ -n "${PACKER_TARGETS:-}" ]; then
  DOCKER_CMD+=" -e PACKER_TARGETS=\"$PACKER_TARGETS\""
fi
DOCKER_CMD+=" $IMAGE_TAG"

# Disable MSYS path conversion (Windows Git Bash)
MSYS_NO_PATHCONV=1 eval "$DOCKER_CMD"

echo "Build finished. Outputs in ./dist/"
