# Installation

JsonQ is a PHP extension written in Rust. You can install it using the **PIE** installer or by compiling from source.

## Requirements

- **PHP**: 8.1 or higher
- **OS**: Linux (Ubuntu/Debian recommended) or macOS
- **Rust**: 1.75+ (only for building from source)

## Option 1: Install via PIE (Recommended)

The easiest way to install JsonQ is using the **PHP Installer for Extensions (PIE)**.

```bash
# Install the latest stable version
pie install mameyugo/jsonq
```

This will automatically download, compile, and configure the extension for your PHP version.

### Requirements for PIE
- PHP 8.1+
- Rust toolchain (cargo) must be installed as PIE builds from source for Rust extensions.

## Option 2: Compile from Source

If you want to install JsonQ on your local machine or server, follow these steps.

### 1. Install Rust and PHP Dev Tools

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

### 2. Clone the Repository

```bash
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ
```

### 3. Build the Extension

We use `cargo` to build the extension.

```bash
# Build release version
cargo build --release
```

This will create the shared library at `target/release/libjsonq.so` (Linux) or `target/release/libjsonq.dylib` (macOS).

### 4. Enable the Extension

Locate your PHP extension directory:

```bash
php -i | grep extension_dir
```

Copy the compiled library to that directory:

```bash
sudo cp target/release/libjsonq.so /usr/lib/php/20230831/  # Adjust path as needed
```

Create a configuration file for PHP:

```bash
# /etc/php/8.3/mods-available/jsonq.ini
extension=jsonq
```

Enable the extension:

```bash
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
