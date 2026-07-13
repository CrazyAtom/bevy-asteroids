# Bevy Asteroids Phase 5 — 스테이지·테마·보스 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 무한 웨이브 위에 스테이지(테마 구간)+보스를 얹는다 — 한 사이클=테마 풀에서 무작위로 뽑은 스테이지들이며 각 스테이지는 테마 웨이브 N회 뒤 그 테마의 보스로 끝난다(무한 순환, cycle로 난이도↑). Phase 5는 프레임워크+테마 3종/보스 3종.

**Architecture:** 신규 `systems/stage.rs`의 `Progression` 리소스가 사이클/스테이지/페이즈를 관리하고 `stage_control` 시스템이 테마 웨이브 스폰과 보스 전환을 구동한다. 신규 `entities/boss.rs`의 `Boss{kind,health}` 컴포넌트를 총알/빔이 깎고, `BossKind`별 이동/공격 시스템이 분기한다. 기존 `Wave`/`wave_control`은 `Progression`/`stage_control`로 대체한다. 아트는 자체 SVG→PNG(기존 build.rs 파이프라인).

**Tech Stack:** Rust / Bevy `0.19` / rand `0.10` / resvg(build-dep, 기존).

## Global Constraints

- **Bevy `0.19`** — 모든 API는 설치된 0.19에 맞춰 검증(불확실하면 컴파일러/소스 확인).
- **무한 순환** — 엔딩 없음. 한 사이클 = 테마 풀(Phase 5는 3종)에서 무작위 `CYCLE_LEN`개(중복 없이), cycle로 난이도↑.
- **스테이지당 웨이브 `WAVES_PER_STAGE`(=3)** 후 보스. 보스는 체력을 깎는 방식(즉사 X) + 체력 바.
- **게임플레이 로직 재사용** — 소행성·UFO·총알·충돌·파워업·이동은 그대로 재사용. 기존 게임플레이 테스트 유지·통과.
- **아트 자체 완결** — SVG 소스 커밋, PNG는 build.rs 생성(gitignore). 외부 에셋 0.
- **테스트는 순수 로직만** — 진행 전이·셔플·보스 체력/격파·테마 파라미터 등 순수 함수. 렌더/연출은 비테스트.
- **커밋 메시지·PR은 한국어**, 작성자 이메일 CrazyAtom noreply.
- **좌표 규약** — 정면 +Y. 방향성 스프라이트는 그에 맞춰 회전.

## 아트 스타일 가이드 (Phase 5 신규 SVG)

기존 카툰 톤(플랫 채색 + 두꺼운 어두운 외곽선 + 하이라이트 1톤)과 동일. 테마 갤러리 초안 팔레트 기준. 저작 주체는 컨트롤러(구현 시 Task 2에서 작성).

| 파일 | viewBox | 방향/비고 |
|---|---|---|
| `boss_mother_rock.svg` | 140×140 | 거대 돌덩이(크레이터), 무방향 |
| `boss_mothership.svg` | 200×90 | 넓은 접시 모함, 정면 아래 |
| `boss_blazing_core.svg` | 140×140 | 화염 코어(방사 스파이크) |
| `bg_belt.svg` | 640×360 | 갈색 성운(기존 background와 유사) |
| `bg_fleet.svg` | 640×360 | 푸른/보라 기계 격자 |
| `bg_flare.svg` | 640×360 | 붉은/주황 태양 근접 |

---

## 파일 구조

- `src/systems/stage.rs` — 신규: `Progression`·`ThemeId`·`StagePhase`·순수 함수·`StagePlugin`(stage_control)
- `src/entities/boss.rs` — 신규: `BossKind`·`Boss`·`boss_for_theme`·체력/격파 순수 함수·spawn/이동/공격·`BossPlugin`
- `src/systems.rs` — `pub mod stage;` 추가
- `src/entities.rs` — `pub mod boss;` 추가
- `src/core/config.rs` — 스테이지/보스/테마 상수
- `src/main.rs` — `StagePlugin`·`BossPlugin` 등록, `Wave` 사용 제거
- `src/core/state.rs` — `Wave` 제거, `Progression` 초기화(reset)
- `src/entities/asteroid.rs` — `wave_control`/`spawn_initial_wave` 제거(스폰은 stage가 담당), `spawn_wave`는 테마 파라미터 받게
- `src/ui.rs` — HUD 스테이지 표기, 스테이지/보스 배너, 보스 체력 바
- `src/systems/collision.rs` — 총알/빔 ↔ 보스 체력, 보스/보스탄 ↔ 플레이어
- `src/fx/background.rs` — 테마별 배경 교체
- `src/fx/sprites.rs` — 보스/배경 스프라이트 핸들 추가
- `assets/sprites/src/*.svg` — 신규 6종

