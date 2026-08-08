#!/usr/bin/env bash
# Verifies that every Go file in upstream-go/ is accounted for in UPSTREAM_MAPPING.md.
set -u
cd "$(dirname "$0")/.."
missing=0
while IFS= read -r f; do
  if ! grep -qF "$f" UPSTREAM_MAPPING.md; then
    echo "MISSING (.go): $f"
    missing=1
  fi
done < <(cd upstream-go 2>/dev/null && git ls-files '*.go' || true)
if [ "$missing" -ne 0 ]; then exit 1; fi
echo "OK: every upstream file is accounted for in UPSTREAM_MAPPING.md"
