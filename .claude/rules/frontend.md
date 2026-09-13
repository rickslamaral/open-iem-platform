---
paths:
  - "web/**/*.ts"
  - "web/**/*.tsx"
  - "web/**/package.json"
---

# Frontend Rules

- Run typecheck, tests, and build in each affected app.
- Preserve WebSocket revision ordering and stale-state rejection.
- Validate server messages at the boundary; do not trust browser input.
- Keep musician controls accessible on mobile and keyboard usable.
