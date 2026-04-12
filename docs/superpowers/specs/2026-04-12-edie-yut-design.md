# EDIE 초능력 윷놀이 — Design Spec

> AeiROBOT에 관심 있는 개발자가 만든 팬게임
> Date: 2026-04-12

## 1. Overview

한국 전통 보드게임 윷놀이에 에디의 초능력을 더한 2~4인 전략 대전.
윷을 던지고, 말을 옮기고, 초능력으로 판세를 뒤집는 턴제 보드게임.

### Success Criteria
1. 2~4인 로컬 대전이 완전히 플레이 가능
2. 전통 윷놀이 규칙 충실 재현 (도/개/걸/윷/모, 잡기, 업기, 지름길)
3. 16종 초능력 카드 시스템으로 전략적 깊이 제공
4. PC + 모바일 모두 터치/클릭으로 조작 가능
5. AeiROBOT 브랜드 색상 (주황→초록 그라데이션) 적용
6. 단일 .wasm 파일로 브라우저 배포

## 2. Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust 1.84 | 기존 인프라 재활용 |
| Engine | macroquad 0.4 | 경량 2D WASM |
| Target | wasm32-unknown-unknown | 브라우저 배포 |
| Art | Procedural + EDIE 에셋 | 기존 generate_art.py 활용 |
| Audio | macroquad OGG | 기존 SFX 파이프라인 |

## 3. Game Design

### 3.1 Core Loop
1. 플레이어가 윷을 던짐 (화면 탭/클릭)
2. 결과에 따라 이동할 말과 경로 선택
3. 말 이동 → 잡기/업기/지름길 처리
4. 초능력 카드 사용 가능 (보유 시)
5. 윷/모 → 보너스 턴
6. 모든 말이 도착하면 승리

### 3.2 Board Layout
전통 십자형 말판 (29칸):

```
        [5]---[6]---[7]---[8]---[9]---[10]
         |  \                         / |
        [4]  [20]                 [22] [11]
         |     \                 /     |
        [3]    [21]           [23]    [12]
         |       \           /        |
        [2]      [CENTER=24]         [13]
         |       /           \        |
        [1]    [27]           [25]   [14]
         |     /                 \    |
        [0]  [28]                [26] [15]
         |  /                         \ |
       [19]--[18]--[17]--[16]--[15]--[EXIT]
```

- 외곽 20칸: 0~19 (반시계 방향)
- 대각선 A (5→중앙→19방향): 20, 21, 24(중앙), 27, 28
- 대각선 B (10→중앙→19방향): 22, 23, 24(중앙), 25, 26
- 출발: 0번 칸 / 도착: 19번 지나면 완주

### 3.3 Yut Throw (윷 던지기)

4개의 윷가락, 각각 50% 확률로 앞(flat)/뒤(round):

| Result | Flats | Move | Bonus Turn |
|--------|-------|------|------------|
| 도 (Do) | 1 | 1칸 | No |
| 개 (Gae) | 2 | 2칸 | No |
| 걸 (Geol) | 3 | 3칸 | No |
| 윷 (Yut) | 4 | 4칸 | Yes |
| 모 (Mo) | 0 | 5칸 | Yes |

### 3.4 Piece Rules
- 각 플레이어 4개 말 (EDIE 색상 변형)
- **잡기**: 상대 말이 있는 칸에 도착 → 상대 말 출발점으로 귀환 + 보너스 턴
- **업기**: 자기 말이 있는 칸에 도착 → 합체 (함께 이동)
- **지름길**: 코너(5, 10)에서 대각선 경로 선택 가능
- **완주**: 마지막 칸을 넘기면 도착 (정확히 맞출 필요 없음)

### 3.5 Superpowers (16종)

매 3턴마다 랜덤 초능력 카드 1장 획득 (최대 2장 보유):

