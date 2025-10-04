# Whiskerlog

Terminal history analyzer with intelligent insights and TUI interface.

## Features

- **Command Analysis**: Frequency, patterns, and usage statistics
- **Risk Detection**: Identifies dangerous commands and security issues
- **Learning Tracker**: Detects experimentation and learning sessions
- **Host Management**: Multi-environment command tracking
- **Smart Aliases**: AI-powered alias suggestions
- **Network Analysis**: Endpoint tracking and security insights
- **Package Tracking**: Installation and dependency monitoring
- **Interactive TUI**: Real-time terminal interface

## Installation

### Quick Install
```bash
# Clone and install globally
git clone https://github.com/Zer0C0d3r/Whiskerlog.git
cd Whiskerlog
sudo ./install.sh install
```

### Manual Build
```bash
# Build from source
cargo build --release
sudo cp target/release/whiskerlog /usr/local/bin/
```

### Requirements
- Rust 1.70+
- Linux/macOS (x86_64/aarch64)
- Terminal with Unicode support

## Usage

```bash
# Start TUI
whiskerlog

# Navigation
Tab/Shift+Tab    Switch tabs
↑↓/hjkl          Navigate
Enter            Select
q/Q              Quit
1-9              Jump to tab
```

## Configuration

Config file: `~/.config/whiskerlog/config.toml`

```toml
database_path = "~/.local/share/whiskerlog/history.db"
history_paths = [
    "~/.bash_history",
    "~/.zsh_history",
    "~/.local/share/fish/fish_history"
]
redaction_enabled = true
auto_import = true
danger_threshold = 0.7
experiment_detection = true
```

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
cargo clippy
cargo fmt
```

### Project Structure
```
src/
├── analysis/          # Analytics engines
├── db/               # Database layer
├── history/          # History parsing
├── ui/               # TUI components
├── config/           # Configuration
├── app.rs            # Main application
└── main.rs           # Entry point
```

### Adding Features
1. Create module in appropriate directory
2. Add to `mod.rs` exports
3. Integrate with `App` struct
4. Add UI component if needed
5. Write tests

## Troubleshooting

### Common Issues

**Database errors**
```bash
# Reset database
rm -rf ~/.local/share/whiskerlog/
rm -rf ~/.config/whiskerlog/
```

**Terminal corruption after exit**
```bash
# Reset terminal
reset
# or
stty sane
```

**Permission denied**
```bash
# Install with sudo
sudo ./install.sh install

# Or install to user directory
mkdir -p ~/.local/bin
cp target/release/whiskerlog ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

**Build failures**
```bash
# Update Rust
rustup update

# Clean build
cargo clean
cargo build --release
```

**No command history**
```bash
# Check history files exist
ls -la ~/.bash_history ~/.zsh_history

# Check config paths
whiskerlog --config
```

### Performance Issues

**Large history files**
- Enable `redaction_enabled = true`
- Increase `danger_threshold` to reduce analysis
- Limit `history_paths` to active shells only

**Memory usage**
- Restart application periodically
- Clear old database entries
- Reduce analysis frequency

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug whiskerlog

# Check database
sqlite3 ~/.local/share/whiskerlog/history.db ".tables"
```

## Uninstall

```bash
sudo ./install.sh uninstall
```

## License

MIT License - see LICENSE file for details.

## Help Needed

**Platform Support**
- Windows (PowerShell/CMD), BSD systems, ARM32, RISC-V

**Infrastructure**
- Docker containers, CI/CD improvements, package managers (Homebrew, AUR)

**Optimizations**
- Performance improvements, database optimization, memory management

**Features**
- Shell plugins, export formats, web interface, API endpoints

**Contribute**: Check [Issues](https://github.com/Zer0C0d3r/Whiskerlog/issues) → Fork → PR with tests

## Platform Support

**Supported**: Linux (x86_64, aarch64), macOS (Intel/Apple Silicon), Bash/Zsh/Fish
**Planned**: Windows, BSD, ARM32, additional shells

## Contributing

1. Fork repository
2. Create feature branch
3. Make changes with tests
4. Submit pull request

Keep commits focused and add tests for new features.