---

## Task 1: 진행 모델(Progression) + 순수 함수

**Files:**
- Create: `src/systems/stage.rs`
- Modify: `src/systems.rs`, `src/core/config.rs`
- Test: `src/systems/stage.rs`(하단 tests)

**Interfaces:**
- Produces: `enum ThemeId{AsteroidBelt,AlienFleet,SolarFlare}`, `THEME_POOL: [ThemeId;3]`, `enum StagePhase{Waves,Boss}`, `Resource Progression{cycle:u32, stage_in_cycle:usize, order:Vec<ThemeId>, phase:StagePhase, wave_in_stage:u32}`, `Progression::current_theme(&self)->ThemeId`, `pick_cycle_order(pool,count)->Vec<ThemeId>`, `advance_stage(&mut Progression)`, `stage_wave_number(&Progression)->u32`, `struct ThemeParams{count_mul,speed_mul,ufo_interval_mul: f32}`, `theme_params(ThemeId)->ThemeParams`, `new_progression()->Progression`. 상수 `CYCLE_LEN:usize=3`, `WAVES_PER_STAGE:u32=3`.

- [ ] **Step 1: config 상수 추가**

`src/core/config.rs` 하단에:

```rust
// Phase 5 — 스테이지/보스
pub const CYCLE_LEN: usize = 3;        // 한 사이클의 스테이지 수(= 무작위로 뽑을 테마 수)
pub const WAVES_PER_STAGE: u32 = 3;    // 스테이지당 보스 전 웨이브 수
```

- [ ] **Step 2: `src/systems.rs`에 모듈 추가**

```rust
pub mod collision;
pub mod movement;
pub mod stage;
```

- [ ] **Step 3: 실패 테스트 작성** (`src/systems/stage.rs`, 테스트만 먼저)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_order_is_distinct_subset() {
        for _ in 0..20 {
            let o = pick_cycle_order(&THEME_POOL, CYCLE_LEN);
            assert_eq!(o.len(), CYCLE_LEN);
            for (i, a) in o.iter().enumerate() {
                assert!(THEME_POOL.contains(a));
                assert!(!o[..i].contains(a), "중복 없음");
            }
        }
    }

    #[test]
    fn advance_within_cycle_then_new_cycle() {
        let mut p = new_progression();
        p.phase = StagePhase::Boss;
        p.wave_in_stage = WAVES_PER_STAGE;
        let c0 = p.cycle;
        advance_stage(&mut p); // 0 -> 1
        assert_eq!(p.stage_in_cycle, 1);
        assert_eq!(p.cycle, c0);
        assert_eq!(p.phase, StagePhase::Waves);
        assert_eq!(p.wave_in_stage, 0);
        advance_stage(&mut p); // 1 -> 2
        advance_stage(&mut p); // 2 -> 새 사이클
        assert_eq!(p.stage_in_cycle, 0);
        assert_eq!(p.cycle, c0 + 1);
        assert_eq!(p.order.len(), CYCLE_LEN);
    }

    #[test]
    fn wave_number_increases_with_progress() {
        let mut p = new_progression();
        let a = stage_wave_number(&p);
        p.wave_in_stage = 2;
        let b = stage_wave_number(&p);
        p.stage_in_cycle = 1;
        let c = stage_wave_number(&p);
        p.cycle = 1;
        let d = stage_wave_number(&p);
        assert!(a < b && b < c && c < d);
    }

    #[test]
    fn theme_params_differ() {
        assert!(theme_params(ThemeId::AsteroidBelt).count_mul > 1.0);
        assert!(theme_params(ThemeId::SolarFlare).speed_mul > 1.0);
        assert!(theme_params(ThemeId::AlienFleet).ufo_interval_mul < 1.0);
    }
}
```

- [ ] **Step 4: 테스트 실패 확인**

Run: `cargo test -p bevy-asteroids cycle_order_is_distinct_subset 2>&1 | tail -5`
Expected: FAIL(미정의 컴파일 에러).

- [ ] **Step 5: 구현 작성** (`src/systems/stage.rs` 상단)

```rust
use bevy::prelude::*;

use crate::core::config::{CYCLE_LEN, WAVES_PER_STAGE};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeId {
    AsteroidBelt,
    AlienFleet,
    SolarFlare,
}

pub const THEME_POOL: [ThemeId; 3] = [ThemeId::AsteroidBelt, ThemeId::AlienFleet, ThemeId::SolarFlare];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StagePhase {
    Waves,
    Boss,
}

#[derive(Resource)]
pub struct Progression {
    pub cycle: u32,
    pub stage_in_cycle: usize,
    pub order: Vec<ThemeId>,
    pub phase: StagePhase,
    pub wave_in_stage: u32,
}

