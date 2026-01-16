#!/bin/bash
set -e

# Build and deploy ZSM to byteFORCE for testing

MODE="${1:-debug}"
TARGET="wasm32-wasip1"
REMOTE="dan@byteFORCE:.config/zellij/plugins/"

if [[ "$MODE" == "--debug" || "$MODE" == "debug" ]]; then
    echo "Building debug..."
    cargo build --target "$TARGET"
    WASM="target/$TARGET/debug/zsm.wasm"
else
    echo "Building release..."
    cargo build --target "$TARGET" --release
    WASM="target/$TARGET/release/zsm.wasm"
fi

echo "Deploying to $REMOTE"
scp "$WASM" "$REMOTE"
cp -f "$WASM" ~/.config/zellij/plugins/

echo "Done. Reload plugin in zellij with Alt+R or:"
echo "  zellij action start-or-reload-plugin file:~/.config/zellij/plugins/zsm.wasm"
