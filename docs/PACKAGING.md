# Debian packaging

Build release binaries, then:

```bash
ARCH=amd64 packaging/deb/build-deb.sh
ARCH=arm64 API_BIN=... ADMIN_BIN=... packaging/deb/build-deb.sh
python3 scripts/validate-deb-package.py dist/open-iem_$(cat VERSION)_amd64.deb
```

Artifacts: `open-iem_<version>_amd64.deb` and `open-iem_<version>_arm64.deb` The builder emits the required `open-iem_...` names.

Package owns binaries, systemd unit and example configuration. `/etc/openiem/openiem-server.env` is a conffile. Upgrade preserves `/etc/openiem` and `/var/lib/openiem`; uninstall removes package payload; explicit purge removes state. No credentials are embedded. Metadata is apt-compatible for future repository publication.