impl Progression {
    pub fn current_theme(&self) -> ThemeId {
        self.order[self.stage_in_cycle]
    }
}

pub fn new_progression() -> Progression {
    Progression {
        cycle: 0,
        stage_in_cycle: 0,
        order: pick_cycle_order(&THEME_POOL, CYCLE_LEN),
        phase: StagePhase::Waves,
        wave_in_stage: 0,
    }
}

/// pool에서 중복 없이 count개를 무작위로 뽑는다(부분 Fisher-Yates).
pub fn pick_cycle_order(pool: &[ThemeId], count: usize) -> Vec<ThemeId> {
    use rand::RngExt;
    let mut rng = rand::rng();
    let mut v = pool.to_vec();
    let n = v.len();
    for i in 0..n.min(count) {
        let j = rng.random_range(i..n);
        v.swap(i, j);
    }
    v.truncate(count);
    v
}

/// 보스 격파 후 다음 스테이지로. 사이클 끝이면 재셔플 + cycle↑.
pub fn advance_stage(prog: &mut Progression) {
    prog.stage_in_cycle += 1;
    if prog.stage_in_cycle >= CYCLE_LEN {
        prog.cycle += 1;
        prog.stage_in_cycle = 0;
        prog.order = pick_cycle_order(&THEME_POOL, CYCLE_LEN);
    }
    prog.phase = StagePhase::Waves;
    prog.wave_in_stage = 0;
}

/// 기존 난이도 공식(asteroid_count_for_wave 등)에 넘길 '웨이브 번호'.
/// 사이클/스테이지/웨이브가 누적될수록 단조 증가.
pub fn stage_wave_number(prog: &Progression) -> u32 {
    let stages_done = prog.cycle * CYCLE_LEN as u32 + prog.stage_in_cycle as u32;
    stages_done * WAVES_PER_STAGE + prog.wave_in_stage + 1
}

pub struct ThemeParams {
    pub count_mul: f32,
    pub speed_mul: f32,
    pub ufo_interval_mul: f32,
}

pub fn theme_params(t: ThemeId) -> ThemeParams {
    match t {
        ThemeId::AsteroidBelt => ThemeParams { count_mul: 1.3, speed_mul: 1.0, ufo_interval_mul: 1.0 },
        ThemeId::AlienFleet => ThemeParams { count_mul: 1.0, speed_mul: 1.0, ufo_interval_mul: 0.6 },
        ThemeId::SolarFlare => ThemeParams { count_mul: 1.0, speed_mul: 1.4, ufo_interval_mul: 1.0 },
    }
}
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test stage:: 2>&1 | tail -4`
Expected: 4 신규 테스트 PASS.

- [ ] **Step 7: 커밋**

```bash
git add src/systems/stage.rs src/systems.rs src/core/config.rs
git commit -m "feat: 스테이지 진행 모델(Progression)과 순수 함수 추가"
```

---

## Task 2: 보스 모델 + 아트(SVG)

**Files:**
- Create: `src/entities/boss.rs`
- Modify: `src/entities.rs`, `src/core/config.rs`, `src/fx/sprites.rs`
- Create: `assets/sprites/src/{boss_mother_rock,boss_mothership,boss_blazing_core,bg_belt,bg_fleet,bg_flare}.svg`
- Test: `src/entities/boss.rs`(하단)

**Interfaces:**
- Consumes: `ThemeId`(stage), `SpriteAssets`.
- Produces: `enum BossKind{MotherRock,Mothership,BlazingCore}`, `boss_for_theme(ThemeId)->BossKind`, `Component Boss{kind:BossKind, health:f32, max_health:f32}`, `boss_max_health(BossKind, cycle:u32)->f32`, `apply_boss_damage(&mut Boss, f32)->bool`(격파 시 true). `SpriteAssets`에 `boss_mother_rock/boss_mothership/boss_blazing_core/bg_belt/bg_fleet/bg_flare: Handle<Image>` 추가.

- [ ] **Step 1: config 상수 추가**

`src/core/config.rs` Phase 5 섹션에:

```rust
pub const BULLET_BOSS_DAMAGE: f32 = 1.0;
pub const BEAM_BOSS_DAMAGE: f32 = 4.0;
pub const BOSS_BASE_HEALTH: f32 = 40.0;      // 기준 체력
pub const BOSS_HEALTH_PER_CYCLE: f32 = 20.0; // 사이클마다 증가
pub const BOSS_SCORE_BONUS: u32 = 2000;      // 격파 보너스
```

- [ ] **Step 2: `src/entities.rs`에 모듈 추가**

기존 `pub mod ...` 목록에 `pub mod boss;` 추가.

- [ ] **Step 3: 실패 테스트** (`src/entities/boss.rs`, 테스트만)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::stage::ThemeId;

    #[test]
    fn boss_kind_matches_theme() {
        assert_eq!(boss_for_theme(ThemeId::AsteroidBelt), BossKind::MotherRock);
        assert_eq!(boss_for_theme(ThemeId::AlienFleet), BossKind::Mothership);
        assert_eq!(boss_for_theme(ThemeId::SolarFlare), BossKind::BlazingCore);
    }

    #[test]
    fn health_scales_with_cycle() {
        let h0 = boss_max_health(BossKind::MotherRock, 0);
        let h2 = boss_max_health(BossKind::MotherRock, 2);
        assert!(h2 > h0);
    }

    #[test]
    fn damage_reduces_and_defeats() {
        let mut b = Boss { kind: BossKind::Mothership, health: 3.0, max_health: 10.0 };
        assert!(!apply_boss_damage(&mut b, 1.0)); // 2 남음
        assert!((b.health - 2.0).abs() < 1e-6);
        assert!(apply_boss_damage(&mut b, 5.0)); // 0 이하 → 격파
    }
}
```

