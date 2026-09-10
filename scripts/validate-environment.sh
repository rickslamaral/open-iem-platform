#!/usr/bin/env bash
set -u

status=0
check() {
  local label="$1" command="$2" required="$3"
  if command -v "$command" >/dev/null 2>&1; then
    printf 'OK: %s (%s)\n' "$label" "$(command -v "$command")"
  elif [[ "$required" == required ]]; then
    printf 'MISSING: %s\n' "$label"
    status=3
  else
    printf 'OPTIONAL: %s not installed\n' "$label"
  fi
}

printf 'Open IEM environment validation\n'
printf 'OS: %s\n' "$(uname -s)"
printf 'ARCH: %s\n' "$(uname -m)"
printf 'KERNEL: %s\n' "$(uname -r)"
check 'Rust/Cargo' cargo required
check 'Node.js' node required
check 'npm' npm required
check 'Docker' docker optional
if docker compose version >/dev/null 2>&1; then
  printf 'OK: Docker Compose (%s)\n' "$(docker compose version --short 2>/dev/null || printf 'plugin')"
elif command -v docker-compose >/dev/null 2>&1; then
  printf 'OK: Docker Compose (%s)\n' "$(command -v docker-compose)"
else
  printf 'OPTIONAL: Docker Compose not installed\n'
fi
check 'PipeWire' pw-cli optional
check 'ALSA utilities' aplay optional

if [[ "$(uname -s)" == Linux ]]; then
  if command -v pw-cli >/dev/null 2>&1; then
    printf 'PipeWire state: '
    pw-cli info 0 >/dev/null 2>&1 && echo 'OK' || echo 'UNKNOWN/unavailable'
  else
    echo 'PipeWire state: HARDWARE VALIDATION REQUIRED'
  fi
else
  echo 'PipeWire state: NOT APPLICABLE on this host'
fi

printf 'Hardware audio: HARDWARE VALIDATION REQUIRED\n'
printf 'Raspberry Pi ARM64 runtime: HARDWARE VALIDATION REQUIRED\n'
printf 'WebRTC media path: SIMULATED until receiver validation\n'
exit "$status"
