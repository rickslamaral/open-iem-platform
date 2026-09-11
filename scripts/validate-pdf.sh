#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
PDF_INPUT="${1:-docs/guides/MUSICIANS-GUIDE.pdf}"
if [[ "$PDF_INPUT" == -* ]]; then
  printf 'PDF path must not begin with a dash: %s\n' "$PDF_INPUT" >&2
  exit 2
fi
PDF="$(realpath -e -- "$REPO_ROOT/$PDF_INPUT" 2>/dev/null || true)"
DOCS_GUIDES_ROOT="$REPO_ROOT/docs/guides/"
if [[ -z "$PDF" || "$PDF" != "$DOCS_GUIDES_ROOT"*.pdf ]]; then
  printf 'PDF path must resolve under docs/guides and end in .pdf: %s\n' "$PDF_INPUT" >&2
  exit 2
fi
if [[ ! -s "$PDF" ]]; then
  printf 'PDF missing or empty: %s\n' "$PDF_INPUT" >&2
  exit 1
fi

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

if command -v pdftotext >/dev/null 2>&1; then
  pdftotext "$PDF" "$WORKDIR/content.txt"
elif command -v mutool >/dev/null 2>&1; then
  mutool draw -F txt -o "$WORKDIR/content.txt" "$PDF" >/dev/null
else
  printf 'PDF text extractor missing: install poppler-utils (pdftotext) or MuPDF (mutool).\n' >&2
  exit 2
fi

if [[ ! -s "$WORKDIR/content.txt" ]]; then
  printf 'PDF text extraction produced empty output: %s\n' "$PDF" >&2
  exit 1
fi

for required in 'Open IEM Platform' 'SIMULATED' 'Raspberry Pi 5'; do
  if ! grep -Fq "$required" "$WORKDIR/content.txt"; then
    printf 'PDF content missing required text: %s\n' "$required" >&2
    exit 1
  fi
done

mkdir -p "$WORKDIR/render"
if command -v mutool >/dev/null 2>&1; then
  mutool draw -r 120 -o "$WORKDIR/render/page-%03d.png" "$PDF" >/dev/null
elif command -v pdftoppm >/dev/null 2>&1; then
  pdftoppm -png -r 120 "$PDF" "$WORKDIR/render/page" >/dev/null
else
  printf 'PDF renderer missing: install MuPDF (mutool) or Poppler (pdftoppm).\n' >&2
  exit 2
fi

if ! compgen -G "$WORKDIR/render/page*.png" >/dev/null; then
  printf 'PDF render produced no pages: %s\n' "$PDF" >&2
  exit 1
fi

printf 'PDF validation passed: %s\n' "$PDF"
