# Contributing to JsonQ

Thanks for your interest in contributing to JsonQ! This document covers the development workflow and guidelines.

## Development Setup

### Prerequisites

- PHP 8.1+ with `php-dev` headers
- Rust 1.75+ (via [rustup](https://rustup.rs/))
- `libclang-dev` (for bindgen)

### Quick Start

```bash
git clone https://github.com/mameyugo/JsonQ.git
cd JsonQ

# Install dependencies (Ubuntu/Debian)
sudo apt-get install php8.3-dev libclang-dev

# Build debug mode (faster compilation)
cargo build

# Build release mode (optimized)
cargo build --release

# Run tests
php -d "extension=$(pwd)/target/release/libjsonq.so" tests/integration/run_tests.php
```

### Project Structure

```
JsonQ/
├── src/                # Rust core implementation
│   ├── lib.rs          # Entry point + PHP module registration
│   ├── conversion/     # PHP ↔ Rust conversion
│   ├── store/          # Core storage engine
│   ├── index/          # Indexing system
│   ├── query/          # Query engine
│   └── ...             # Other modules
├── tests/
│   ├── integration/    # PHP integration tests
│   └── unit/           # Rust unit tests
├── benches/            # Performance benchmarks
├── stubs/JsonQ.php     # PHP stubs for IDE autocompletion
├── Cargo.toml          # Rust dependencies and metadata
└── ...
```

## Development Workflow

1. **Fork and branch** — Create a feature branch from `main`
2. **Make changes** — Edit the relevant module in `src/` and add tests
3. **Build** — `cargo build --release`
4. **Test** — Run the full test suite (100+ tests must pass)
5. **Submit PR** — Open a pull request with a clear description

## Code Style

### Rust

- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Method names use camelCase in `#[php_method]` to match PHP conventions
- Internal functions use snake_case
- Keep functions focused — one responsibility each
- Use short but descriptive variable names in hot paths

### PHP Tests

- One `test()` call per behavior
- Use `assert_eq`, `assert_true`, `assert_false`, `assert_count`, `assert_null`
- Group tests by feature section
- Each test should be independent (use `fresh_store()`)

## Adding a New Feature

1. **Implement in Rust** — Add the logic in the appropriate module under `src/`
2. **Expose via PHP** — Register the method in `src/php/`
3. **Update stubs** — Add PHPDoc to `stubs/JsonQ.php`
4. **Add tests** — Cover happy path, edge cases, and error conditions
5. **Update docs** — Add to README.md and CHANGELOG.md

## Reporting Issues

- Use GitHub Issues with a clear title
- Include: PHP version, OS, Rust version, extension version
- Provide a minimal reproduction script
- Include the error output

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for new functionality
- Update documentation if the API changes
- All existing tests must pass
- Add a CHANGELOG entry under `[Unreleased]`

## Release Process

JsonQ uses GitHub Actions to automatically build binaries and create releases. There are two ways to trigger a release:

### Method 1: Tag Push (Recommended)

When releasing a new version of JsonQ, follow these steps:

1. **Update `Cargo.toml`**: Increment the `version` field.
2. **Update PHP Stubs**: Update the `@version` tag in `stubs/jsonq.php`.
3. **Update Documentation**:
    - Update the version number in `README.md`.
    - Update `CHANGELOG.md` with the new version and release date.
    - Update version strings in `docs/API.md` and `docs/INSTALL-DEBIAN.md`.
4. **Update Tests**: Update the version assertion in `tests/integration/run_tests.php`.
5. **Run Full Test Suite**: Ensure `cargo test` and `php tests/integration/run_tests.php` both pass.
6. **Commit and Tag**:
   ```bash
   git add .
   git commit -m "Release v0.X.X"
   git tag -a v0.X.X -m "Release v0.X.X"
   git push origin main --tags
   ```

The GitHub Actions workflow will automatically:
- Build binaries for Linux and macOS (PHP 8.1-8.4)
- Create .deb packages for Debian/Ubuntu
- Create a GitHub release with all artifacts attached

### Method 2: Manual Workflow Dispatch

If you need to create a release without pushing a new tag (e.g., to rebuild binaries for the current version):

1. Go to the [Actions tab](https://github.com/mameyugo/JsonQ/actions/workflows/build.yml)
2. Click "Run workflow"
3. Select the branch (usually `main`)
4. Click "Run workflow"

The release will be created using the version specified in `Cargo.toml` (e.g., version `0.3.2` -> tag `v0.3.2`).

**Note**: If a release with that version already exists, it will be updated with the new binaries. This is useful for rebuilding releases without changing the version number.

## Performance Considerations

When contributing, keep in mind:

- **Hot path optimization** — `get()`, `find()`, and `value_to_zval()` are called frequently
- **Memory** — Avoid unnecessary clones; use references where possible
- **Cache invalidation** — Writes must clear the cache and indexes
- **Atomic writes** — All mutations must go through the tmp+fsync+rename pattern

## License

By contributing, you agree that your contributions will be licensed under The PHP License, version 3.01.
