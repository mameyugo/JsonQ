# Installing JsonQ on macOS

## Pre-built Binaries (Recommended)

GitHub Actions automatically builds `.dylib` binaries for macOS on every release (Apple Silicon arm64 and Intel x86_64).

### Quick Install

```bash
PHP_VER=$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')
ARCH=$(uname -m)   # arm64 or x86_64
LATEST=$(curl -s https://api.github.com/repos/mameyugo/JsonQ/releases/latest \
  | grep tag_name | sed 's/.*"v\(.*\)".*/\1/')

curl -LO "https://github.com/mameyugo/JsonQ/releases/download/v${LATEST}/jsonq-v${LATEST}-macos-${ARCH}-php${PHP_VER}.dylib"
sudo cp jsonq-v*.dylib $(php-config --extension-dir)/jsonq.so
echo 'extension=jsonq.so' >> $(php --ini | grep 'Loaded Configuration' | awk '{print $NF}')
```

### Available macOS binaries per release

| File | PHP | Architecture |
|------|-----|--------------|
| `jsonq-v{ver}-macos-arm64-php8.1.dylib` | PHP 8.1 | Apple Silicon |
| `jsonq-v{ver}-macos-arm64-php8.2.dylib` | PHP 8.2 | Apple Silicon |
| `jsonq-v{ver}-macos-arm64-php8.3.dylib` | PHP 8.3 | Apple Silicon |
| `jsonq-v{ver}-macos-arm64-php8.4.dylib` | PHP 8.4 | Apple Silicon |
| `jsonq-v{ver}-macos-x86_64-php8.1.dylib` | PHP 8.1 | Intel |
| `jsonq-v{ver}-macos-x86_64-php8.2.dylib` | PHP 8.2 | Intel |
| `jsonq-v{ver}-macos-x86_64-php8.3.dylib` | PHP 8.3 | Intel |
| `jsonq-v{ver}-macos-x86_64-php8.4.dylib` | PHP 8.4 | Intel |

---

## Verify installation

```bash
php -m | grep jsonq
php -r "echo jsonq_version();"
```

---

## Build from Source

Only needed if pre-built binaries don't work for your setup:

```bash
brew install php rust
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ && make build && make install
```
