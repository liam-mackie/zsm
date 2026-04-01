.PHONY: dev build release clean test watch reload

TARGET := wasm32-wasip1
WASM_DEBUG := target/$(TARGET)/debug/zsm.wasm
WASM_RELEASE := target/$(TARGET)/release/zsm.wasm
PLUGIN_URL := file:$(WASM_DEBUG)

# Open debug layout with auto-reload
dev: build
	zellij --layout debug.kdl

# Watch for changes and auto-rebuild + reload (run this in a pane)
watch:
	watchexec -e rs -r -- make build reload

# Build debug WASM
build:
	cargo build --target $(TARGET)

# Build release WASM
release:
	cargo build --target $(TARGET) --release

# Run tests (native target, library only)
test:
	cargo test --lib

# Clean build artifacts
clean:
	cargo clean

# Reload plugin in running Zellij session (for manual reload)
reload:
	zellij action start-or-reload-plugin $(PLUGIN_URL)
