#!/usr/bin/env bash
set -euo pipefail
PACKAGE=${1:?package path required}
PLATFORM=${PACKAGE_PLATFORM:-linux/amd64}
if ! command -v docker >/dev/null 2>&1; then
  echo "PACKAGE_RELEASE_GATE lifecycle: PENDING (docker unavailable)" >&2
  exit 2
fi
[[ -f "$PACKAGE" ]] || { echo "missing package: $PACKAGE" >&2; exit 2; }
PACKAGE_DIR=$(cd "$(dirname "$PACKAGE")" && pwd)
PACKAGE_FILE=$(basename "$PACKAGE")
docker run --rm --platform "$PLATFORM" -e PACKAGE_FILE="$PACKAGE_FILE" -v "$PACKAGE_DIR:/pkg:ro" debian:bookworm bash -eu -o pipefail -c '
  apt-get update >/dev/null
  mkdir -p /tmp/old/DEBIAN /tmp/old/usr/lib/openiem
  dpkg-deb --extract /pkg/"$PACKAGE_FILE" /tmp/old
  dpkg-deb --control /pkg/"$PACKAGE_FILE" /tmp/old/DEBIAN
  sed -i 's/^Version:.*/Version: 0.0.1/' /tmp/old/DEBIAN/control
  dpkg-deb --build --root-owner-group /tmp/old /tmp/openiem-old.deb >/dev/null
  apt-get install -y --no-install-recommends /tmp/openiem-old.deb >/dev/null
  test -x /usr/lib/openiem/api-server
  test -f /etc/openiem/openiem-server.env
  printf "state-before-upgrade\n" >/var/lib/openiem/lifecycle-marker
  apt-get install -y --no-install-recommends /pkg/"$PACKAGE_FILE" >/dev/null
  test -f /var/lib/openiem/lifecycle-marker
  apt-get remove -y openiem >/dev/null
  test -f /var/lib/openiem/lifecycle-marker
  apt-get install -y --no-install-recommends /pkg/"$PACKAGE_FILE" >/dev/null
  apt-get purge -y openiem >/dev/null
  test ! -e /var/lib/openiem
  test ! -e /etc/openiem
'
echo 'PACKAGE_RELEASE_GATE lifecycle: PASS'
