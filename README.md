# EDIE Minigames

AeiROBOT에 관심 있는 개발자가 만든 팬게임 컬렉션. AeiROBOT 마스코트 **EDIE**와 함께하며, Rust + WebAssembly로 구동됩니다. 브라우저에서 바로 플레이할 수 있습니다.

**[미니게임 포털 바로가기](https://roscore.github.io/edie-minigames/)**

---

## Games

| Game | Genre | Status | Link |
|------|-------|--------|------|
| **EDIE Runner** | Endless Runner | PLAYABLE | [Play](https://roscore.github.io/edie-minigames/runner/) |
| **EDIE Battle Reverse** | Board / Othello | IN DEV | [Play](https://roscore.github.io/edie-minigames/reverse/) |
| **EDIE 초능력 윷놀이** | Board / Yut Nori | COMING SOON | [Play](https://roscore.github.io/edie-minigames/yut/) |

---

### EDIE Runner

판교 백화점에서 안산 AeiROBOT 본사까지, 에디의 집 찾기 대모험.

- 7개 스테이지 (판교 백화점 → 판교 거리 → 고속도로 → 한양대 ERICA → AeiROBOT 사무실 → 대표실 → 보스전)
- 50000점 돌파 시 **몽치 보스전** (60초 서바이벌, 5가지 공격 패턴)
- 오로라 스톤 수집 → **오로라 대시** (400ms 무적 + 장애물 파괴)
- 하트 수집으로 추가 목숨 (최대 3개)

**조작:**  Space/Up = 점프 | Down = 덕 | Shift = 대시 | P = 일시정지

### EDIE Battle Reverse

에디 vs 앨리스, 오셀로 기반 전략 보드게임.

- 1v1 대전 (vs AI: Easy / Normal / Hard)
- 오로라 파워업 + 몽치 바이러스 확산 메카닉
- HP 기반 승부 시스템

### EDIE 초능력 윷놀이

전통 윷놀이에 16종 초능력을 추가한 전략 대결.

- 강제 귀가, 밥상 뒤집기, 마이더스의 손 등 초능력 카드
- 온라인/로컬 멀티플레이 예정

---

## Build

### Prerequisites

- Rust 1.84+ (`rust-toolchain.toml`로 자동 설치)
- `rustup target add wasm32-unknown-unknown`
- Python 3 + Pillow + numpy (아트/SFX 생성)
- Optional: `wasm-opt` (Binaryen 116+)

### Local build & run

```bash
# Linux / macOS
./scripts/build.sh
cd web && python -m http.server 8080
```

```powershell
# Windows
./scripts/build.ps1
cd web; python -m http.server 8080
```

Then open http://localhost:8080 in Chrome.

### Tests

```bash
cargo test --lib    # 68+ unit tests
```

Physics, obstacles, dash, score, difficulty, camera, boss patterns 등을 커버합니다.

---

## Deployment

GitHub Actions (`deploy.yml`)가 `main` / `edie-runner` / `edie-reverse` / `edie-yut` 브랜치 push 시 자동 배포합니다.

```
site/
  index.html       ← 랜딩 페이지 (게임 선택)
  runner/           ← EDIE Runner (wasm)
  reverse/          ← Battle Reverse (wasm)
  yut/              ← 윷놀이 (placeholder)
```

Pages 활성화: Settings → Pages → Source: "GitHub Actions"

## Project structure

```
edie-minigames/
  src/
    main.rs              # macroquad entry point
    game/                # pure game logic (platform-independent)
      player.rs, obstacles.rs, boss.rs, pickups.rs,
      dash.rs, effects.rs, world.rs, state.rs, difficulty.rs
    render/              # camera, sprites, UI
    platform/            # Input/Storage/Visibility trait seams
  tools/
    generate_art.py      # procedural sprite + SFX generator
    generate_extras.py   # shop tiles, gate, BGM
  web/                   # WASM host HTML + JS
  assets/                # source art + generated assets
  .github/workflows/     # CI/CD
```

## Branch strategy

| Branch | Purpose |
|--------|---------|
| `main` | Protected, release only |
| `develop` | Integration branch |
| `edie-runner` | Runner game source |
| `edie-reverse` | Battle Reverse game source |
| `edie-yut` | Yut Nori game source |
| `fix/*`, `feat/*` | Feature/fix branches → PR to develop |
