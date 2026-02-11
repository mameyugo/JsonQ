#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# JsonQ — Build Debian package for a specific PHP version
#
# Usage: ./scripts/build-deb.sh [PHP_VERSION] [ARCH]
#   PHP_VERSION: 8.1, 8.2, 8.3, 8.4 (default: auto-detect)
#   ARCH:        amd64, arm64 (default: auto-detect)
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

JSONQ_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
PHP_VERSION="${1:-$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')}"
PHP_API=$(php-config --phpapi 2>/dev/null || echo "20230831")
ARCH="${2:-$(dpkg --print-architecture 2>/dev/null || echo amd64)}"
PKG_NAME="php${PHP_VERSION}-jsonq"
PKG_DIR="build/${PKG_NAME}_${JSONQ_VERSION}-1_${ARCH}"
EXT_DIR="/usr/lib/php/${PHP_API}"
INI_DIR="/etc/php/${PHP_VERSION}/mods-available"

echo "══════════════════════════════════════════════════"
echo "  Building ${PKG_NAME} v${JSONQ_VERSION}"
echo "  PHP ${PHP_VERSION} | API ${PHP_API} | ${ARCH}"
echo "══════════════════════════════════════════════════"

# ── 1. Build the Rust extension ──
echo "🔨 Compiling extension..."
cargo build --release

SO_FILE="target/release/libjsonq.so"
if [ ! -f "$SO_FILE" ]; then
    echo "❌ Build failed: $SO_FILE not found"
    exit 1
fi

# ── 2. Create package directory structure ──
echo "📦 Creating package structure..."
rm -rf "$PKG_DIR"
mkdir -p "${PKG_DIR}${EXT_DIR}"
mkdir -p "${PKG_DIR}${INI_DIR}"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/share/doc/${PKG_NAME}"

# ── 3. Copy extension ──
cp "$SO_FILE" "${PKG_DIR}${EXT_DIR}/jsonq.so"
strip --strip-unneeded "${PKG_DIR}${EXT_DIR}/jsonq.so" 2>/dev/null || true
chmod 644 "${PKG_DIR}${EXT_DIR}/jsonq.so"

# ── 4. Create PHP ini file ──
cat > "${PKG_DIR}${INI_DIR}/jsonq.ini" << 'INI'
; JsonQ - High-performance JSON file storage engine
; https://github.com/mameyugo/JsonQ
extension=jsonq.so
INI
chmod 644 "${PKG_DIR}${INI_DIR}/jsonq.ini"

# ── 5. Installed size (in KB) ──
INSTALLED_SIZE=$(du -sk "${PKG_DIR}" | cut -f1)

# ── 6. DEBIAN/control ──
cat > "${PKG_DIR}/DEBIAN/control" << EOF
Package: ${PKG_NAME}
Version: ${JSONQ_VERSION}-1
Section: php
Priority: optional
Architecture: ${ARCH}
Depends: php${PHP_VERSION}-common
Maintainer: Mameyugo <info@mameyugo.com>
Homepage: https://github.com/mameyugo/JsonQ
Description: High-performance JSON file storage engine for PHP
 JsonQ is a PHP extension written in Rust that provides a high-performance
 JSON file storage engine with MongoDB-style queries, fluent query builder,
 schema validation, indexing, transactions, and aggregation functions.
 .
 Features:
  - 2-6x faster than pure PHP JSON handling
  - MongoDB-style query operators (\$gt, \$in, \$regex, etc.)
  - Fluent query builder with sorting, pagination, projection
  - Schema validation with nested object support
  - Single and compound indexing
  - ACID transactions with commit/rollback
  - Atomic writes with crash safety
  - Memory-mapped file reads
Installed-Size: ${INSTALLED_SIZE}
EOF

# ── 7. DEBIAN/postinst — enable module after install ──
cat > "${PKG_DIR}/DEBIAN/postinst" << 'SCRIPT'
#!/bin/sh
set -e

PHP_VERSION_SHORT=$(echo "$DPKG_MAINTSCRIPT_PACKAGE" | sed 's/php\([0-9]*\.[0-9]*\)-.*/\1/')

# Enable for all SAPIs (cli, fpm, apache2, etc.)
if command -v phpenmod > /dev/null 2>&1; then
    phpenmod -v "$PHP_VERSION_SHORT" jsonq || true
else
    # Manual symlink fallback
    for SAPI_DIR in /etc/php/${PHP_VERSION_SHORT}/*/conf.d; do
        if [ -d "$SAPI_DIR" ]; then
            ln -sf "../mods-available/jsonq.ini" "${SAPI_DIR}/20-jsonq.ini" 2>/dev/null || true
        fi
    done
fi

# Restart FPM if running
if systemctl is-active --quiet "php${PHP_VERSION_SHORT}-fpm" 2>/dev/null; then
    systemctl restart "php${PHP_VERSION_SHORT}-fpm" || true
fi

echo "✅ JsonQ extension enabled for PHP ${PHP_VERSION_SHORT}"
SCRIPT
chmod 755 "${PKG_DIR}/DEBIAN/postinst"

# ── 8. DEBIAN/prerm — disable module before removal ──
cat > "${PKG_DIR}/DEBIAN/prerm" << 'SCRIPT'
#!/bin/sh
set -e

PHP_VERSION_SHORT=$(echo "$DPKG_MAINTSCRIPT_PACKAGE" | sed 's/php\([0-9]*\.[0-9]*\)-.*/\1/')

if command -v phpdismod > /dev/null 2>&1; then
    phpdismod -v "$PHP_VERSION_SHORT" jsonq || true
else
    for SAPI_DIR in /etc/php/${PHP_VERSION_SHORT}/*/conf.d; do
        rm -f "${SAPI_DIR}/20-jsonq.ini" 2>/dev/null || true
    done
fi

if systemctl is-active --quiet "php${PHP_VERSION_SHORT}-fpm" 2>/dev/null; then
    systemctl restart "php${PHP_VERSION_SHORT}-fpm" || true
fi
SCRIPT
chmod 755 "${PKG_DIR}/DEBIAN/prerm"

# ── 9. Documentation ──
cat > "${PKG_DIR}/usr/share/doc/${PKG_NAME}/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: JsonQ
Upstream-Contact: Mameyugo <info@mameyugo.com>
Source: https://github.com/mameyugo/JsonQ

Files: *
Copyright: $(date +%Y) Mameyugo
License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
EOF

# Changelog
cat > "${PKG_DIR}/usr/share/doc/${PKG_NAME}/changelog.Debian" << EOF
${PKG_NAME} (${JSONQ_VERSION}-1) stable; urgency=low

  * Initial release

 -- Mameyugo <info@mameyugo.com>  $(date -R)
EOF
gzip -9n "${PKG_DIR}/usr/share/doc/${PKG_NAME}/changelog.Debian"

# ── 10. Build .deb ──
echo "📦 Building .deb package..."
mkdir -p build/out
dpkg-deb --build --root-owner-group "$PKG_DIR" "build/out/${PKG_NAME}_${JSONQ_VERSION}-1_${ARCH}.deb"

echo ""
echo "══════════════════════════════════════════════════"
echo "✅ Package built successfully!"
ls -lh "build/out/${PKG_NAME}_${JSONQ_VERSION}-1_${ARCH}.deb"
echo ""
echo "Install with:"
echo "  sudo dpkg -i build/out/${PKG_NAME}_${JSONQ_VERSION}-1_${ARCH}.deb"
echo "  sudo apt-get install -f  # resolve dependencies if needed"
echo "══════════════════════════════════════════════════"