- [ ] **Step 4: 실패 확인** → `cargo test boss_kind_matches_theme 2>&1 | tail -5` FAIL.

- [ ] **Step 5: 구현** (`src/entities/boss.rs` 상단)

```rust
use bevy::prelude::*;

use crate::core::config::{BOSS_BASE_HEALTH, BOSS_HEALTH_PER_CYCLE};
use crate::systems::stage::ThemeId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    MotherRock,
    Mothership,
    BlazingCore,
}

pub fn boss_for_theme(t: ThemeId) -> BossKind {
    match t {
        ThemeId::AsteroidBelt => BossKind::MotherRock,
        ThemeId::AlienFleet => BossKind::Mothership,
        ThemeId::SolarFlare => BossKind::BlazingCore,
    }
}

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
}

/// 보스 종류별 기준 체력(모암이 가장 높음) × 사이클 증가.
pub fn boss_max_health(kind: BossKind, cycle: u32) -> f32 {
    let base = match kind {
        BossKind::MotherRock => BOSS_BASE_HEALTH * 1.5,
        BossKind::Mothership => BOSS_BASE_HEALTH,
        BossKind::BlazingCore => BOSS_BASE_HEALTH * 1.2,
    };
    base + cycle as f32 * BOSS_HEALTH_PER_CYCLE
}

/// 체력을 깎고, 0 이하이면 true(격파).
pub fn apply_boss_damage(boss: &mut Boss, dmg: f32) -> bool {
    boss.health -= dmg;
    boss.health <= 0.0
}
```

- [ ] **Step 6: 보스/배경 SVG 저작 + SpriteAssets 확장**

`assets/sprites/src/`에 아트 표의 6종 SVG를 스타일 가이드대로 작성(컨트롤러). `src/fx/sprites.rs`의 `SpriteAssets`에 6개 `Handle<Image>` 필드 추가하고 `build_sprite_assets`에서 로드, `dummy_sprite_assets`에도 필드 추가.

```rust
// SpriteAssets 필드 추가:
pub boss_mother_rock: Handle<Image>,
pub boss_mothership: Handle<Image>,
pub boss_blazing_core: Handle<Image>,
pub bg_belt: Handle<Image>,
pub bg_fleet: Handle<Image>,
pub bg_flare: Handle<Image>,
// build_sprite_assets 안:
boss_mother_rock: asset_server.load("sprites/boss_mother_rock.png"),
// ...나머지 5개 동일 패턴...
// dummy_sprite_assets 안: 각 필드 h.clone()
```

- [ ] **Step 7: 테스트·빌드·커밋**

Run: `cargo build 2>&1 | tail -2 && cargo test boss:: 2>&1 | tail -3` → PASS, 6 PNG 생성.
```bash
git add src/entities/boss.rs src/entities.rs src/core/config.rs src/fx/sprites.rs assets/sprites/src
git commit -m "feat: 보스 모델(Boss/BossKind)과 보스·배경 SVG 아트 추가"
```

---

## Task 3: 스테이지 흐름 — Wave를 Progression으로 대체(Waves 단계)

이 태스크는 `Wave`/`wave_control`을 제거하고 `stage_control`이 테마 웨이브를 스폰하도록 바꾼다. **보스 전환은 임시로 즉시 advance**(다음 스테이지)로 두어 스테이지가 순환하게 하고, 실제 보스는 Task 4에서 붙인다.

