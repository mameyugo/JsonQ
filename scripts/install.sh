#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# JsonQ — Quick Installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash
#   curl -fsSL https://raw.githubusercontent.com/mameyugo/JsonQ/main/scripts/install.sh | sudo bash -s -- --php 8.3
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

REPO_URL="https://mameyugo.github.io/JsonQ"
GITHUB_RELEASES="https://github.com/mameyugo/JsonQ/releases"
PHP_VERSION=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --php) PHP_VERSION="$2"; shift 2 ;;
        *) echo -e "${RED}Unknown option: $1${NC}"; exit 1 ;;
    esac
done

echo -e "${CYAN}"
echo "╔══════════════════════════════════════╗"
echo "║     JsonQ Extension Installer        ║"
echo "╚══════════════════════════════════════╝"
echo -e "${NC}"

# ── Detect PHP version ──
if [ -z "$PHP_VERSION" ]; then
    if command -v php > /dev/null 2>&1; then
        PHP_VERSION=$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')
        echo -e "${GREEN}✓${NC} Detected PHP ${PHP_VERSION}"
    else
        echo -e "${RED}✗ PHP not found. Install PHP first or specify version with --php${NC}"
        exit 1
    fi
fi

# ── Detect architecture ──
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
echo -e "${GREEN}✓${NC} Architecture: ${ARCH}"

# ── Check if already installed ──
if php -m 2>/dev/null | grep -q jsonq; then
    CURRENT=$(php -r "echo jsonq_version();" 2>/dev/null || echo "unknown")
    echo -e "${YELLOW}⚠ JsonQ v${CURRENT} is already installed${NC}"
    read -p "  Reinstall/upgrade? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
fi

# ── FIX: Función centralizada para habilitar y recargar la extensión ──
# Se llama siempre después de instalar el .deb, tanto por APT como por descarga directa.
enable_and_reload() {
    local php_ver="$1"

    echo -e "\n${CYAN}→ Enabling extension for all SAPIs...${NC}"

    # phpenmod habilita para cli, fpm, apache2 y cualquier otro SAPI registrado
    if command -v phpenmod > /dev/null 2>&1; then
        phpenmod -v "$php_ver" jsonq && \
            echo -e "${GREEN}✓${NC} phpenmod: extension enabled for all SAPIs (cli, fpm, apache2…)"
    else
        # Fallback manual: crear symlinks en cada conf.d que exista
        for sapi_dir in /etc/php/${php_ver}/*/conf.d; do
            if [ -d "$sapi_dir" ]; then
                sapi=$(basename "$(dirname "$sapi_dir")")
                ln -sf "/etc/php/${php_ver}/mods-available/jsonq.ini" \
                       "${sapi_dir}/20-jsonq.ini" 2>/dev/null || true
                echo -e "${GREEN}✓${NC} Linked for SAPI: ${sapi}"
            fi
        done
    fi

    # ── FIX: Reiniciar php-fpm si está activo ──
    if systemctl is-active --quiet "php${php_ver}-fpm" 2>/dev/null; then
        systemctl restart "php${php_ver}-fpm" && \
            echo -e "${GREEN}✓${NC} php${php_ver}-fpm restarted"
    fi

    # ── FIX: Reiniciar Apache si está activo ──
    # El postinst del .deb no lo hace; es necesario para mod_php y php-fpm vía Apache
    if systemctl is-active --quiet apache2 2>/dev/null; then
        systemctl restart apache2 && \
            echo -e "${GREEN}✓${NC} apache2 restarted"
    elif systemctl is-active --quiet httpd 2>/dev/null; then
        systemctl restart httpd && \
            echo -e "${GREEN}✓${NC} httpd restarted"
    fi

    # ── FIX: Reiniciar nginx si está activo ──
    # nginx no ejecuta PHP directamente pero puede depender de fpm; un reload basta
    if systemctl is-active --quiet nginx 2>/dev/null; then
        systemctl reload nginx && \
            echo -e "${GREEN}✓${NC} nginx reloaded"
    fi
}

# ── FIX: Función para verificar que la extensión quedó activa ──
verify_installation() {
    local php_ver="$1"
    local ok=true

    echo -e "\n${CYAN}→ Verifying installation...${NC}"

    # CLI
    if php -m 2>/dev/null | grep -q jsonq; then
        VERSION=$(php -r "echo jsonq_version();" 2>/dev/null || echo "?")
        echo -e "${GREEN}✓${NC} CLI:    JsonQ v${VERSION} loaded"
    else
        echo -e "${YELLOW}⚠${NC} CLI:    extension NOT detected in php -m"
        echo "         Check: php --ini | grep mods-available"
        ok=false
    fi

    # FPM (si el socket/servicio está disponible)
    if command -v php-fpm${php_ver} > /dev/null 2>&1 || \
       systemctl is-active --quiet "php${php_ver}-fpm" 2>/dev/null; then
        if php-fpm${php_ver} -m 2>/dev/null | grep -q jsonq; then
            echo -e "${GREEN}✓${NC} FPM:    extension loaded"
        else
            echo -e "${YELLOW}⚠${NC} FPM:    extension NOT detected"
            ok=false
        fi
    fi

    # Resumen
    if [ "$ok" = true ]; then
        echo -e "\n${GREEN}✅ All checks passed!${NC}"
    else
        echo -e "\n${YELLOW}⚠ Some SAPIs may not have the extension active.${NC}"
        echo "  Run: php -m | grep jsonq"
        echo "  Or check: ls /etc/php/${php_ver}/*/conf.d/ | grep jsonq"
    fi
}

