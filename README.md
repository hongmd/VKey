# VKey - Vietnamese Input Method for macOS

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://www.apple.com/macos/)

A modern Vietnamese input method application built with Rust, featuring real-time text conversion and a clean, intuitive interface. VKey provides seamless Vietnamese text input using popular input methods like Telex and VNI.

## Screenshot

![VKey Application Interface](screenshot.png)

*VKey application showing the main configuration interface with input method settings, keyboard shortcuts, and advanced options.*

## Features

### Core Functionality
- ✨ **Real-time Vietnamese text conversion** using Telex, VNI, and VIQR input methods
- ⌨️ **System-wide keyboard integration** with native macOS support
- 🔄 **Smart backspace handling** that properly manages Vietnamese diacritics
- 🎯 **Multiple input method support** with easy switching between methods

### User Interface
- 🎨 **Modern, clean UI** built with GPUI framework
- ⚙️ **Comprehensive settings panel** for customizing input behavior
- 🔧 **Advanced configuration options** for power users
- 💡 **Intuitive controls** with keyboard shortcuts and mouse support

### Input Methods
- **Telex**: Type `aa` → `â`, `ee` → `ê`, `oo` → `ô`, etc.
- **VNI**: Type `a6` → `â`, `e6` → `ê`, `o6` → `ô`, etc.
- **VIQR**: Type `a^` → `â`, `e^` → `ê`, `o^` → `ô`, etc.

### Advanced Features
- 🔤 **Multiple encoding support** (Unicode, TCVN3, VNI-Win)
- 🧠 **Smart input mode switching** between Vietnamese and English
- ✅ **Spell checking** and auto-correction capabilities
- 📱 **App-specific encoding memory** for consistent behavior across applications

## Requirements

- **Operating System**: macOS 10.15+ (Catalina or later)
- **Rust**: 1.70 or later
- **Cargo**: Latest stable version
- **Accessibility Permissions**: Required for system-wide keyboard integration

## Installation

### From Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/vkey.git
   cd vkey
   ```

2. **Build the application**:
   ```bash
   cargo build --release
   ```

3. **Run VKey**:
   ```bash
   cargo run --release
   ```

### Prerequisites

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Usage

### First-time Setup

1. **Launch VKey** and grant accessibility permissions when prompted
2. **Choose your preferred input method** (Telex, VNI, or VIQR)
3. **Configure settings** according to your preferences
4. **Toggle Vietnamese input** using the interface or keyboard shortcuts

### Input Method Examples

#### Telex Input Method
```
Type: "Tieeng Vieej naaam"
Output: "Tiếng Việt Nam"

Type: "chaof ban!"
Output: "chào bạn!"
```

#### VNI Input Method
```
Type: "Tie65ng Vie65t Nam"
Output: "Tiếng Việt Nam"

Type: "cha2o ba5n!"
Output: "chào bạn!"
```

### Tone Mark Reference

| Mark | Telex | VNI | VIQR | Example |
|------|-------|-----|------|---------|
| Acute (sắc) | s | 1 | ' | á |
| Grave (huyền) | f | 2 | ` | à |
| Hook (hỏi) | r | 3 | ? | ả |
| Tilde (ngã) | x | 4 | ~ | ã |
| Dot (nặng) | j | 5 | . | ạ |
| Circumflex (mũ) | aa/ee/oo | 6 | ^^ | â/ê/ô |
| Breve (móc) | aw/ow/uw | 7/8 | (+ | ă/ơ/ư |
| D-stroke | dd | 9 | dd | đ |

### Keyboard Shortcuts

- **Toggle Vietnamese/English**: Configure in settings
- **Clear buffer**: Backspace
- **Commit text**: Space or Enter

## Configuration

VKey stores its configuration in JSON format. You can customize:

- **Input Method**: Choose between Telex, VNI, or VIQR
- **Character Encoding**: Unicode, TCVN3, or VNI-Win
- **Keyboard Modifiers**: Configure which modifier keys are enabled
- **Advanced Settings**: Spell checking, auto-correction, and more

### Configuration File Location

The configuration is automatically saved and can be found at:
```
~/.config/vkey/config.json
```

## Development

### Project Structure

```
src/
├── core/               # Core Vietnamese input processing
│   ├── config.rs      # Configuration management
│   ├── types.rs       # Type definitions
│   └── vietnamese_input.rs  # Input method logic
├── platform/          # Platform-specific integrations
│   └── macos.rs       # macOS keyboard handling
├── ui/                # User interface components
│   └── components/    # UI components
└── error/             # Error handling
```

### Building from Source

1. **Clone and enter the project directory**:
   ```bash
   git clone https://github.com/your-username/vkey.git
   cd vkey
   ```

2. **Install dependencies**:
   ```bash
   cargo check
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Build for development**:
   ```bash
   cargo build
   ```

5. **Build for release**:
   ```bash
   cargo build --release
   ```

### Dependencies

VKey uses the following major dependencies:

- **[gpui](https://github.com/zed-industries/zed)**: Modern GPU-accelerated UI framework
- **[vi](https://crates.io/crates/vi)**: Vietnamese input method library
- **[serde](https://serde.rs/)**: Serialization framework for configuration
- **[thiserror](https://crates.io/crates/thiserror)**: Error handling utilities

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Getting Started

1. **Fork the repository** on GitHub
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** and add tests if applicable
4. **Run the test suite**: `cargo test`
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to the branch**: `git push origin feature/amazing-feature`
7. **Open a Pull Request**

### Code Style

- Follow standard Rust formatting with `cargo fmt`
- Run `cargo clippy` to catch common mistakes
- Add tests for new functionality
- Update documentation for any public API changes

### Reporting Issues

Please use the [GitHub Issues](https://github.com/your-username/vkey/issues) page to report bugs or request features. Include:

- **Operating system version**
- **VKey version**
- **Steps to reproduce the issue**
- **Expected vs actual behavior**

## Roadmap

### Planned Features

- [ ] **Windows and Linux support**
- [ ] **Additional input methods** (Simple Telex, etc.)
- [ ] **Customizable keyboard shortcuts**
- [ ] **Theme and appearance customization**
- [ ] **Plugin system for extending functionality**
- [ ] **Cloud sync for settings**

### Known Limitations

- Currently only supports macOS
- Requires accessibility permissions for system-wide input
- Some applications may not fully support all features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **[vi-rs](https://github.com/zerox-dg/vi-rs)** for the Vietnamese input processing library
- **[Zed](https://github.com/zed-industries/zed)** for the GPUI framework
- The Vietnamese input method community for input method specifications
- All contributors who help improve VKey

## Support

- **Documentation**: Check this README and inline code documentation
- **Issues**: [GitHub Issues](https://github.com/your-username/vkey/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/vkey/discussions)

---

Made with ❤️ for the Vietnamese developer community 