**Files:**
- Modify: `src/systems/stage.rs`(StagePlugin/stage_control), `src/entities/asteroid.rs`(wave_control/spawn_initial_wave 제거, spawn_wave 시그니처), `src/core/state.rs`(Wave 제거, Progression reset), `src/ui.rs`(HUD/배너 stage화), `src/main.rs`(StagePlugin 등록, AsteroidPlugin 조정), `src/fx/background.rs`(테마 배경 교체)

**Interfaces:**
- Consumes: `Progression`, `theme_params`, `stage_wave_number`, `advance_stage`, `SpriteAssets`, `spawn_asteroid`.
- Produces: `StagePlugin`, `themed_wave_count(theme, wave_no)->usize`, `themed_speed_scale(theme, wave_no)->f32`.

- [ ] **Step 1: 테마 반영 난이도 순수 함수(stage.rs) + 테스트**

```rust
use crate::core::logic::{asteroid_count_for_wave, asteroid_speed_scale_for_wave};

pub fn themed_wave_count(theme: ThemeId, wave_no: u32) -> usize {
    let base = asteroid_count_for_wave(wave_no) as f32;
    (base * theme_params(theme).count_mul).round().max(1.0) as usize
}
pub fn themed_speed_scale(theme: ThemeId, wave_no: u32) -> f32 {
    asteroid_speed_scale_for_wave(wave_no) * theme_params(theme).speed_mul
}
```
테스트:
```rust
#[test]
fn themed_scaling_applies_multiplier() {
    let belt = themed_wave_count(ThemeId::AsteroidBelt, 1);
    let fleet = themed_wave_count(ThemeId::AlienFleet, 1);
    assert!(belt >= fleet); // belt count_mul 1.3
    assert!(themed_speed_scale(ThemeId::SolarFlare, 1) > themed_speed_scale(ThemeId::AlienFleet, 1));
}
```

- [ ] **Step 2: `state.rs`에서 `Wave` 제거, `Progression` 초기화**

- `Wave` 구조체/`insert_resource(Wave(1))` 제거.
- `reset_game`에 `mut prog: ResMut<crate::systems::stage::Progression>` 추가, `*prog = crate::systems::stage::new_progression();` 로 초기화. `wave` 파라미터/`wave.0=1` 제거.
- `GameStatePlugin`에 `.insert_resource(crate::systems::stage::new_progression())` 추가.
- 테스트 `restart_resets_ufo_spawn_timer`의 `insert_resource(Wave(5))` 제거, 대신 `insert_resource(crate::systems::stage::new_progression())` 추가(reset_game가 Progression을 요구).

- [ ] **Step 3: `asteroid.rs` 정리**

- `spawn_initial_wave`·`wave_control`·`AsteroidPlugin`의 해당 시스템 등록 제거. `AsteroidPlugin`은 `draw`가 이미 없으니 빈 플러그인이 되면 제거하고 main에서 등록도 제거(또는 유지하되 시스템 없음). **권장**: `AsteroidPlugin` 제거, `spawn_asteroid`·`random_velocity`·`random_spawn_position`는 `pub`으로 유지(stage/boss/collision이 사용).
  - `random_spawn_position`을 `pub`으로 바꿈.
- `spawn_wave`는 stage로 이동하므로 asteroid.rs에서 제거.

- [ ] **Step 4: `stage.rs`에 `StagePlugin` + `stage_control`**

```rust
use crate::core::state::{GameState, GameplayEntity};
use crate::entities::asteroid::{random_spawn_position, random_velocity, spawn_asteroid, Asteroid};
use crate::core::logic::AsteroidSize;
use crate::fx::sprites::SpriteAssets;

pub struct StagePlugin;
impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), start_first_stage)
            .add_systems(Update, stage_control.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_themed_wave(commands: &mut Commands, assets: &SpriteAssets, prog: &Progression) {
    let theme = prog.current_theme();
    let wave_no = stage_wave_number(prog);
    let count = themed_wave_count(theme, wave_no);
    let scale = themed_speed_scale(theme, wave_no);
    for _ in 0..count {
        let base = random_velocity(AsteroidSize::Large);
        spawn_asteroid(commands, assets, AsteroidSize::Large, random_spawn_position(), base * scale);
    }
}

fn start_first_stage(mut commands: Commands, assets: Res<SpriteAssets>, prog: Res<Progression>) {
    spawn_themed_wave(&mut commands, &assets, &prog);
}

/// Waves 단계: 소행성 전멸 시 다음 웨이브 또는 보스 전환.
/// (이 태스크에선 보스 대신 즉시 advance — Task 4에서 보스 스폰으로 교체)
fn stage_control(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut prog: ResMut<Progression>,
    asteroids: Query<(), With<Asteroid>>,
) {
    if prog.phase != StagePhase::Waves {
        return;
    }
    if asteroids.iter().count() != 0 {
        return;
    }
    prog.wave_in_stage += 1;
    if prog.wave_in_stage < WAVES_PER_STAGE {
        spawn_themed_wave(&mut commands, &assets, &prog);
    } else {
        // TODO(Task 4): 보스 스폰. 임시로 즉시 다음 스테이지.
        prog.phase = StagePhase::Boss;
        advance_stage(&mut prog);
        spawn_themed_wave(&mut commands, &assets, &prog);
    }
}
```

