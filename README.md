# Zoxide Session Manager (ZSM)

A powerful Zellij plugin that seamlessly integrates **zoxide** (smart directory navigation) with **Zellij sessions**, making it incredibly easy to jump between your most-used directories and create/manage development sessions.

## 🚀 What Does It Do?

ZSM bridges the gap between `zoxide` and Zellij's session management:

- **🎯 Smart Directory Listing**: Shows your most-used directories from zoxide, ranked by frequency
- **⚡ Instant Session Creation**: Create new Zellij sessions in any directory with one keystroke
- **🔍 Fuzzy Search**: Search through directories and existing sessions simultaneously
- **🧠 Intelligent Naming**: Auto-generates meaningful session names with conflict resolution
- **⚙️ Layout Support**: Choose from available layouts or use your default layout

## 📋 Requirements

- **[zoxide](https://github.com/ajeetdsouza/zoxide)** - Install and use it for a while to build up your directory database
- **Zellij** with plugin support

## 📦 Installation

### Option 1: Download Release (Recommended)
1. Download the latest `zsm.wasm` from [Releases](https://github.com/liam-mackie/zsm/releases)
2. Copy it to your Zellij plugins directory (~/.config/zellij/plugins/):
3. Add to your Zellij configuration (see [Configuration](#%EF%B8%8F-configuration))

```bash
mkdir -p ~/.config/zellij/plugins
curl -sSL -o ~/.config/zellij/plugins https://github.com/liam-mackie/zsm/releases/download/v0.1.0/zsm.wasm
```

### Option 2: Build from Source

```bash
# Add WASM target if not already added
rustup target add wasm32-wasip1

# Build the plugin
cargo build --target wasm32-wasip1 --release

# The plugin will be at: target/wasm32-wasip1/release/zsm.wasm
```

## ⚙️ Configuration

Add ZSM to your configuration file (e.g., `~/.config/zellij/config.kdl`):

### Basic Configuration - bind to a key

```kdl
keybinds clear-defaults=true {
...
    shared_except "locked" {
        bind "<your-key>" { 
            // Note: you must use the absolute path to the plugin file
            LaunchOrFocusPlugin "file:/your/home/dir/.config/zellij/plugins/zsm.wasm" {
                floating true
                move_to_focused_tab true
            }
        }
    }
...
}
```

### Advanced Configuration - with options
```kdl
keybinds clear-defaults=true {
...
    shared_except "locked" {
        bind "<your-key>" { 
            // Note: you must use the absolulte path to the plugin file
            LaunchOrFocusPlugin "file:/your/home/dir/.config/zellij/plugins/zsm.wasm" {
                floating true
                move_to_focused_tab true
                
                // Default layout for Ctrl+Enter quick creation
                default_layout "development"
            
                // Session name separator (default: ".")
                session_separator "_"
                
                // Show sessions that can be resurrected
                show_resurrectable_sessions true
                
                // Base paths to strip from directory names (pipe-separated)
                // Example: "/home/user/projects/foo" becomes just "foo"
                base_paths "/home/user/projects|/Users/dev/work"
            }
        }
    }
...
```

### Configuration Options

| Option                        | Description                               | Default | Example                                 |
|-------------------------------|-------------------------------------------|---------|-----------------------------------------|
| `default_layout`              | Layout name for Ctrl+Enter quick creation | None    | `"development"`                         |
| `session_separator`           | Character used in session names           | `"."`   | `"-"` or `"_"`                          |
| `show_resurrectable_sessions` | Show sessions that can be resurrected     | `false` | `true`                                  |
| `base_paths`                  | Base paths to strip from directory names (pipe-separated) | None    | `"/home/user\|/home/user/git/projects"` |

## 🎯 How It Works

### 1. Directory Display

ZSM shows your zoxide directories ranked by usage frequency:

```
~/projects/my-app        (most used)
~/work/client-project
~/personal/website
~/dotfiles              (least used)
```

### 2. Smart Session Naming

ZSM automatically generates meaningful session names:

- **Simple**: `~/projects/webapp` → `webapp`
- **Nested**: `~/projects/client/backend` → `client.backend`
- **Conflicts**: Multiple "app" directories → `client.app`, `personal.app`
- **Long names**: Intelligent abbreviation → `very-long-project-name` → `v-l-p-name`
- **Base Paths**: Configure base paths to strip from names (e.g., `/home/user` as base path)
  - `/home/user/projects/foo` → `projects.foo`
  - `/home/user` → `/home/user` (exact matches keep full path)

### 3. Session Integration

- **Existing sessions** are shown with indicators: `● current` or `○ available`
- **Resurrectable sessions** (if enabled) are shown with a `↺` icon
- **Auto-increment**: If session `webapp` exists, creates `webapp.2`, `webapp.3`, etc.

### 4. Quick Workflows

**Jump to existing session**

1. Open ZSM
2. Type to search for session
3. Press `Enter` → Instantly switch

**Create new session**

1. Open ZSM  
2. Navigate to directory
3. Press `Enter` → Opens session creation (or `Ctrl+Enter` for default)
4. Choose layout 
5. Session is created in that directory

## 🔐 Permissions

ZSM requires these Zellij permissions:

- **RunCommands**: Execute zoxide queries
- **ReadApplicationState**: Read existing sessions and layouts
- **ChangeApplicationState**: Create and switch sessions  
- **MessageAndLaunchOtherPlugins**: Launch filepicker

## 🐛 Troubleshooting

### No directories showing?

- Ensure zoxide is installed: `which zoxide`
- Build up your directory database by navigating around: `cd ~/projects && cd ~/work`
- Check zoxide database: `zoxide query -l`

### Default layout not working?

- Verify layout name matches exactly (case-sensitive)
- Check available layouts in Zellij
- Layout must exist in current session

### Filepicker issues?

- Ensure MessageAndLaunchOtherPlugins permission is granted

## 🚧 Development

### Prerequisites

```bash
# Add WASM target
rustup target add wasm32-wasip1
```

### Makefile Commands

| Command | Description |
|---------|-------------|
| `make build` | Build debug WASM plugin |
| `make release` | Build release WASM plugin |
| `make test` | Run tests (native target) |
| `make dev` | Start Zellij with debug layout |
| `make watch` | Watch for changes and auto-rebuild + reload |
| `make reload` | Manually reload plugin in running Zellij |
| `make clean` | Clean build artifacts |

### Local Development

#### Option 1: Using the dev layout (recommended)

```bash
# Start Zellij with the debug layout
make dev

# In a separate pane, watch for changes and auto-reload
make watch
```

#### Option 2: Manual workflow

```bash
# Build the plugin
make build

# Start Zellij with plugin layout
zellij -l zellij.kdl

# Reload after changes
make reload
```

### Running Tests

```bash
# Run all tests
make test

# Run specific test module
cargo test --lib search::engine::tests

# Run with output
cargo test --lib -- --nocapture
```

## 🤝 Contributing

Contributions welcome, though my time is limited so please be patient with reviews!

---

**Made with ❤️ for the Zellij community**

