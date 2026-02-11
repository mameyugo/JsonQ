#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# JsonQ — Generate APT repository from .deb packages
#
# Creates a Packages/Release structure suitable for hosting
# on GitHub Pages, S3, or any static file server.
#
# Usage: ./scripts/build-repo.sh [GPG_KEY_ID]
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

REPO_DIR="repo"
DIST="stable"
COMPONENT="main"
GPG_KEY="${1:-}"

echo "📦 Building APT repository..."

rm -rf "${REPO_DIR}"
mkdir -p "${REPO_DIR}/dists/${DIST}/${COMPONENT}/binary-amd64"
mkdir -p "${REPO_DIR}/dists/${DIST}/${COMPONENT}/binary-arm64"
mkdir -p "${REPO_DIR}/pool/${COMPONENT}"

# ── Copy all .deb files to pool ──
if [ -d "build/out" ]; then
    cp build/out/*.deb "${REPO_DIR}/pool/${COMPONENT}/" 2>/dev/null || true
fi

# ── Generate Packages index for each arch ──
for ARCH in amd64 arm64; do
    PACKAGES_DIR="${REPO_DIR}/dists/${DIST}/${COMPONENT}/binary-${ARCH}"
    
    cd "${REPO_DIR}"
    dpkg-scanpackages --arch "${ARCH}" "pool/${COMPONENT}" > "dists/${DIST}/${COMPONENT}/binary-${ARCH}/Packages" 2>/dev/null || \
        touch "dists/${DIST}/${COMPONENT}/binary-${ARCH}/Packages"
    gzip -9c "dists/${DIST}/${COMPONENT}/binary-${ARCH}/Packages" > "dists/${DIST}/${COMPONENT}/binary-${ARCH}/Packages.gz"
    cd ..
done

# ── Generate Release file ──
cd "${REPO_DIR}"
cat > "dists/${DIST}/Release" << EOF
Origin: JsonQ
Label: JsonQ PHP Extension
Suite: ${DIST}
Codename: ${DIST}
Architectures: amd64 arm64
Components: ${COMPONENT}
Description: High-performance JSON file storage engine for PHP
Date: $(date -Ru)
EOF

# Add checksums
{
    echo "MD5Sum:"
    find "dists/${DIST}/${COMPONENT}" -type f | while read -r file; do
        SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file")
        MD5=$(md5sum "$file" | cut -d' ' -f1)
        REL_PATH=$(echo "$file" | sed "s|dists/${DIST}/||")
        printf " %s %s %s\n" "$MD5" "$SIZE" "$REL_PATH"
    done
    
    echo "SHA256:"
    find "dists/${DIST}/${COMPONENT}" -type f | while read -r file; do
        SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file")
        SHA=$(sha256sum "$file" | cut -d' ' -f1)
        REL_PATH=$(echo "$file" | sed "s|dists/${DIST}/||")
        printf " %s %s %s\n" "$SHA" "$SIZE" "$REL_PATH"
    done
} >> "dists/${DIST}/Release"

# ── GPG sign if key provided ──
if [ -n "$GPG_KEY" ]; then
    echo "🔐 Signing repository with GPG key ${GPG_KEY}..."
    gpg --default-key "${GPG_KEY}" --armor --detach-sign --output "dists/${DIST}/Release.gpg" "dists/${DIST}/Release"
    gpg --default-key "${GPG_KEY}" --armor --clearsign --output "dists/${DIST}/InRelease" "dists/${DIST}/Release"
    
    # Export public key for users
    gpg --armor --export "${GPG_KEY}" > "jsonq-archive-keyring.gpg"
    echo "✅ GPG signed. Public key: repo/jsonq-archive-keyring.gpg"
fi

cd ..

echo ""
echo "══════════════════════════════════════════════════"
echo "✅ APT repository generated in ${REPO_DIR}/"
find "${REPO_DIR}" -type f | head -20
echo "══════════════════════════════════════════════════"