- [ ] **Step 5: `main.rs` 등록 조정**

- `AsteroidPlugin` 등록 줄 제거(제거했다면), `StagePlugin` 추가: `.add_plugins(systems::stage::StagePlugin)`.

- [ ] **Step 6: `ui.rs` 스테이지 표기**

- `update_hud`에서 `Res<Wave>` → `Res<Progression>`. 표시 문자열에 `Wave: {}` 대신 `Stage {cycle+1}-{stage_in_cycle+1} W{wave_in_stage+...}` 형태(간단히 `S{cycle}-{stage} / Wave {wave_in_stage}`).
- `announce_wave`(Wave 변화 배너) → `Progression` 변화 감지로 "STAGE — {테마명}" 배너. `theme_name(ThemeId)->&str` 헬퍼 추가(소행성대/외계함대/화염).
- `hud_shows_score_lives_wave` 테스트: `Wave` 제거에 맞춰 `insert_resource(Progression)` 사용하도록 수정.

- [ ] **Step 7: `background.rs` 테마 배경**

- `spawn_background`가 기본 배경을 스폰하되, 스테이지 시작 시 테마 배경으로 교체하는 시스템 추가: 배경 스프라이트 엔티티에 마커 `BackgroundSprite`, `Progression` 변화 시 `theme_bg(theme, &assets)->Handle<Image>`로 `Sprite.image` 교체.

- [ ] **Step 8: 빌드·테스트·실행 확인 + 커밋**

`cargo build`/`cargo test` 통과. 실행 시 스테이지가 웨이브 3회마다 넘어가며 배경/난이도가 바뀜(보스는 아직 없음).
```bash
git add -A
git commit -m "feat: Wave를 Progression으로 대체, 스테이지별 테마 웨이브 스폰"
```

---

## Task 4: 보스 스폰 + 체력 바 + 피해/격파 → 다음 스테이지

**Files:**
- Modify: `src/entities/boss.rs`(spawn_boss, BossPlugin, 기본 이동), `src/systems/stage.rs`(stage_control 보스 전환), `src/systems/collision.rs`(총알/빔↔보스, 보스/보스탄↔플레이어), `src/ui.rs`(보스 체력 바), `src/main.rs`(BossPlugin 등록)

**Interfaces:**
- Consumes: `Boss`, `boss_for_theme`, `boss_max_health`, `apply_boss_damage`, `Progression`, `Collider`, `SpriteAssets`.
- Produces: `spawn_boss(commands, assets, kind, cycle)`, `BossPlugin`, `boss_defeated`(내부), 체력 바.

- [ ] **Step 1: `boss.rs` spawn + 기본 이동 + 격파 감지**

```rust
use crate::core::components::Collider;
use crate::core::config::{BOSS_SCORE_BONUS, Z_ENTITY};
use crate::core::state::{GameState, GameplayEntity, Score};

pub fn boss_image(kind: BossKind, assets: &SpriteAssets) -> Handle<Image> {
    match kind {
        BossKind::MotherRock => assets.boss_mother_rock.clone(),
        BossKind::Mothership => assets.boss_mothership.clone(),
        BossKind::BlazingCore => assets.boss_blazing_core.clone(),
    }
}
pub fn boss_radius(kind: BossKind) -> f32 {
    match kind { BossKind::MotherRock => 55.0, BossKind::Mothership => 70.0, BossKind::BlazingCore => 45.0 }
}

pub fn spawn_boss(commands: &mut Commands, assets: &SpriteAssets, kind: BossKind, cycle: u32) {
    let hp = boss_max_health(kind, cycle);
    let r = boss_radius(kind);
    commands.spawn((
        Boss { kind, health: hp, max_health: hp },
        Sprite { image: boss_image(kind, assets), custom_size: Some(Vec2::splat(r * 2.0)), ..default() },
        Transform::from_xyz(0.0, 120.0, Z_ENTITY),
        Collider { radius: r },
        GameplayEntity,
    ));
}

pub struct BossPlugin;
impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (boss_movement, boss_defeat_check).run_if(in_state(GameState::Playing)));
    }
}

// 기본 이동(느린 좌우). Task 5~7에서 kind별로 확장.
fn boss_movement(time: Res<Time>, mut q: Query<(&Boss, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (_boss, mut tf) in &mut q {
        tf.translation.x = (t * 0.5).sin() * 200.0;
    }
}

fn boss_defeat_check(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut prog: ResMut<crate::systems::stage::Progression>,
    assets: Res<SpriteAssets>,
    bosses: Query<(Entity, &Boss)>,
) {
    // 보스가 사라졌고(격파로 despawn) phase가 Boss면 다음 스테이지로.
    // 격파 자체는 collision에서 despawn + 보너스. 여기선 "보스 없음 && Boss phase" 감지.
    use crate::systems::stage::StagePhase;
    if prog.phase != StagePhase::Boss { return; }
    if bosses.iter().count() != 0 { return; }
    // 보스 격파 완료 → 다음 스테이지
    score.0 += BOSS_SCORE_BONUS;
    crate::systems::stage::advance_stage(&mut prog);
    // 다음 스테이지 첫 웨이브 스폰
    crate::systems::stage::spawn_first_wave_of_stage(&mut commands, &assets, &prog);
}
```
(보스 이동 쿼리와 defeat_check 쿼리는 겹치지 않게 — defeat_check는 `Query<(Entity,&Boss)>` 읽기, movement는 `&mut Transform`. 충돌 없음. `spawn_first_wave_of_stage`는 stage.rs의 `spawn_themed_wave`를 pub로 노출한 것.)

