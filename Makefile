EXTENSION_DIR := $(shell php-config --extension-dir 2>/dev/null || echo "/usr/lib/php/extensions")
PHP_VERSION := $(shell php -r "echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;" 2>/dev/null || echo "8.3")
SO_FILE := target/release/libjsonq.so
DYLIB_FILE := target/release/libjsonq.dylib
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
  EXT_FILE := $(DYLIB_FILE)
else
  EXT_FILE := $(SO_FILE)
endif

.PHONY: build debug test bench install uninstall clean help quickstart

help: ## Show this help
	@echo "JsonQ — High-performance JSON file storage for PHP"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the extension (release mode)
	cargo build --release
	@echo "\n✅ Built: $(EXT_FILE)"
	@ls -lh $(EXT_FILE)

debug: ## Build the extension (debug mode)
	cargo build
	@echo "\n✅ Built: target/debug/libjsonq.so"

test: build ## Run the test suite
	php -d "extension=$$(pwd)/$(EXT_FILE)" tests/integration/run_tests.php

bench: build ## Run benchmarks
	php -d "extension=$$(pwd)/$(EXT_FILE)" examples/benchmark.php

quickstart: build ## Run quickstart example
	php -d "extension=$$(pwd)/$(EXT_FILE)" examples/quickstart.php

install: build ## Install the extension system-wide
	@echo "Installing to $(EXTENSION_DIR)..."
	sudo cp $(EXT_FILE) $(EXTENSION_DIR)/jsonq.so
	@echo "extension=jsonq.so" | sudo tee /etc/php/$(PHP_VERSION)/cli/conf.d/20-jsonq.ini > /dev/null 2>&1 || true
	@echo "extension=jsonq.so" | sudo tee /etc/php/$(PHP_VERSION)/fpm/conf.d/20-jsonq.ini > /dev/null 2>&1 || true
	@echo "\n✅ Installed! Verify with: php -m | grep jsonq"

uninstall: ## Remove the extension
	sudo rm -f $(EXTENSION_DIR)/jsonq.so
	sudo rm -f /etc/php/$(PHP_VERSION)/cli/conf.d/20-jsonq.ini
	sudo rm -f /etc/php/$(PHP_VERSION)/fpm/conf.d/20-jsonq.ini
	@echo "✅ Uninstalled"

clean: ## Clean build artifacts
	cargo clean
	@echo "✅ Cleaned"

lint: ## Run clippy and fmt check
	cargo clippy -- -D warnings -A non_snake_case
	cargo fmt --check
