#!/usr/bin/env bash
# Secours si Jenkins ne peut pas builder : meme build sur la VM de dev, memes tags.
# Usage : gaspezia/build-local.sh [--no-push]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

REGISTRY=nexus.gaspezia.lan/docker-private-mc
SHA=$(git rev-parse HEAD)
IMG=$REGISTRY/pumpkin

cargo build --release -p pumpkin --features plugin-v0_2

ctx=$(mktemp -d); trap 'rm -rf "$ctx"' EXIT
cp target/release/pumpkin gaspezia/Dockerfile "$ctx"/
docker build -t "$IMG:$SHA" -t "$IMG:gaspezia" "$ctx"

if [ "${1:-}" != "--no-push" ]; then
  docker push "$IMG:$SHA"
  docker push "$IMG:gaspezia"
fi
echo "Image: $IMG:$SHA"
