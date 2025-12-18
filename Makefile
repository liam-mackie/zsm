.PHONY: dev build release clean test watch reload

TARGET := wasm32-wasip1
WASM_DEBUG := target/$(TARGET)/debug/zsm.wasm
WASM_RELEASE := target/$(TARGET)/release/zsm.wasm
PLUGIN_URL := file:$(WASM_DEBUG)

# Open debug layout with auto-reload
dev:
	zellij --layout debug.kdl

# Watch for changes and auto-rebuild + reload (run this in a pane)
watch:
	watchexec -e rs -r -- bash -c "cargo build --target $(TARGET) && zellij action start-or-reload-plugin $(PLUGIN_URL)"

# Build debug WASM
build:
	cargo build --target $(TARGET)

# Build release WASM
release:
	cargo build --target $(TARGET) --release

# Run tests (on native target, not WASM)
test:
	cargo test --target $$(rustc -vV | sed -n 's/host: //p')

# Clean build artifacts
clean:
	cargo clean

# Reload plugin in running Zellij session (for manual reload)
reload:
	zellij action start-or-reload-plugin $(PLUGIN_URL)