주: stage.rs의 `spawn_themed_wave`를 `pub fn spawn_first_wave_of_stage`로 노출.

- [ ] **Step 2: `stage_control`의 보스 전환을 실제 스폰으로 교체**

`stage.rs` stage_control의 else 분기(Task 3의 TODO)를:
```rust
    } else {
        prog.phase = StagePhase::Boss;
        let kind = crate::entities::boss::boss_for_theme(prog.current_theme());
        crate::entities::boss::spawn_boss(&mut commands, &assets, kind, prog.cycle);
    }
```

- [ ] **Step 3: `collision.rs` 총알/빔 ↔ 보스 체력**

- 신규 시스템 `bullet_vs_boss`/`beam_vs_boss`(또는 기존 bullet_vs_ufo 옆): 총알이 보스와 겹치면 `apply_boss_damage(boss, BULLET_BOSS_DAMAGE)`, 총알 despawn, 격파면 보스 despawn + 폭발. 빔은 `BEAM_BOSS_DAMAGE`.
- 보스 격파 시 `spawn_explosion(&mut commands, &assets, pos, EXPLOSION_PARTICLES*2)` 큰 폭발 + shake.

- [ ] **Step 4: `player_damage`에 보스/보스탄 위험요소 추가**

- `player_damage`에 `bosses: Query<(&Transform,&Collider), With<Boss>>` 추가, 소행성/UFO처럼 접촉 시 피격. (보스탄은 기존 EnemyBullet 재사용 → 이미 처리됨.)

- [ ] **Step 5: `ui.rs` 보스 체력 바**

- `Boss`가 존재하면 화면 상단에 체력 바(`Node` 2겹: 배경 + `health/max_health` 폭). 없으면 숨김. 마커 `BossHealthBar`.

- [ ] **Step 6: `main.rs` BossPlugin 등록** → `.add_plugins(entities::boss::BossPlugin)`.

- [ ] **Step 7: 빌드·테스트·실행 + 커밋**

실행: 웨이브 3회 후 보스 등장, 총알로 체력 깎아 격파 시 다음 스테이지. 보스 접촉/보스탄 피격.
```bash
git add -A
git commit -m "feat: 보스 스폰·체력 바·피해/격파→다음 스테이지"
```

---

## Task 5: 보스 행동 — 거대 모암 (소행성대)

**Files:** Modify `src/entities/boss.rs`

- [ ] **Step 1: kind별 이동·공격 분기 도입 + 모암 구현**

`boss_movement`를 kind별로 분기하고, 신규 `boss_attack` 시스템 추가(타이머 리소스 or Boss에 타이머 필드). 모암:
- 이동: 느린 드리프트(화면 순환) — `Velocity`+`Wrapping` 부여로 재사용 가능(spawn 시 `Velocity(느린 무작위)`, `Wrapping`, `AngularVelocity` 추가). 그러면 movement가 처리.
- 공격: 주기적으로 `spawn_asteroid(Medium/Small)`를 보스 위치에서 방사. `boss_attack`가 `BossAttackTimer`(kind별 주기)로 발동.

구현 지침: `Boss`에 `attack_timer: Timer` 필드 추가(spawn 시 kind별 주기로 초기화). `boss_attack` 시스템이 tick 후 kind==MotherRock이면 파편 N개 방사.

