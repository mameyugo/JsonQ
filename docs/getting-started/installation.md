# Installation

JsonQ is a PHP extension written in Rust. Choose the installation method that best fits your needs.

## Requirements

- **PHP**: 8.1 or higher
- **OS**: Linux (Ubuntu/Debian recommended) or macOS
- **Rust**: 1.75+ (only for building from source)

## Option 1: Precompiled Binaries (Recommended)

The easiest way to install JsonQ is using precompiled binaries or packages.

### Debian/Ubuntu Packages (.deb)

**Method 1a: Via APT Repository**

```bash
# 1. Add GPG key
curl -fsSL https://mameyugo.github.io/JsonQ/jsonq-archive-keyring.gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/jsonq-archive-keyring.gpg

# 2. Add repository
echo "deb [signed-by=/usr/share/keyrings/jsonq-archive-keyring.gpg] https://mameyugo.github.io/JsonQ stable main" \
  | sudo tee /etc/apt/sources.list.d/jsonq.list

# 3. Install (replace 8.3 with your PHP version)
sudo apt update
sudo apt install php8.3-jsonq
```

**Method 1b: Direct .deb Download**

Download from [GitHub Releases](https://github.com/mameyugo/JsonQ/releases/latest):

```bash
# Download (replace version and PHP version as needed)
wget https://github.com/mameyugo/JsonQ/releases/download/v0.5.0/php8.3-jsonq_0.5.0-1_amd64.deb

# Install
sudo dpkg -i php8.3-jsonq_0.5.0-1_amd64.deb
sudo apt install -f    # fix dependencies if needed
```

### Binary Libraries (.so / .dylib)

Download the appropriate binary for your system from [GitHub Releases](https://github.com/mameyugo/JsonQ/releases/latest):

```bash
# Example for Linux x86_64 with PHP 8.3
wget https://github.com/mameyugo/JsonQ/releases/download/v0.5.0/jsonq-v0.5.0-linux-x86_64-php8.3.so

# Copy to PHP extension directory
sudo cp jsonq-v0.5.0-linux-x86_64-php8.3.so $(php-config --extension-dir)/jsonq.so

# Create configuration file
echo "extension=jsonq.so" | sudo tee /etc/php/8.3/mods-available/jsonq.ini

# Enable the extension
sudo phpenmod jsonq
```

Available binaries:
- Linux: `amd64`, `arm64` for PHP 8.1, 8.2, 8.3, 8.4
- macOS: `x86_64`, `arm64` for PHP 8.1, 8.2, 8.3, 8.4

## Option 2: Quick Install Script

For a quick automated installation:

```bash
# Install for default PHP version
curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash

# Or specify PHP version
curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash -s -- --php 8.3
```

This script will automatically detect your system and install the appropriate binary.

## Option 3: Install via PIE

If you already have PIE (PHP Installer for Extensions) installed:

```bash
# Install the latest stable version
pie install mameyugo/jsonq
```

**Note:** PIE requires the Rust toolchain as it builds extensions from source.

## Option 4: Build from Source

For development or advanced use cases, you can build JsonQ from source.

### Prerequisites

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y php-dev clang curl build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**macOS:**
```bash
brew install php rust
```

### Build Steps

```bash
# 1. Clone the repository
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ

# 2. Build release version
cargo build --release
```

This will create the shared library at `target/release/libjsonq.so` (Linux) or `target/release/libjsonq.dylib` (macOS).

### Installation

```bash
# 1. Copy to PHP extension directory
sudo cp target/release/libjsonq.so $(php-config --extension-dir)/jsonq.so

# 2. Create configuration file
echo "extension=jsonq.so" | sudo tee /etc/php/8.3/mods-available/jsonq.ini

# 3. Enable the extension
sudo phpenmod jsonq
```

## Verification

To verify that JsonQ is installed correctly, run:

```bash
php -m | grep jsonq
```

You should see `jsonq` in the output. You can also run this simple script:

```php
<?php
echo "JsonQ Version: " . jsonq_version() . "\n";
```
