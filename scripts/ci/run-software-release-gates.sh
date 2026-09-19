#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"
: "${OPENIEM_SOAK_SECONDS:=60}"
[[ "$OPENIEM_SOAK_SECONDS" =~ ^[0-9]+$ ]] || {
  echo "software stability gate: duration must be a decimal integer" >&2
  exit 2
}
(( ${#OPENIEM_SOAK_SECONDS} <= 5 )) || { echo software stability gate: duration exceeds 86400 seconds >&2; exit 2; }
SOAK_SECONDS=$((10#$OPENIEM_SOAK_SECONDS))
(( SOAK_SECONDS <= 86400 )) || { echo software stability gate: duration exceeds 86400 seconds >&2; exit 2; }
python3 scripts/validate-deb-package.py "$1"
python3 - <<'PY'
from pathlib import Path
import subprocess
for p in Path('packaging/deb/DEBIAN').glob('*'):
    if p.name in {'postinst','prerm','postrm'}:
        subprocess.run(['bash','-n',str(p)],check=True)
print('package maintainer scripts: PASS')
PY
if (( SOAK_SECONDS < 1 )); then
  echo 'software stability gate: invalid duration' >&2
  exit 2
fi

# Bounded-command deterministic software regression loop. This exercises CODE/SIMULATED
# audio and media paths; it does not claim realtime, PipeWire, WebRTC, or
# hardware stability. Profile deadline is checked between bounded commands; each gets a small completion grace.
start_epoch=$SECONDS
iterations=0
while :; do
  remaining=$((SOAK_SECONDS - (SECONDS - start_epoch)))
  (( remaining > 0 )) || break
  if ! timeout --signal=TERM --kill-after=5s "$((remaining + 5))s" scripts/ci/run-headless-audio.sh; then
    echo 'SOFTWARE stability soak: FAIL (headless audio regression)' >&2
    exit 1
  fi
  remaining=$((SOAK_SECONDS - (SECONDS - start_epoch)))
  (( remaining > 0 )) || break
  if ! timeout --signal=TERM --kill-after=5s "$((remaining + 5))s" cargo test --manifest-path server/Cargo.toml --package streaming --lib; then
    echo 'SOFTWARE stability soak: FAIL (streaming regression)' >&2
    exit 1
  fi
  iterations=$((iterations + 1))
done

if (( iterations == 0 )); then
  echo 'SOFTWARE stability soak: FAIL (no complete audio-media iteration)' >&2
  exit 1
fi

printf 'PACKAGE_RELEASE_GATE: PARTIAL (package validation and maintainer-script syntax)\n'
printf 'SOFTWARE stability soak: PASS (deterministic CODE/SIMULATED audio-media loop; requested_seconds=%s; iterations=%s)\n' "$SOAK_SECONDS" "$iterations"
