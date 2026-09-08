#!/usr/bin/env bash
# scripts/validate-skills.sh
# Validates all project skills in .agents/skills/

set -euo pipefail

SKILLS_DIR=".agents/skills"
ERRORS=0
CHECKED=0

if [ ! -d "$SKILLS_DIR" ]; then
  echo "ERROR: Skills directory not found: $SKILLS_DIR"
  exit 1
fi

echo "=== Open IEM Platform — Skill Validation ==="
echo ""

for skill_dir in "$SKILLS_DIR"/*/; do
  skill_name=$(basename "$skill_dir")
  skill_file="$skill_dir/SKILL.md"
  CHECKED=$((CHECKED + 1))

  echo "Checking: $skill_name"

  # SKILL.md exists
  if [ ! -f "$skill_file" ]; then
    echo "  ERROR: SKILL.md not found in $skill_dir"
    ERRORS=$((ERRORS + 1))
    continue
  fi

  # Frontmatter: check for opening ---
  if ! head -1 "$skill_file" | grep -q "^---"; then
    echo "  ERROR: SKILL.md missing YAML frontmatter (no opening ---)"
    ERRORS=$((ERRORS + 1))
  fi

  # Has 'name:' field
  if ! grep -q "^name:" "$skill_file"; then
    echo "  ERROR: SKILL.md missing 'name:' field"
    ERRORS=$((ERRORS + 1))
  fi

  # Has 'description:' field
  if ! grep -q "^description:" "$skill_file"; then
    echo "  ERROR: SKILL.md missing 'description:' field"
    ERRORS=$((ERRORS + 1))
  fi

  # Description is not empty
  desc=$(grep "^description:" "$skill_file" | sed 's/^description: *//')
  if [ -z "$desc" ]; then
    echo "  ERROR: 'description:' field is empty"
    ERRORS=$((ERRORS + 1))
  fi

  # Check directory name matches (loosely — underscore/hyphen equivalence)
  declared_name=$(grep "^name:" "$skill_file" | sed 's/^name: *//')
  dir_normalized=$(echo "$skill_name" | tr '_' '-')
  name_normalized=$(echo "$declared_name" | tr '_' '-')
  if [ "$dir_normalized" != "$name_normalized" ]; then
    echo "  WARNING: directory name '$skill_name' does not match declared name '$declared_name'"
  fi

  # File has content beyond frontmatter
  line_count=$(wc -l < "$skill_file")
  if [ "$line_count" -lt 10 ]; then
    echo "  WARNING: SKILL.md seems too short ($line_count lines) — may be a stub"
  fi

  echo "  OK"
done

echo ""
echo "=== Results ==="
echo "Checked: $CHECKED skills"
echo "Errors:  $ERRORS"

if [ "$ERRORS" -gt 0 ]; then
  echo "FAIL: $ERRORS error(s) found"
  exit 1
else
  echo "PASS: All skills valid"
  exit 0
fi
