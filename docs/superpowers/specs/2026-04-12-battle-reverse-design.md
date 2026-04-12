# AeiROBOT Battle Reverse — Design Spec

**Date:** 2026-04-12
**Status:** Draft v1
**Owner:** helle
**Engine:** Rust + macroquad 0.4 (WASM), same stack as EDIE Runner

## 1. Overview

A browser-playable Othello/Reversi variant themed around AeiROBOT's world.
Two sides — **EDIE** (white/player) and **Alice** (black/opponent) — compete
on an 8×8 board with virus-infected blocked tiles and AeiROBOT-exclusive
power-up mechanics. Reuses the EDIE Runner pixel art pipeline and deploys
to the same GitHub Pages site.

**Success criteria**

1. Loads alongside EDIE Runner from the same `web/` folder.
2. 60 FPS at 1280×720 logical resolution on integrated GPU.
3. Mouse/touch primary input (click/tap to place pieces).
4. Single-player vs AI + local 2-player mode.
5. AeiROBOT identity: EDIE/Alice pieces, virus holes, aurora power-ups.
6. Persistent win/loss record via the same jsonblob remote storage.

## 2. Game Rules (Standard Othello Base)

### 2.1 Board

- 8×8 grid, 64 cells.
- **Initial placement:** center 4 cells (d4, d5, e4, e5) alternate
  EDIE/Alice in the standard diagonal pattern.
- **Virus cells:** 5 randomly chosen empty cells are occupied by
  **Mungchi viruses** (mini boss sprites from EDIE Runner). No piece
  can be placed there. Flip paths that hit a virus cell are blocked
  (virus = wall). Visually rendered using the existing `virus_green`
  / `virus_purple` animated sprites, pulsing at 2 FPS.

### 2.2 Turn Structure

1. Active player selects a valid cell (highlighted on hover).
2. All opponent pieces bracketed in 8 directions are flipped.
3. Turn passes to the other player.
4. If no valid move exists, turn is auto-skipped.
5. Game ends when neither player can move.

### 2.3 Scoring — HP Mode (MapleStory-inspired)

Each player starts with **10,000 HP**. Flipping deals damage:

| Flipped | Damage | Per-piece |
|---------|--------|-----------|
| 1       | 100    | 100       |
| 2       | 220    | 110       |
| 3       | 360    | 120       |
| 4       | 500    | 125       |
| 5       | 680    | 136       |
| 6+      | 140×N  | 140       |

HP reaches 0 → instant loss, even if the board isn't full.

## 3. AeiROBOT-Exclusive Mechanics

These are **new** mechanics that don't exist in MapleStory's version.
They leverage EDIE Runner's existing concepts to give the game its own
identity.

### 3.1 Aurora Power-Up Tiles

3 random empty cells contain **Aurora Stones** (visible as glowing
purple/green orbs on the board). When a player places a piece on an
aurora cell, they gain one charge of that aurora's power:

