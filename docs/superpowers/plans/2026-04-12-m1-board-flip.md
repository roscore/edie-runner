# M1 Plan: Board + Flip Logic

**Date:** 2026-04-12
**Milestone:** M1 of Battle Reverse
**Spec ref:** `docs/superpowers/specs/2026-04-12-battle-reverse-design.md` §2

## Goal

Implement the core Othello board data structure and flip logic as a
pure-logic Rust module with no graphics dependency.

## Deliverables

- `src/reversi/board.rs`: Board struct, Cell/Side enums, 8-dir flip,
  HP damage system, Mungchi Alert, turn management, game-over detection.
- `src/reversi/mod.rs`: module re-exports.
- 12 unit tests covering all core logic paths.
- M1 plan document.
