# Bevy Asteroids — Phase 5 설계 문서: 스테이지 · 테마 · 보스

- **작성일**: 2026-07-13
- **목표**: 무한 웨이브 위에 **스테이지(테마 구간) + 보스** 개념을 얹는다. 한 사이클 = 테마 풀에서 무작위로 뽑은 스테이지들이며, 각 스테이지는 테마 웨이브들을 거쳐 **그 테마의 보스**로 끝난다. Phase 5는 그 **프레임워크 + 테마 3종/보스 3종**을 구현한다.
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10` / bevy-persistent `0.11`, macOS arm64. 렌더링은 Phase 4 카툰 스프라이트(자체 SVG→PNG) 기반.
- **선행**: Phase 1~4 (main 병합 완료).

---

## 1. 배경 — 현재 진행 구조

현재는 `Wave` 리소스(숫자 하나)로 난이도만 무한 스케일한다. `wave_control`이 소행성 전멸 시 `Wave`를 올리고 다음 웨이브를 스폰한다. 스테이지·보스·테마 구분은 없다. 배경은 성운 스프라이트 1장 고정.

---

## 2. 결정 사항 (브레인스토밍 합의)

1. **전체 구조** = **무한 순환**. 사이클을 돌 때마다 난이도↑. 엔딩 없음(현재 무한 웨이브 철학의 확장).
2. **한 사이클 = 테마 풀에서 무작위 3개(중복 없이) 선택**, 무작위 순서로 진행. 3번째 보스 격파 시 다음 사이클(재추첨 + cycle↑).
3. **테마 풀 최종 목표 = 6종**(소행성대·외계함대·화염·얼음·전자기폭풍·블랙홀), **트위스트 풀 구현**. 규모가 커 **여러 Phase로 분할**한다.
4. **분할**:
   - **Phase 5(본 스펙)**: 진행 프레임워크 + 보스 인프라 + 테마 **3종**(소행성대·외계함대·화염) + 보스 **3종**. → 한 사이클(3스테이지) 완성 플레이.
   - Phase 6: +얼음(벽 튕김·미끄러움) → 풀 4.
   - Phase 7: +전자기폭풍(시야 제한) +블랙홀(중력장) → 풀 6, 랜덤 3-of-6 완성.
5. **스테이지 흐름** = 테마 웨이브 N회 클리어 → 보스 → 격파 시 다음 스테이지. 보스는 **체력을 깎는** 방식(즉사 X) + 체력 바.
6. **게임플레이 재사용** = 기존 `Wave`/`wave_control`·소행성·UFO·충돌·파워업을 재사용해 그 위에 얹는다.

---

## 3. Phase 5 상세 설계

### 3.1 진행 상태 모델

```
pub enum ThemeId { AsteroidBelt, AlienFleet, SolarFlare } // Phase 5 풀(3). 후속 Phase에서 변형 추가.

pub enum StagePhase { Waves, Boss }

#[derive(Resource)]
pub struct Progression {
    pub cycle: u32,            // 0-based. 난이도 티어.
    pub stage_in_cycle: usize, // 0..CYCLE_LEN
    pub order: Vec<ThemeId>,   // 이번 사이클의 테마 순서(길이 CYCLE_LEN, 셔플됨)
    pub phase: StagePhase,
    pub wave_in_stage: u32,    // 현재 스테이지에서 클리어한 웨이브 수
}
```

- **CYCLE_LEN** = `min(3, 풀 크기)`. Phase 5는 풀=3 → `order` = 3개 전부의 무작위 순열. (Phase 7에서 풀=6 → 6개 중 무작위 3개.)
- **현재 테마** = `order[stage_in_cycle]`.
- **난이도 지수** = `cycle * CYCLE_LEN + stage_in_cycle` 기반(기존 `asteroid_count_for_wave` 등에 넘길 값). 사이클이 돌수록 상승.
- 셔플/추첨은 순수 함수로 분리해 테스트: `pub fn pick_cycle_order(pool: &[ThemeId], count: usize, rng) -> Vec<ThemeId>` (중복 없이 count개).

### 3.2 스테이지 진행 시스템 (기존 `wave_control` 확장)

- **스테이지 시작**: 테마 배경 스프라이트 교체, `phase=Waves`, `wave_in_stage=0`, 테마 파라미터로 첫 웨이브 스폰. "STAGE — {테마명}" 배너.
- **Waves 단계**: `wave_control`이 소행성 전멸 시 `wave_in_stage += 1`. `wave_in_stage`가 **WAVES_PER_STAGE(=3)** 도달 시 → **Boss 단계로 전환**: 잔여 위험요소 정리, 보스 스폰, "⚠ BOSS" 배너.
- **Boss 단계**: 보스 활성. 일반 웨이브 소행성은 스폰 안 함(보스가 자기 위험요소 생성). 보스 체력 0 → **격파**: 큰 폭발 + 점수 보너스 + "STAGE CLEAR" → 다음 스테이지.
- **다음 스테이지**: `stage_in_cycle += 1`. `>= CYCLE_LEN`이면 **새 사이클**: `cycle += 1`, `order` 재셔플, `stage_in_cycle=0`. 이어서 스테이지 시작.
- 상태 전이(다음 스테이지/새 사이클/난이도 계산)는 순수 함수로 분리해 테스트: `advance_stage(prog) -> prog'`.

### 3.3 보스 인프라 (kind로 분기, 재사용 가능)

```
pub enum BossKind { MotherRock, Mothership, BlazingCore } // Phase 5. 후속 Phase에서 변형 추가.

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
}
```

