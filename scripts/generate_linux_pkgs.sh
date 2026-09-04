#!/usr/bin/env bash
set -euo pipefail

# ──────────────────────────────────────────────────────────────────────────────
    # Usage: generate_linux_pkgs.sh <version> <type>
#   version — Release tag (e.g. ttl-v0.6.9)
#   type    — aur | flathub
#
# Generates PKGBUILDs for AUR packages:
#   ttl-bin/PKGBUILD          — Binary package (pre-built .deb)
#   ttl-stable-src/PKGBUILD   — Source package (builds from source)
#   ttl-git/PKGBUILD          — Git package (tracks master)
# ──────────────────────────────────────────────────────────────────────────────

if [ $# -lt 2 ]; then
    echo "Usage: $0 <version> <type>"
    echo "  type: aur | flathub"
    echo "  e.g., $0 ttl-v0.6.9 aur"
    exit 1
fi

VERSION="$1"
TYPE="$2"

case "$TYPE" in
    aur)
        PKGV="${VERSION#ttl-v}"
        REPO_URL="https://github.com/TwintailTeam/TwintailLauncher"
        PKGDESC="Your anime games, one launcher"
        DEPENDS="('cairo' 'desktop-file-utils' 'gdk-pixbuf2' 'glib2' 'gtk3' 'hicolor-icon-theme' 'pango' 'webkit2gtk-4.1' 'libappindicator-gtk3' 'libayatana-appindicator' 'mangohud')"
        OPTDEPENDS="('gamemode: Feral Interactive gamemode utility' 'gamescope: ValveSoftware gamescope session utility')"
        MAKEDEPENDS="('git' 'openssl' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'cargo' 'pnpm' 'nodejs')"
        ARCH="('x86_64' 'aarch64')"
        LICENSE="('GPL-3.0-only')"
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        TEMPLATE_DIR="$SCRIPT_DIR/templates"

        echo "VERSION is: $VERSION"
        echo "PKGVER is: $PKGV"

        # Download debs and calculate checksums
        TMPDIR=$(mktemp -d)
        trap "rm -rf $TMPDIR" EXIT

        x86_64_url="$REPO_URL/releases/download/$VERSION/twintaillauncher_${PKGV}_amd64.deb"
        aarch64_url="$REPO_URL/releases/download/$VERSION/twintaillauncher_${PKGV}_arm64.deb"

        echo "Downloading and calculating checksums..."
        curl -sL "$x86_64_url" -o "$TMPDIR/twintaillauncher-${PKGV}.deb"
        curl -sL "$aarch64_url" -o "$TMPDIR/twintaillauncher-${PKGV}-arm64.deb"
        x86_64_sum=$(sha256sum "$TMPDIR/twintaillauncher-${PKGV}.deb" | cut -d' ' -f1)
        aarch64_sum=$(sha256sum "$TMPDIR/twintaillauncher-${PKGV}-arm64.deb" | cut -d' ' -f1)

        # Generate ttl-bin/PKGBUILD
        mkdir -p ttl-bin/
        sed -e "s/@PKGVER@/$PKGV/g" \
            -e "s|@REPO_URL@|$REPO_URL|g" \
            -e "s/@PKGDESC@/$PKGDESC/g" \
            -e "s/@DEPENDS@/$DEPENDS/g" \
            -e "s/@OPTDEPENDS@/$OPTDEPENDS/g" \
            -e "s/@ARCH@/$ARCH/g" \
            -e "s/@LICENSE@/$LICENSE/g" \
            -e "s/@X86_64_SUM@/$x86_64_sum/g" \
            -e "s/@AARCH64_SUM@/$aarch64_sum/g" \
            "$TEMPLATE_DIR/PKGBUILD_bin.in" > ttl-bin/PKGBUILD

        # Generate ttl-stable-src/PKGBUILD
        mkdir -p ttl-stable-src/
        sed -e "s/@PKGVER@/$PKGV/g" \
            -e "s|@REPO_URL@|$REPO_URL|g" \
            -e "s/@PKGDESC@/$PKGDESC/g" \
            -e "s/@DEPENDS@/$DEPENDS/g" \
            -e "s/@OPTDEPENDS@/$OPTDEPENDS/g" \
            -e "s/@MAKEDEPENDS@/$MAKEDEPENDS/g" \
            -e "s/@ARCH@/$ARCH/g" \
            -e "s/@LICENSE@/$LICENSE/g" \
            "$TEMPLATE_DIR/PKGBUILD_stable_src.in" > ttl-stable-src/PKGBUILD

        # Generate ttl-git/PKGBUILD
        # HEAD is the release tag (on stable), but PKGBUILD_git.in clones master, so read master
        git fetch --no-tags origin +refs/heads/master:refs/remotes/origin/master
        GIT_COMMIT_COUNT=$(git rev-list --count origin/master)
        GIT_COMMIT_HASH=$(git rev-parse --short=7 origin/master)
        GIT_PKGVER="r${GIT_COMMIT_COUNT}.${GIT_COMMIT_HASH}"
        echo "GIT_PKGVER is: $GIT_PKGVER"

        mkdir -p ttl-git/
        sed -e "s/@GIT_PKGVER@/$GIT_PKGVER/g" \
            -e "s|@REPO_URL@|$REPO_URL|g" \
            -e "s/@PKGDESC@/$PKGDESC/g" \
            -e "s/@DEPENDS@/$DEPENDS/g" \
            -e "s/@OPTDEPENDS@/$OPTDEPENDS/g" \
            -e "s/@MAKEDEPENDS@/$MAKEDEPENDS/g" \
            -e "s/@ARCH@/$ARCH/g" \
            -e "s/@LICENSE@/$LICENSE/g" \
            "$TEMPLATE_DIR/PKGBUILD_git.in" > ttl-git/PKGBUILD

        echo "Generated PKGBUILDs:"
        echo "  ttl-bin/PKGBUILD"
        echo "  ttl-stable-src/PKGBUILD"
        echo "  ttl-git/PKGBUILD"
        ;;
    flathub)
        echo "Error: flathub manifest generation not implemented"
        exit 1
        ;;
esac
