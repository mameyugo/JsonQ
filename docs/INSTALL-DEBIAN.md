# 📦 Installing JsonQ on Debian / Ubuntu

> ✅ **Pre-built binaries available**
> GitHub Actions automatically builds and publishes `.deb` packages on every
> release. No compilation required.
> Download from: https://github.com/mameyugo/JsonQ/releases/latest

---

## Quick Install (one-liner)

```bash
curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash
```

For a specific PHP version:
```bash
curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash -s -- --php 8.3
```

---

## Method 1: APT Repository (recommended)

Add the JsonQ repository for automatic updates:

```bash
# 1. Add GPG key
curl -fsSL https://mameyugo.github.io/JsonQ/jsonq-archive-keyring.gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/jsonq-archive-keyring.gpg

# 2. Add repository
echo "deb [signed-by=/usr/share/keyrings/jsonq-archive-keyring.gpg] https://mameyugo.github.io/JsonQ stable main" \
  | sudo tee /etc/apt/sources.list.d/jsonq.list

# 3. Install (replace 8.3 with your PHP version)
sudo apt-get update
sudo apt-get install php8.3-jsonq
```

### Available packages

| Package | PHP Version | Architecture |
|---------|-------------|-------------|
| `php8.1-jsonq` | PHP 8.1 | amd64, arm64 |
| `php8.2-jsonq` | PHP 8.2 | amd64, arm64 |
| `php8.3-jsonq` | PHP 8.3 | amd64, arm64 |
| `php8.4-jsonq` | PHP 8.4 | amd64, arm64 |

### Updating

```bash
sudo apt-get update
sudo apt-get upgrade php8.3-jsonq
```

---

## Method 2: Direct .deb Download

Pre-built `.deb` packages are available on every release.
Download from [GitHub Releases](https://github.com/mameyugo/JsonQ/releases/latest):

```bash
# One-liner: auto-detect PHP version and download latest .deb
PHP_VER=$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')
ARCH=$(dpkg --print-architecture)
LATEST=$(curl -s https://api.github.com/repos/mameyugo/JsonQ/releases/latest \
  | grep tag_name | sed 's/.*"v\(.*\)".*/\1/')

wget "https://github.com/mameyugo/JsonQ/releases/download/v${LATEST}/php${PHP_VER}-jsonq_${LATEST}-1_${ARCH}.deb"
sudo dpkg -i php${PHP_VER}-jsonq_${LATEST}-1_${ARCH}.deb
sudo apt-get install -f   # fix dependencies if needed
```

### Available packages per release

| File | PHP | Architecture |
|------|-----|--------------|
| php8.1-jsonq_{ver}-1_amd64.deb | PHP 8.1 | amd64 |
| php8.2-jsonq_{ver}-1_amd64.deb | PHP 8.2 | amd64 |
| php8.3-jsonq_{ver}-1_amd64.deb | PHP 8.3 | amd64 |
| php8.4-jsonq_{ver}-1_amd64.deb | PHP 8.4 | amd64 |
| php8.1-jsonq_{ver}-1_arm64.deb | PHP 8.1 | arm64 |
| php8.2-jsonq_{ver}-1_arm64.deb | PHP 8.2 | arm64 |
| php8.3-jsonq_{ver}-1_arm64.deb | PHP 8.3 | arm64 |
| php8.4-jsonq_{ver}-1_arm64.deb | PHP 8.4 | arm64 |

---

## Method 3: Build from Source

Requires Rust toolchain and PHP development headers:

```bash
# Dependencies
sudo apt-get install clang libclang-dev php8.3-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ
make build

# Install
make install
```

---

## Verify Installation

```bash
# Check module is loaded
php -m | grep jsonq

# Check version
php -r "echo jsonq_version();" # Should output 0.2.0

# Quick test
php -r "
    \$s = new JsonQ\Store('/tmp/test.json');
    \$s->set('hello', 'world');
    echo \$s->get('hello') . PHP_EOL;
"
```

## Uninstall

```bash
# Via APT
sudo apt-get remove php8.3-jsonq

# Remove repository (optional)
sudo rm /etc/apt/sources.list.d/jsonq.list
sudo rm /usr/share/keyrings/jsonq-archive-keyring.gpg
```

## Troubleshooting

**Extension not loading:**
```bash
# Check if .ini file exists
ls /etc/php/8.3/mods-available/jsonq.ini

# Enable manually
sudo phpenmod -v 8.3 jsonq

# Restart FPM
sudo systemctl restart php8.3-fpm
```

**Wrong PHP version:**
```bash
# Check your PHP version
php -v

# Install correct package
sudo apt-get install php$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')-jsonq
```
