#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
VERSION=${VERSION:-$(tr -d '[:space:]' < "$ROOT/VERSION")}
[[ "$VERSION" =~ ^[0-9][0-9A-Za-z.+:~-]*$ ]] || { printf "invalid Debian package version: %s\n" "$VERSION" >&2; exit 2; }
ARCH=${ARCH:-$(dpkg --print-architecture)}
OUT_DIR=${OUT_DIR:-$ROOT/dist}
API_BIN=${API_BIN:-$ROOT/server/target/release/api-server}
ADMIN_BIN=${ADMIN_BIN:-$ROOT/server/target/release/open-iem-admin}

case "$ARCH" in amd64|arm64) ;; *) echo "unsupported Debian architecture: $ARCH" >&2; exit 2;; esac
[[ -x "$API_BIN" && -x "$ADMIN_BIN" ]] || { echo "build release binaries first or set API_BIN/ADMIN_BIN" >&2; exit 3; }
command -v readelf >/dev/null || { echo "readelf is required to verify package binary architecture" >&2; exit 3; }
case "$ARCH" in
  amd64) EXPECTED_MACHINE='Advanced Micro Devices X86-64' ;;
  arm64) EXPECTED_MACHINE='AArch64' ;;
esac
for binary in "$API_BIN" "$ADMIN_BIN"; do
  readelf -h "$binary" | grep -Fq "Machine:                           $EXPECTED_MACHINE" || {
    echo "binary architecture mismatch for $ARCH: $binary" >&2
    exit 2
  }
done
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
PKG="$WORK/openiem"
mkdir -p "$PKG/DEBIAN" "$PKG/usr/lib/openiem" "$PKG/usr/lib/systemd/system" "$PKG/etc/openiem" "$PKG/usr/share/doc/openiem"
chmod 0750 "$PKG/etc/openiem"
install -m 0755 "$API_BIN" "$PKG/usr/lib/openiem/api-server"
install -m 0755 "$ADMIN_BIN" "$PKG/usr/lib/openiem/open-iem-admin"
install -m 0644 "$ROOT/packaging/deb/usr/lib/systemd/system/openiem-server.service" "$PKG/usr/lib/systemd/system/"
install -m 0644 "$ROOT/packaging/deb/etc/openiem/openiem-server.env.example" "$PKG/usr/share/doc/openiem/"
install -Dm0640 "$ROOT/packaging/deb/etc/openiem/openiem-server.env.example" "$PKG/etc/openiem/openiem-server.env"
chown root:openiem "$PKG/etc/openiem/openiem-server.env" 2>/dev/null || true
install -m 0644 "$ROOT/packaging/deb/usr/share/doc/openiem/README.Debian" "$PKG/usr/share/doc/openiem/"
install -m 0644 "$ROOT/packaging/deb/DEBIAN/conffiles" "$PKG/DEBIAN/"
VERSION_SED=${VERSION//\\/\\\\}; VERSION_SED=${VERSION_SED//&/\\&}; VERSION_SED=${VERSION_SED//|/\\|}
sed -e "s|@VERSION@|$VERSION_SED|g" -e "s|@ARCH@|$ARCH|g" "$ROOT/packaging/deb/DEBIAN/control.in" > "$PKG/DEBIAN/control"
for f in postinst prerm postrm; do install -m 0755 "$ROOT/packaging/deb/DEBIAN/$f" "$PKG/DEBIAN/$f"; done
install -d "$OUT_DIR"
OUTPUT="$OUT_DIR/open-iem_${VERSION}_${ARCH}.deb"
dpkg-deb --root-owner-group --build "$PKG" "$OUTPUT" >/dev/null
printf '%s\n' "$OUTPUT"