# ── Method 1: Try APT repository ──
echo -e "\n${CYAN}→ Setting up APT repository...${NC}"

SIGNED_REPO=false
if curl -fsSL -I "${REPO_URL}/jsonq-archive-keyring.gpg" >/dev/null 2>&1; then
    SIGNED_REPO=true
fi

if [ "$SIGNED_REPO" = true ]; then
    if curl -fsSL "${REPO_URL}/jsonq-archive-keyring.gpg" -o /tmp/jsonq-keyring.gpg 2>/dev/null; then
        gpg --dearmor -o /usr/share/keyrings/jsonq-archive-keyring.gpg < /tmp/jsonq-keyring.gpg 2>/dev/null || \
            cp /tmp/jsonq-keyring.gpg /usr/share/keyrings/jsonq-archive-keyring.gpg

        echo "deb [signed-by=/usr/share/keyrings/jsonq-archive-keyring.gpg] ${REPO_URL} stable main" \
            > /etc/apt/sources.list.d/jsonq.list
    fi
else
    echo -e "${YELLOW}⚠ GPG key not found, using untrusted repository...${NC}"
    echo "deb [trusted=yes] ${REPO_URL} stable main" \
        > /etc/apt/sources.list.d/jsonq.list
fi

apt-get update -qq 2>/dev/null || echo -e "${YELLOW}⚠ apt-get update had warnings${NC}"

PKG="php${PHP_VERSION}-jsonq"
if apt-cache show "$PKG" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Package found in APT repository"
    apt-get install -y "$PKG"
    # FIX: el postinst del .deb ya llama a phpenmod, pero Apache no se reinicia ahí
    enable_and_reload "$PHP_VERSION"
    verify_installation "$PHP_VERSION"
    echo -e "\n${CYAN}Verify:${NC} php -r \"echo jsonq_version();\""
    exit 0
fi

echo -e "${YELLOW}⚠ Package not found in APT repo, trying direct download...${NC}"

# ── Method 2: Direct .deb download from GitHub Releases ──
echo -e "\n${CYAN}→ Downloading from GitHub Releases...${NC}"

LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/mameyugo/JsonQ/releases/latest" 2>/dev/null \
    | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/' || echo "")

if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/mameyugo/JsonQ/releases?per_page=1" 2>/dev/null \
        | grep '"tag_name"' | head -1 | sed 's/.*"v\(.*\)".*/\1/' || echo "")
fi

if [ -z "$LATEST_TAG" ]; then
    echo -e "${RED}✗ Could not determine latest version (No releases found?)${NC}"
    echo "  Visit ${GITHUB_RELEASES} for manual download"
    exit 1
fi

DEB_FILE="php${PHP_VERSION}-jsonq_${LATEST_TAG}-1_${ARCH}.deb"
DEB_URL="${GITHUB_RELEASES}/download/v${LATEST_TAG}/${DEB_FILE}"

echo "  Downloading ${DEB_FILE}..."
if curl -fsSL "$DEB_URL" -o "/tmp/${DEB_FILE}"; then
    if dpkg -i "/tmp/${DEB_FILE}"; then
        rm -f "/tmp/${DEB_FILE}"
    else
        echo -e "${YELLOW}⚠ dpkg failed, trying to fix dependencies...${NC}"
        apt-get install -f -y
        dpkg -i "/tmp/${DEB_FILE}"
        rm -f "/tmp/${DEB_FILE}"
    fi
    # FIX: habilitar y recargar todos los SAPIs (incluido Apache)
    enable_and_reload "$PHP_VERSION"
    verify_installation "$PHP_VERSION"
    echo -e "\n${CYAN}Verify:${NC} php -r \"echo jsonq_version();\""
else
    echo -e "${RED}✗ Download failed (404). File might not exist for this PHP version/Architecture.${NC}"
    echo "  Tried: ${DEB_URL}"
    echo "  Available packages at: ${GITHUB_RELEASES}/latest"
    exit 1
fi