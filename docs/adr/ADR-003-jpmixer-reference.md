# ADR-003: JPMixer as Product/UX Reference

## Status
Accepted (Reference Only — no code copying)

## Context
Open IEM Platform needs UX and product design references. JPMixer (https://github.com/JPMixing-inc/jpmixer) is an existing open-source personal monitor mixing system with relevant UX patterns.

## Decision
Study JPMixer as a product and UX reference for:
- Mix model design
- Scene model design
- WebSocket protocol patterns
- Device management UX
- Console adapter architecture concepts
- Authorization model patterns

Do NOT copy JPMixer implementation code without verifying license compatibility.

## Rationale
- Learning from existing implementations avoids reinventing solved problems
- JPMixer addresses the same product domain (personal monitor mixing)
- Architecture patterns are transferable even if code is not copied

## Consequences
**Positive:**
- Validated UX patterns for monitor mixing
- Reference for WebSocket protocol design
- Reference for scene model

**Negative:**
- License must be verified before any code reuse
- Open IEM must remain independently designed

## Constraints
- Goal: learn from architecture
- Not: clone source code
- Any borrowed patterns must be documented with provenance
