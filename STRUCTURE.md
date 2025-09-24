# Project Structure

## 📁 Directory Layout

```
Eye-Of-God/
├── 📄 README.md              # Main project documentation
├── 📄 BUILD.md               # Build instructions  
├── 📄 LICENSE                # MIT License
├── 📄 Cargo.toml             # Rust project configuration
├── 📄 Cargo.lock             # Dependency lock file
├── 📄 .gitignore             # Git ignore rules
│
├── 📁 src/                   # Source code
│   ├── main.rs               # Enhanced version entry point
│   ├── main_stealth.rs       # Stealth version entry point
│   ├── backup_manager.rs     # Survival system
│   ├── auth.rs               # Authentication system
│   ├── token_security.rs     # Token obfuscation
│   ├── device_manager.rs     # Device information
│   ├── telegram_client.rs    # Telegram API client
│   ├── config.rs             # Configuration
│   ├── system_commands.rs    # System operations
│   ├── file_commands.rs      # File operations
│   ├── monitoring_commands.rs # System monitoring
│   ├── security_commands.rs  # Security features
│   ├── execution_commands.rs # Process management
│   ├── popup_commands.rs     # UI interactions
│   └── advanced_commands*.rs # Advanced features
│
├── 📁 releases/              # Final executables
│   ├── eye-ultimate-enhanced.exe  # Full-featured version
│   └── eye-ultimate-stealth.exe   # Invisible version
│
├── 📁 docs/                  # Documentation
│   ├── BUILD_GUIDE.md        # Detailed build guide
│   ├── COMMANDS.md           # Available commands
│   ├── PROJECT_REPORT.md     # Development report
│   └── ...                   # Other documentation
│
├── 📁 scripts/               # Build and utility scripts
│   ├── build-all.ps1         # Complete build script
│   ├── start-stealth.*       # Stealth launch scripts
│   └── ...                   # Other scripts
│
└── 📁 tools/                 # Development tools
    ├── *.exe                 # Utility executables
    ├── Cargo_*.toml          # Alternative configurations
    └── Makefile              # Build automation
```

## 🔧 Key Components

### Core Source Files
- **main.rs**: Enhanced version with console output
- **main_stealth.rs**: Invisible version without console  
- **backup_manager.rs**: Auto-persistence and survival mechanisms
- **telegram_client.rs**: Telegram Bot API integration
- **device_manager.rs**: System information gathering

### Build Outputs
- **Enhanced**: Full-featured version with debugging output
- **Stealth**: Completely invisible background operation

### Documentation
- Comprehensive build guides and command references
- Development reports and configuration details

## 🚀 Ready for GitHub

This structure is optimized for:
- ✅ Clean repository organization
- ✅ Easy build process
- ✅ Clear documentation
- ✅ Professional presentation
- ✅ MIT License compliance
