#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"
: "${OPENIEM_SOAK_SECONDS:=60}"
python3 scripts/validate-deb-package.py "$1"
python3 - <<'PY'
from pathlib import Path
import subprocess
for p in Path('packaging/deb/DEBIAN').glob('*'):
    if p.name in {'postinst','prerm','postrm'}:
        subprocess.run(['bash','-n',str(p)],check=True)
print('package maintainer scripts: PASS')
PY
if (( OPENIEM_SOAK_SECONDS < 1 )); then
  echo 'software stability gate: invalid duration' >&2
  exit 2
fi
printf 'PACKAGE_RELEASE_GATE: PARTIAL (package validation and maintainer-script syntax)\n'
printf 'SOFTWARE stability soak: BLOCKED (requested %s seconds; executable media/audio soak harness not wired yet)\n' "$OPENIEM_SOAK_SECONDS"
exit 2