```rust
// Boss 구조체에 추가:
pub attack_timer: Timer,
// spawn_boss에서 kind별 주기로 초기화(예: MotherRock 2.5s Repeating)
```

- [ ] **Step 2: 빌드·실행·커밋**

실행: 소행성대 보스가 느리게 떠다니며 주기적으로 소행성 파편을 뿜음.
```bash
git add src/entities/boss.rs && git commit -m "feat: 보스 행동 - 거대 모암(드리프트+파편 방사)"
```

---

## Task 6: 보스 행동 — UFO 모함 (외계함대)

**Files:** Modify `src/entities/boss.rs`

- [ ] **Step 1: 모함 이동·공격**

- 이동: 상단 좌우 스윕(`Transform.x = sin(t*w)*범위`, y 상단 고정).
- 공격: `attack_timer` 발동 시 플레이어로 조준탄 3발(`spawn_enemy_bullet`, `aim_direction`) + 가끔 `spawn_ufo(Small)` 소환.

`boss_movement`/`boss_attack`의 `match kind`에 Mothership 분기 추가. 조준에는 플레이어 Transform 쿼리 필요.

- [ ] **Step 2: 빌드·실행·커밋** → `feat: 보스 행동 - UFO 모함(스윕+조준연사+소환)`

---

## Task 7: 보스 행동 — 화염 코어 (화염)

**Files:** Modify `src/entities/boss.rs`

- [ ] **Step 1: 코어 이동·공격**

- 이동: 상단 고정 + 약한 부유(작은 sin).
- 공격: `attack_timer` 발동 시 방사형 탄막(`spawn_enemy_bullet` N발 균등 각도) + 가끔 플레이어 위치로 돌진 후 복귀(간단한 상태: `charge_timer`/목표 위치).

돌진은 복잡하니 최소 구현: 주기적으로 y를 아래로 낮췄다 올리는 lunge, 또는 방사 탄막만으로 시작하고 돌진은 "가끔 아래로 이동" 정도. `match kind`에 BlazingCore 분기.

- [ ] **Step 2: 빌드·실행·커밋** → `feat: 보스 행동 - 화염 코어(방사 탄막+돌진)`

---

## Task 8: 연출·배너·마무리

**Files:** Modify `src/ui.rs`, 필요한 파일, `README.md`

- [ ] **Step 1: 배너**

- 스테이지 시작 "STAGE — {테마명}", 보스 등장 "⚠ BOSS {보스명}", 격파 "STAGE CLEAR" 배너(기존 `WaveBanner` 패턴 재사용/일반화).

- [ ] **Step 2: 잔여 정리**

- 미사용 심볼/`Wave` 잔재 제거. `grep -rn "Wave" src` 로 확인.
- 보스 등장 시 잔여 소행성 제거(스펙 §5): stage_control 보스 전환에서 `Asteroid` 엔티티 despawn.

- [ ] **Step 3: clippy + 전체 테스트**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -2 && cargo test 2>&1 | tail -3`
Expected: 클린 + 전체 통과.

- [ ] **Step 4: 실행 최종 확인**

`cargo run` — 3스테이지(무작위 순서) 각 테마 배경/난이도 + 웨이브 3회 → 보스 → 격파 → 다음, 사이클 반복 시 난이도↑.

- [ ] **Step 5: README 한 줄 + 커밋**

`README.md`에 스테이지/보스 한 줄 추가.
```bash
git add -A && git commit -m "chore: 스테이지/보스 연출·배너·정리(Phase 5 마무리)"
```

---

## Self-Review 메모 (작성자 확인)

- **스펙 커버리지**: §3.1 진행모델→Task1, §3.3 보스인프라→Task2·4, §3.2 스테이지흐름→Task3·4, §3.4 3보스→Task5·6·7, §3.5 테마효과→Task3, §3.6 아트→Task2, §3.7 UI/연출→Task4·8, §3.8 기존관계(Wave대체)→Task3. 전 항목 커버.
- **타입 일관성**: `ThemeId`·`StagePhase`·`Progression`·`BossKind`·`Boss`·`boss_for_theme`·`apply_boss_damage`·`spawn_boss`·`stage_wave_number`·`themed_wave_count` — 태스크 간 명칭 일치.
- **확인 필요(구현 중 컴파일러로)**: Bevy 0.19 쿼리 충돌(boss_movement `&mut Transform` vs player 쿼리), `Res<Progression>` 변화 감지(`is_changed`), rand 0.10 `random_range`, `entities.rs` 존재/모듈 선언 방식. 불확실 시 소스 확인.
- **오픈 이슈**: 보스 밸런스(HP·주기)는 실기 조정. `Wave` 참조 지점 전수 제거는 Task3+Task8에서.
