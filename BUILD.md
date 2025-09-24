# Build Instructions

## Prerequisites

1. Install Rust: https://rustup.rs/
2. Windows SDK (for Windows API functions)
3. Git

## Quick Build

```bash
# Clone repository
git clone <repository-url>
cd Eye-Of-God

# Build enhanced version
cargo build --release --bin eye

# Build stealth version  
cargo build --release --bin eye-stealth
```

## Configuration

Before building, configure your bot:

1. Get bot token from @BotFather on Telegram
2. Get your chat ID
3. Update token and chat ID in source code

## Output

Built executables will be in `target/release/`:
- `eye.exe` - Enhanced version with console
- `eye-stealth.exe` - Invisible stealth version

## Pre-built Releases

Check the `releases/` folder for pre-compiled binaries.
