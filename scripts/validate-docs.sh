#!/usr/bin/env bash
# Validate required project documentation and release metadata.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

required_files=(
  README.md CHANGELOG.md START.md docs/TODO.md docs/DEVELOPMENT-LOG.md
  docs/reviews/PHASE-12-REVIEW.md docs/guides/MUSICIANS-GUIDE.md
  docs/guides/MUSICIANS-GUIDE.pdf
)
for file in "${required_files[@]}"; do
  [[ -f "$file" ]] || { printf 'missing required document: %s\n' "$file" >&2; exit 1; }
done

version="$(sed -n '/^\[workspace.package\]/,/^\[/ { /^version = / { s/.*= "\([^"]*\)"/\1/p; } }' server/Cargo.toml)"
semver_re='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'
[[ "$version" =~ $semver_re ]] || {
  printf 'invalid workspace SemVer: %s\n' "$version" >&2
  exit 1
}
# SemVer forbids leading zeroes in numeric prerelease identifiers.
prerelease="${version#*-}"
if [[ "$version" == *-* ]]; then
  prerelease="${prerelease%%+*}"
  IFS=. read -ra prerelease_parts <<< "$prerelease"
  for part in "${prerelease_parts[@]}"; do
    if [[ "$part" =~ ^[0-9]+$ && "$part" != 0 && "$part" == 0* ]]; then
      printf 'invalid numeric prerelease identifier: %s\n' "$part" >&2
      exit 1
    fi
  done
fi

grep -q '^## \[Unreleased\]$' CHANGELOG.md || {
  printf 'CHANGELOG.md lacks [Unreleased] section\n' >&2
  exit 1
}
[[ "$(grep -c '^## \[Unreleased\]$' CHANGELOG.md)" -eq 1 ]] || {
  printf 'CHANGELOG.md must contain exactly one [Unreleased] section\n' >&2
  exit 1
}
[[ -s docs/guides/MUSICIANS-GUIDE.pdf ]] || {
  printf 'musician guide PDF is empty\n' >&2
  exit 1
}

printf 'documentation validation passed (version %s)\n' "$version"