| # | Name | Effect |
|---|------|--------|
| 1 | 오로라 대시 | 이번 이동에 +2칸 추가 |
| 2 | 바이러스 트랩 | 빈 칸에 함정 설치 (밟으면 귀환) |
| 3 | 강제 귀가 | 상대 말 1개를 출발점으로 보냄 |
| 4 | 텔레포트 | 내 말 1개를 아무 빈 칸으로 이동 |
| 5 | 방어막 | 내 말 1개에 2턴간 잡기 방지 |
| 6 | 밥상 뒤집기 | 내 말과 상대 말 위치 교환 |
| 7 | 마이더스의 손 | 다음 윷 결과 × 2배 |
| 8 | 시간 역행 | 상대의 마지막 이동 취소 |
| 9 | 합체 소환 | 떨어진 내 말 2개를 한 칸에 합침 |
| 10 | 분열 | 상대 업힌 말을 개별 분리 |
| 11 | 몽치 소환 | 칸 1개를 3턴간 통행 불가 |
| 12 | 에너지 드레인 | 상대 초능력 카드 1장 뺏기 |
| 13 | 윷 조작 | 다음 던지기 결과를 직접 선택 |
| 14 | 연속 턴 | 즉시 추가 턴 획득 |
| 15 | 부활 | 잡힌 말이 즉시 출발점에서 재등장 |
| 16 | 정찰 | 상대 보유 초능력 카드 공개 |

### 3.6 Player Colors (AeiROBOT Theme)

| Player | Color | Hex |
|--------|-------|-----|
| P1 (EDIE) | Orange | #E8923C |
| P2 (Alice) | Green | #5BE3A8 |
| P3 (Amy) | Purple | #9B6FD4 |
| P4 (BoxBot) | Cyan | #4FC3F7 |

## 4. Architecture

```
src/yut/
  mod.rs          # module re-exports
  board.rs        # Board graph, position connections, path resolution
  game.rs         # Game state machine, turns, win condition
  pieces.rs       # Piece state, capture, stacking
  powers.rs       # 16 superpowers logic
  throw.rs        # Yut stick throw mechanics + animation
  render.rs       # Board, pieces, HUD, menus (macroquad)

src/bin/edie_yut.rs   # macroquad entry point
```

### Data Flow
```
Input → Throw → SelectPiece → MovePiece → Capture/Stack → CheckWin → NextTurn
                                ↑
                          UsePowerup (optional)
```

## 5. Persistence

- 리더보드: 최단 턴 수 기록 (jsonblob.com 동기화)
- 전적: localStorage에 승/패 기록
- 기존 `edie_runner.leaderboard.v3` 키 패턴 재활용하되 버전 체크 제거

## 6. Phased Delivery

### M1 — Board + Core Rules (이번 세션)
- 보드 데이터 구조 (29칸 그래프)
- 윷 던지기 시뮬레이션
- 말 이동, 잡기, 업기
- 지름길 경로 선택
- 승리 판정
- 단위 테스트 15+개

### M2 — Rendering + Input
- 보드 그리기 (십자형, AeiROBOT 테마)
- 말 에셋 렌더링 (플레이어별 색상)
- 윷 던지기 애니메이션
- 터치/클릭 입력
- 메뉴 (2P/3P/4P 선택)
- WASM 빌드 & 배포

### M3 — Superpowers + Polish
- 16종 초능력 카드 구현
- 카드 UI (획득, 보유, 사용)
- SFX (던지기, 이동, 잡기, 초능력)
- AI 상대 (Easy/Hard)
- 온라인 매칭 (선택)

## 7. Testing

### Unit Tests (M1)
- 보드 그래프 연결 무결성
- 각 윷 결과의 이동 거리
- 일반 경로 이동
- 지름길 경로 이동
- 잡기 메카닉
- 업기 메카닉
- 보너스 턴 (윷/모/잡기)
- 승리 판정
- 2~4인 턴 순환

## 8. Out of Scope (M1)
- 온라인 멀티플레이
- AI 상대
- 빽도 (특수 뒷걸음)
- 애니메이션
- 사운드

## 9. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| 지름길 경로 복잡성 | High | 그래프 기반 경로 탐색 + 충분한 테스트 |
| 초능력 밸런스 | Medium | M3에서 플레이테스트 후 조정 |
| 4인 UI 복잡도 | Medium | 최소한의 HUD, 현재 턴 강조 |