| Aurora     | Color  | Power                                              |
|------------|--------|----------------------------------------------------|
| **Dash**   | Purple | Next turn: flip in a straight line up to 3 cells,   |
|            |        | ignoring gaps (pieces don't need to be contiguous). |
| **Shield** | Green  | Protect one of your pieces from being flipped for   |
|            |        | 2 turns. Shielded piece shows a green glow.         |
| **Scan**   | Gold   | Reveal the 3 highest-value moves for your next      |
|            |        | turn (shown as score overlays on valid cells).      |

Auroras respawn on a random empty cell 4 turns after being collected.

### 3.2 Virus Spread

Every **8 turns** (4 per player), one random virus cell "spreads" by
converting an adjacent empty cell into a new virus cell. This shrinks
the playable area over time and creates urgency. A virus cell that has
no adjacent empty neighbors cannot spread.

Max virus cells: **9** (5 initial + 4 spreads in a typical 32-turn game).

### 3.3 Mungchi Center Event

When the total number of flipped pieces in a single turn reaches **6 or
more**, a brief "Mungchi Alert" animation plays (screen shake + flash)
and the flipping player gains **+500 bonus HP** on top of the normal
damage dealt. This rewards aggressive multi-flip plays.

## 4. Visual Design

### 4.1 Board Theme

- **Background:** AeiROBOT lab — dark blue-grey workspace (reuse
  `bg_office_far` / `bg_factory_far` tinted darker).
- **Board frame:** metallic silver border with rivets, subtle LED
  accent strip along the top edge.
- **Cell colors:** alternating muted teal (#2a3a3f) and dark grey
  (#1e2628), AeiROBOT corporate palette.
- **Grid lines:** thin 1px bright teal (#3af0c8, 30% opacity).

### 4.2 Pieces

| Side   | Base Sprite          | Idle Animation                    |
|--------|----------------------|-----------------------------------|
| EDIE   | `edie_static_run`    | Gentle 2-frame bob (reuse runner) |
| Alice  | `obstacle_alice3`    | Static, red virus glow when enemy |

- **Flip animation:** piece rotates 180° over 0.3 s, crossfading from
  one side's sprite to the other's.
- **Place animation:** piece drops from 20 px above with a soft bounce.
- **Virus cell (Mungchi):** small Mungchi boss sprite (`boss_virus`
  scaled to cell size) or mini virus sprite (`virus_green` /
  `virus_purple`), pulsing at 2 FPS with a faint green aura.
  When a virus spreads, a mini Mungchi "splits" into the new cell
  with a brief spawn animation.

### 4.3 UI Layout (1280×720)

```
┌──────────────────────────────────────────────────────┐
│  [EDIE HP ████████░░]   TURN 12   [Alice HP ██████░░]│  <- top bar
│                                                      │
│            ┌────────────────────────┐                 │
│   EDIE     │                        │     ALICE      │
│   portrait │     8×8 BOARD (560px)  │     portrait   │
│            │                        │                 │
│   Aurora:  │                        │     Aurora:     │
│   [D][S]   └────────────────────────┘     [D][S]     │
│                                                      │
│  [Score: 14]    VIRUS SPREAD: 3 turns   [Score: 12]  │  <- bottom bar
└──────────────────────────────────────────────────────┘
```

- Board is centered, 560×560 px (70 px per cell).
- Portraits are animated EDIE/Alice sprites from the runner assets.
- HP bars are horizontal, color-coded (EDIE = teal, Alice = red).
- Aurora charges shown as small icons below portrait.

### 4.4 Effects (borrowed from EDIE Runner)

- **Flip:** particle burst (reuse `smash_burst` colors).
- **Aurora pickup:** same pickup SFX + aurora-stone sparkle.
- **Mungchi Alert:** screen shake + red flash (reuse `effects.rs`).
- **Virus spread:** green pulse + expanding ring.
- **Win/Lose:** EDIE cheer / EDIE sad animation (reuse runner sheets).

## 5. AI Opponent

Single-player mode uses a simple AI with 3 difficulty levels:

| Level  | Algorithm                                        |
|--------|--------------------------------------------------|
| Easy   | Random valid move                                |
| Normal | Greedy: pick move that flips the most pieces     |
| Hard   | Minimax with alpha-beta pruning, depth 4, using  |
|        | weighted position heuristic (corners = high)     |

The AI "thinks" for 0.5-1.0 s (artificial delay) so the player can
see its move telegraphed.

## 6. Multiplayer

### 6.1 Local 2-Player

Both players share the same screen. Turn indicator clearly shows whose
turn it is with a bouncing arrow and name highlight.

### 6.2 Online (Future)

Defer to v2. Would require a signaling server for WebRTC or a simple
WebSocket relay. Out of scope for initial release.

## 7. Persistence

- Win/loss/draw record stored in the same jsonblob endpoint as the
  EDIE Runner leaderboard (separate key: `battle_reverse`).
- Total games played, wins, average flip damage per game.

## 8. Project Structure

```
edie-runner/
├── src/
│   ├── reversi/          # NEW module
│   │   ├── mod.rs        # re-exports
│   │   ├── board.rs      # 8x8 bitboard + flip logic + virus cells
│   │   ├── ai.rs         # minimax AI
│   │   ├── game.rs       # game state machine (menu/playing/gameover)
│   │   ├── aurora.rs     # aurora power-up logic
│   │   └── render.rs     # board + piece + HUD drawing
│   ├── game/             # existing runner modules (unchanged)
│   ├── main.rs           # adds reversi mode selector
│   └── ...
├── assets/gen/           # runner sprites reused; reversi-specific
│   │                     # sprites added by generate_extras.py
│   ├── reversi_board.png
│   ├── reversi_cell_*.png
│   └── ...
└── web/
    └── index.html        # game selector: "EDIE Runner" / "Battle Reverse"
```

## 9. Milestone Plan

| Phase | Deliverable                           | Est. |
|-------|---------------------------------------|------|
| M1    | Board + flip logic + unit tests       | 2h   |
| M2    | Render: board + pieces + flip anim    | 3h   |
| M3    | Game state machine + HP system        | 2h   |
| M4    | AI opponent (easy + normal + hard)    | 2h   |
| M5    | Aurora power-ups + virus spread       | 2h   |
| M6    | UI polish: portraits, HP bars, SFX    | 2h   |
| M7    | Menu + mode select + persistence      | 1h   |
| M8    | Testing + deploy                      | 1h   |
