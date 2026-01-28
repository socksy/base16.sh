#!/usr/bin/env bash
set -e

SCHEMES_DIR="data/schemes"

if [ -d "$SCHEMES_DIR/.git" ]; then
    echo "Updating schemes..."
    git -C "$SCHEMES_DIR" pull --ff-only
else
    echo "Cloning schemes..."
    mkdir -p "$(dirname "$SCHEMES_DIR")"
    rm -rf "$SCHEMES_DIR"
    git clone --depth 1 "https://github.com/tinted-theming/schemes.git" "$SCHEMES_DIR"
fi

echo "Done!"
