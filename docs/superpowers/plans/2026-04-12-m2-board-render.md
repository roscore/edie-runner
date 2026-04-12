# M2 Plan: Board Rendering + Game UI

**Date:** 2026-04-12
**Milestone:** M2 of Battle Reverse
**Depends on:** M1 (board.rs)

## Goal

Render the 8x8 board, pieces, virus cells, and game HUD using macroquad.
Visual feel inspired by warm, rounded pixel-art board game UIs with
AeiROBOT orange-to-green gradient as the primary accent.

## Deliverables

- `src/reversi/render.rs`: Board grid, pieces, viruses, valid move highlights,
  hover, flip animation, HP bars, turn indicator, menu, game over screens.
- `src/reversi/game.rs`: State machine (Menu/Playing/Animating/GameOver).
- All rendering uses macroquad primitives + existing EDIE Runner asset sprites.
- No MapleStory assets referenced.