- **피해**: 플레이어 총알이 보스 체력을 `BULLET_BOSS_DAMAGE`만큼, 특수무기 빔이 `BEAM_BOSS_DAMAGE`만큼 깎음(즉사 X). `health <= 0` → 격파.
- **보스 HP·공격 주기·점수 보너스**는 `BossKind`별 config 상수.
- **플레이어 피격**: 보스 접촉(보스 `Collider`) + 보스 탄(보스는 기존 `EnemyBullet` 재사용해 발사)에 피격. 기존 `player_damage`를 보스/보스탄까지 보도록 확장.
- **보스 체력 바**: 화면 상단 UI(`Node` + 두 겹 바), `health/max_health` 비율 표시. 보스 없을 땐 숨김.
- **보스 이동·공격**은 `BossKind`별 시스템 분기(match). Phase 6·7은 변형 추가 + 함수만 붙이면 됨.

### 3.4 3 보스 (Phase 5)

| 보스 | 테마 | 이동 | 공격 |
|---|---|---|---|
| **거대 모암** | 소행성대 | 느린 드리프트(화면 순환) | 주기적으로 소행성 파편(중/소) 방사, 높은 HP |
| **UFO 모함** | 외계함대 | 상단 좌우 스윕 | 조준탄 연사 + 주기적 소형 UFO 소환 |
| **화염 코어** | 화염 | 상단 고정(약한 부유) | 방사형 탄막(전방위 N발) 주기 + 가끔 플레이어로 돌진 후 복귀 |

- 파편/탄/소형 UFO는 기존 `spawn_asteroid`/`spawn_enemy_bullet`/`spawn_ufo` 재사용.

### 3.5 테마 효과 (스테이지별 파라미터·배경)

- 배경 스프라이트를 **테마별 3종** 중 하나로 교체(스테이지 시작 시).
- 테마 파라미터(순수 함수 `theme_params(ThemeId) -> ThemeParams`): 소행성 수 배율·속도 배율·UFO 빈도 배율 등.
  - 소행성대: 소행성 수↑ / 외계함대: UFO 빈도↑ / 화염: 소행성 속도↑
- 기존 난이도 공식에 테마 배율을 곱해 반영.

### 3.6 아트 (자체 제작 SVG → PNG, 기존 build.rs 파이프라인)

- **보스 3종** SVG: `boss_mother_rock.svg`, `boss_mothership.svg`, `boss_blazing_core.svg`
- **배경 3종** SVG: `bg_belt.svg`, `bg_fleet.svg`, `bg_flare.svg` (기존 `background.png`는 기본/메뉴용으로 유지 또는 belt로 대체)
- 갤러리 초안(팔레트·실루엣)을 기준으로 저작.

### 3.7 UI / 연출

- HUD: 기존 표시 + **"STAGE N — {테마}"**(또는 사이클/스테이지 표기), 보스전 시 **보스 체력 바**.
- 배너: 스테이지 시작 "STAGE — {테마}", 보스 등장 "⚠ BOSS", 격파 "STAGE CLEAR".
- 격파 시 큰 폭발(기존 애니메이션 폭발 확대) + 화면 흔들림.

### 3.8 기존 시스템과의 관계

- `Wave` 리소스 → `Progression`으로 대체/흡수(웨이브 카운트는 `wave_in_stage`). 기존 `Wave` 사용처(HUD, 난이도, 배너)를 진행 모델로 갱신.
- `wave_control` → 스테이지 인지(웨이브 수 도달 시 보스 전환)하도록 확장.
- `reset_game`(재시작) → `Progression` 초기화(cycle 0, 새 셔플, stage 0, Waves).
- 배경: `spawn_background`(Startup 1회) → 스테이지 시작 시 이미지 교체 방식으로 변경.

---

## 4. 테스트 전략

렌더/보스 연출은 비테스트. 순수 로직·상태 전이 위주:

- `pick_cycle_order` — 중복 없는 count개, 풀에서 선택(결정적: 순열/집합 성질 검증).
- `advance_stage` — 다음 스테이지 / 새 사이클(재셔플·cycle↑) / 난이도 지수 계산.
- Waves→Boss 전환 조건(`wave_in_stage >= WAVES_PER_STAGE`).
- 보스 체력 감소·격파 판정(`health -= dmg`, `<=0` → 격파), 점수 보너스.
- `theme_params` 테마별 파라미터.
- 기존 게임플레이 테스트 유지·통과.

---

## 5. 리스크 · 오픈 이슈

- **보스 밸런스**: HP·공격 주기·탄 속도는 실기로 조정 필요(순수 상수라 조정 용이).
- **보스전 난이도 스파이크**: 웨이브→보스 전환 시 잔여 소행성 정리 방식(즉시 제거 vs 자연 소멸) 확정 필요 → 기본은 보스 스폰 시 잔여 소행성 제거.
- **`Wave` 대체 범위**: HUD/배너/난이도에서 `Wave`를 참조하는 지점을 진행 모델로 옮기는 리팩터 규모.
- **보스 아트**: 스프라이트 크기·방향(스윕/돌진 시 회전 여부) 조정.

## 6. 범위 밖 (후속 Phase)

- 테마 04 얼음(벽 튕김·미끄러움), 05 전자기폭풍(시야 제한), 06 블랙홀(중력장)과 각 보스 — Phase 6·7. 프레임워크는 `ThemeId`/`BossKind` 변형 추가로 확장되게 설계한다.
- 보스 다단계 페이즈, 보스별 고유 드롭, 사이클 클리어 보상 등은 이번 범위 밖.
