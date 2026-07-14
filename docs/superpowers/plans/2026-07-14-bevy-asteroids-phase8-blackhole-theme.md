# Phase 8 — 블랙홀 테마(Black Hole) + 특이점 코어 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 테마 풀에 블랙홀(Black Hole) 테마와 특이점 코어 보스를 추가한다. 블랙홀 스테이지엔 화면 중앙에 고정 블랙홀이 있어 거리² 반비례로 우주선·소행성을 끌어당기고, 사건의 지평선(중심)은 접촉 위험이다. 스테이지 다양화의 마지막 테마로 풀이 5→6이 되어 "3-of-6 = 20가지"가 완성된다.

**Architecture:** 중력은 위치를 가진 소스가 필요하므로 **엔티티 기반**이다. `BlackHole` 엔티티를 Startup에 1회 스폰(기본 숨김·비활성)하고, `BlackHoleActive` 리소스로 on/off·중력세기를 제어한다. `gravity_pull`(FixedUpdate)이 `gravity_accel` 순수 함수로 `GravityBody`(우주선·소행성)에 가속을 적분한다. 사건의 지평선은 소행성 소멸 + 기존 `player_damage`에 지평선 체크 추가(사망 로직 재사용). on/off는 `Progression` 테마 기반 stateless sync(배경/안개 패턴). 보스는 `BossKind::SingularityCore`(나선 탄 + 주기적 흡인 강화).

**Tech Stack:** Rust 2021 / Bevy 0.19 / rand 0.10. 스프라이트는 SVG→PNG(build.rs, resvg).

## Global Constraints

- **Bevy 0.19 API**: 메시지 `MessageWriter`, 단일 쿼리 `query.single()`(Result), 리소스는 플러그인 `build()`.
- **비(非)블랙홀 스테이지 동작 불변**: `BlackHoleActive` 기본 off → 중력·지평선 무동작, 기존과 100% 동일.
- **재사용**: 보스 탄은 `spawn_enemy_bullet`, 우주선 사망은 기존 `player_damage` 경로.
- **바이너리 dead_code**: 상수·함수·컴포넌트·필드는 **소비처가 있는 태스크에서** 추가(이 플랜은 그렇게 배치됨). 임시 allow 불필요.
- **테스트**: 순수 로직만. 기존 87개 유지. 각 태스크 후 `cargo test`(전부 통과) + `cargo clippy --all-targets -- -D warnings`(경고 0).
- **Git**: 커밋 한국어 + `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. 브랜치 `feat/phase8-blackhole-theme`.
- **상수 값(확정, 실기 튜닝 대상)**: `GRAVITY_STRENGTH=1_800_000.0`, `GRAVITY_MIN_DIST=40.0`, `EVENT_HORIZON=32.0`, `BLACK_HOLE_POS_Y=120.0`, `BLACK_HOLE_VISUAL=110.0`, `SINGULARITY_HEALTH_MUL=1.3`, 특이점 반경 `52.0`, `SINGULARITY_ATTACK_INTERVAL=0.5`, `SPIRAL_STEP=0.4`, `SPIRAL_ARMS=5`, `GRAVITY_INTENSIFY=2.2`, `GRAVITY_PULSE_PERIOD=6.0`, `theme_params(BlackHole)={count:0.8, speed:0.9, ufo:1.0}`.

---

## File Structure

- `src/core/config.rs` — 중력/블랙홀/특이점 상수(각 소비 태스크에서 추가).
- `src/core/logic.rs` — `gravity_accel`(T1), `gravity_boost`(T3) 순수 함수.
- `src/core/components.rs` — `GravityBody` 마커.
- `src/entities/black_hole.rs`(신규) + `src/entities.rs` — `BlackHole`, `BlackHoleActive`, 스폰, `gravity_pull`, `consume_asteroids`, 가시성, `BlackHolePlugin`.
- `src/entities/{player,asteroid}.rs` — 스폰에 `GravityBody` 부착.
- `src/systems/collision.rs` — `player_damage`에 사건의 지평선 체크(비활성 시 무동작).
- `src/main.rs` — `BlackHolePlugin` 등록.
- `src/systems/stage.rs` — `ThemeId::BlackHole`, 풀 6, `theme_params`/`theme_name` arm, `sync_black_hole`.
- `src/entities/boss.rs` — `BossKind::SingularityCore` + 각 arm, 나선 탄, 흡인 강화.
- `src/fx/sprites.rs`, `src/fx/background.rs` — `bg_void`/`black_hole`/`boss_singularity` 핸들·분기.
- `assets/sprites/src/{black_hole,bg_void,boss_singularity}.svg`.

---

## Task 1: 중력장 인프라 (기본 비활성)

`BlackHole` 엔티티를 Startup에 스폰(숨김·비활성)하고 중력/지평선 전체를 깐다. `BlackHoleActive` 기본 off라 어디서도 중력이 작동하지 않는다(동작 불변). 리소스 기반이라 `ThemeId` 변형 없이 완결.

**Files:**
- Modify: `src/core/config.rs`, `src/core/logic.rs`, `src/core/components.rs`, `src/entities/player.rs`, `src/entities/asteroid.rs`, `src/systems/collision.rs`, `src/fx/sprites.rs`, `src/entities.rs`, `src/main.rs`
- Create: `src/entities/black_hole.rs`, `assets/sprites/src/black_hole.svg`
- Test: `src/core/logic.rs`, `src/entities/black_hole.rs`

**Interfaces:**
- Produces: `gravity_accel(body,hole,strength,min_dist)->Vec2`, `GravityBody`(마커), `BlackHole`(컴포넌트), `BlackHoleActive{active:bool, strength:f32}`(리소스), `BlackHolePlugin`.

- [ ] **Step 1: 상수 추가**

`src/core/config.rs` 파일 끝(Phase 7 블록 아래)에 추가:

```rust
// Phase 8 — 블랙홀(중력장)
pub const GRAVITY_STRENGTH: f32 = 1_800_000.0; // 흡인력 계수(accel = strength / dist²)
pub const GRAVITY_MIN_DIST: f32 = 40.0;        // 중심 근처 클램프(발산 방지)
pub const EVENT_HORIZON: f32 = 32.0;           // 사건의 지평선(치명) 반경
pub const BLACK_HOLE_POS_Y: f32 = 120.0;       // 블랙홀 위치 y(우주선 스폰(0,0)과 겹치지 않게)
pub const BLACK_HOLE_VISUAL: f32 = 110.0;      // 블랙홀 스프라이트 크기(지평선보다 큼 = 경고)
```

- [ ] **Step 2: 실패 테스트 작성(gravity_accel)**

`src/core/logic.rs`의 `mod tests`에 추가:

```rust
#[test]
fn gravity_pulls_toward_hole_and_weakens_with_distance() {
    let hole = Vec2::new(0.0, 100.0);
    let near = gravity_accel(Vec2::new(0.0, 0.0), hole, 1_000_000.0, 40.0);
    let far = gravity_accel(Vec2::new(0.0, -200.0), hole, 1_000_000.0, 40.0);
    // 방향: 둘 다 +y(중심 쪽)
    assert!(near.y > 0.0 && far.y > 0.0);
    assert!(near.x.abs() < 1e-3 && far.x.abs() < 1e-3);
    // 가까울수록 강함
    assert!(near.length() > far.length());
}

#[test]
fn gravity_is_clamped_near_center() {
    let hole = Vec2::ZERO;
    // 중심과 거의 같은 지점이어도 min_dist로 클램프되어 유한
    let a = gravity_accel(Vec2::new(0.1, 0.0), hole, 1_000_000.0, 40.0);
    let max = 1_000_000.0 / (40.0 * 40.0);
    assert!(a.length() <= max + 1e-3);
}

#[test]
fn gravity_zero_at_same_point() {
    let a = gravity_accel(Vec2::ZERO, Vec2::ZERO, 1_000_000.0, 40.0);
    assert_eq!(a, Vec2::ZERO);
}
```

- [ ] **Step 3: 테스트 실패 확인**

Run: `cargo test gravity_ 2>&1 | tail -15`
Expected: `gravity_accel` 미정의로 컴파일 에러.

- [ ] **Step 4: `gravity_accel` 구현**

`src/core/logic.rs`의 순수 함수 구역(예: `reflect_edge` 아래)에 추가:

```rust
/// 블랙홀을 향한 거리² 반비례 가속. 중심 근처는 min_dist로 클램프해 발산을 막고,
/// body와 hole이 같은 지점이면 0을 반환한다.
pub fn gravity_accel(body: Vec2, hole: Vec2, strength: f32, min_dist: f32) -> Vec2 {
    let to = hole - body;
    let dir = to.normalize_or_zero();
    if dir == Vec2::ZERO {
        return Vec2::ZERO;
    }
    let d2 = to.length_squared().max(min_dist * min_dist);
    dir * (strength / d2)
}
```

- [ ] **Step 5: gravity_accel 테스트 통과 확인**

Run: `cargo test gravity_ 2>&1 | tail -6`
Expected: 세 테스트 통과.

- [ ] **Step 6: `GravityBody` 마커 + 우주선·소행성 부착**

`src/core/components.rs` 끝에 추가:

```rust
/// 블랙홀 중력의 영향을 받는 바디(우주선·소행성). 블랙홀이 활성일 때만 당겨진다.
#[derive(Component)]
pub struct GravityBody;
```

`src/entities/player.rs`의 `spawn_player_entity` 부모 번들에 `GravityBody`를 추가한다(`GameplayEntity` 근처):

```rust
            GameplayEntity,
            GravityBody,
```
그리고 player.rs import에 `GravityBody` 추가: `use crate::core::components::{Collider, GravityBody, Velocity, Wrapping};`

`src/entities/asteroid.rs`의 `spawn_asteroid` 번들에 `GravityBody` 추가(`GameplayEntity` 근처) + import 추가:

```rust
        EdgeReflect,
        GravityBody,
        GameplayEntity,
```
import: `use crate::core::components::{AngularVelocity, Collider, EdgeReflect, GravityBody, Velocity};`

- [ ] **Step 7: 블랙홀 SVG 작성**

`assets/sprites/src/black_hole.svg`(검은 원 + 빛나는 강착원반 링, 정사각 viewBox):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="46" fill="none" stroke="#ff9a3c" stroke-width="4" opacity="0.55"/>
  <circle cx="50" cy="50" r="38" fill="none" stroke="#7cf5ff" stroke-width="3" opacity="0.5"/>
  <circle cx="50" cy="50" r="26" fill="#05040a" stroke="#1a1030" stroke-width="3"/>
  <circle cx="50" cy="50" r="26" fill="none" stroke="#c8a2ff" stroke-width="1.5" opacity="0.6"/>
</svg>
```

- [ ] **Step 8: `SpriteAssets`에 `black_hole` 필드 추가**

`src/fx/sprites.rs`의 구조체·`build_sprite_assets`·`dummy_sprite_assets`에 각각 추가(`boss_tesla_core` 근처):

구조체: `pub black_hole: Handle<Image>,`
build: `black_hole: asset_server.load("sprites/black_hole.png"),`
dummy: `black_hole: h.clone(),`

- [ ] **Step 9: black_hole 모듈 작성**

`src/entities/black_hole.rs` 생성:

```rust
//! 블랙홀 중력장(블랙홀 테마). 화면 중앙에 고정 블랙홀이 거리² 반비례로 우주선·소행성을
//! 끌어당기고, 사건의 지평선(중심)은 접촉 위험이다. on/off·중력세기는 `BlackHoleActive`가
//! 제어하고(테마 무관), `systems::stage::sync_black_hole`이 블랙홀 스테이지에서 켠다.

use bevy::prelude::*;

use crate::core::components::{Collider, GravityBody, Velocity};
use crate::core::config::{
    BLACK_HOLE_POS_Y, BLACK_HOLE_VISUAL, EVENT_HORIZON, GRAVITY_MIN_DIST, GRAVITY_STRENGTH, Z_ENTITY,
};
use crate::core::logic::gravity_accel;
use crate::core::state::GameState;
use crate::entities::asteroid::Asteroid;
use crate::fx::sprites::SpriteAssets;

#[derive(Component)]
pub struct BlackHole;

/// 블랙홀 활성/중력세기. 기본 off. 블랙홀 테마에서만 켜진다.
#[derive(Resource)]
pub struct BlackHoleActive {
    pub active: bool,
    pub strength: f32,
}

impl Default for BlackHoleActive {
    fn default() -> Self {
        Self { active: false, strength: GRAVITY_STRENGTH }
    }
}

pub struct BlackHolePlugin;

impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlackHoleActive>()
            .add_systems(Startup, spawn_black_hole)
            .add_systems(Update, (update_black_hole_visibility, consume_asteroids).run_if(in_state(GameState::Playing)))
            .add_systems(FixedUpdate, gravity_pull.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_black_hole(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        BlackHole,
        Sprite {
            image: assets.black_hole.clone(),
            custom_size: Some(Vec2::splat(BLACK_HOLE_VISUAL)),
            ..default()
        },
        Transform::from_xyz(0.0, BLACK_HOLE_POS_Y, Z_ENTITY),
        Collider { radius: EVENT_HORIZON },
        Visibility::Hidden,
    ));
}

fn update_black_hole_visibility(active: Res<BlackHoleActive>, mut q: Query<&mut Visibility, With<BlackHole>>) {
    let vis = if active.active { Visibility::Visible } else { Visibility::Hidden };
    for mut v in &mut q {
        *v = vis;
    }
}

/// 활성 시 GravityBody(우주선·소행성)를 블랙홀로 끌어당긴다.
fn gravity_pull(
    time: Res<Time>,
    active: Res<BlackHoleActive>,
    holes: Query<&Transform, (With<BlackHole>, Without<GravityBody>)>,
    mut bodies: Query<(&Transform, &mut Velocity), With<GravityBody>>,
) {
    if !active.active {
        return;
    }
    let Ok(hole_tf) = holes.single() else { return };
    let hole = hole_tf.translation.truncate();
    let dt = time.delta_secs();
    for (tf, mut vel) in &mut bodies {
        let accel = gravity_accel(tf.translation.truncate(), hole, active.strength, GRAVITY_MIN_DIST);
        vel.0 += accel * dt;
    }
}

/// 활성 시 사건의 지평선에 들어온 소행성을 소멸(빨려듦)시킨다.
fn consume_asteroids(
    mut commands: Commands,
    active: Res<BlackHoleActive>,
    holes: Query<&Transform, With<BlackHole>>,
    asteroids: Query<(Entity, &Transform), With<Asteroid>>,
) {
    if !active.active {
        return;
    }
    let Ok(hole_tf) = holes.single() else { return };
    let hole = hole_tf.translation.truncate();
    for (e, tf) in &asteroids {
        if tf.translation.truncate().distance(hole) < EVENT_HORIZON {
            commands.entity(e).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_hole_defaults_inactive() {
        let d = BlackHoleActive::default();
        assert!(!d.active);
        assert_eq!(d.strength, GRAVITY_STRENGTH);
    }
}
```

- [ ] **Step 10: 모듈 등록 + 플러그인 등록**

`src/entities.rs`에 알파벳 순으로 `pub mod black_hole;` 추가(예: `asteroid` 다음, `boss` 앞).

`src/main.rs`의 플러그인 등록부에 추가(엔티티 플러그인 근처):

```rust
        .add_plugins(entities::black_hole::BlackHolePlugin)
```

- [ ] **Step 11: `player_damage`에 사건의 지평선 체크 추가**

`src/systems/collision.rs`:

import 추가:
```rust
use crate::entities::black_hole::{BlackHole, BlackHoleActive};
use crate::core::config::EVENT_HORIZON;
```
(`EXPLOSION_PARTICLES` 등 기존 config import 줄에 `EVENT_HORIZON`을 합쳐도 됨.)

`player_damage` 시그니처에 파라미터 추가:
```rust
    black_holes: Query<(&Transform, &Collider), With<BlackHole>>,
    bh_active: Res<BlackHoleActive>,
```

적 총알 체크(`if !hit { for ... enemy_bullets ... }`) **다음**, `if !hit { return; }` **앞**에 지평선 체크 추가:

```rust
    if !hit && bh_active.active {
        for (h_tf, h_col) in &black_holes {
            if circles_overlap(ppos, pr, h_tf.translation.truncate(), h_col.radius) {
                hit = true;
                break;
            }
        }
    }
```

> 실드 체크(`if shield.is_some() { return; }`)가 함수 앞에 있어 리스폰 무적 중엔 지평선도 무피해. `bh_active.active`가 false면(비블랙홀) 이 블록은 무동작.

- [ ] **Step 12: 빌드·테스트·클리피 확인**

Run: `cargo build 2>&1 | tail -5 && ls assets/sprites/black_hole.png && cargo test 2>&1 | tail -6 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: black_hole.png 생성, 전부 통과, 경고 0. (BlackHoleActive off → 중력·지평선 무동작, 동작 불변.)

- [ ] **Step 13: 커밋**

```bash
git add src/core/config.rs src/core/logic.rs src/core/components.rs src/entities/black_hole.rs src/entities.rs src/main.rs src/entities/player.rs src/entities/asteroid.rs src/systems/collision.rs src/fx/sprites.rs assets/sprites/src/black_hole.svg
git commit -m "feat: 블랙홀 중력장 인프라(gravity_accel + BlackHole 엔티티, 기본 비활성)

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 2: 블랙홀 테마 + 특이점 코어 turn-on (플레이 가능)

`ThemeId::BlackHole`과 `BossKind::SingularityCore`를 함께 도입(exhaustive match). 완료 시 블랙홀 스테이지가 등장하고 중력이 켜지며 특이점 코어가 나선 탄을 쏜다. 흡인 강화는 Task 3에서.

**Files:**
- Modify: `src/core/config.rs`, `src/systems/stage.rs`, `src/entities/boss.rs`, `src/fx/sprites.rs`, `src/fx/background.rs`
- Create: `assets/sprites/src/bg_void.svg`, `assets/sprites/src/boss_singularity.svg`

**Interfaces:**
- Consumes: `BlackHoleActive`(T1).
- Produces: `ThemeId::BlackHole`, `BossKind::SingularityCore`, `sync_black_hole`.

- [ ] **Step 1: 특이점 상수 추가**

`src/core/config.rs`의 Phase 8 블록 아래에 추가:

```rust
// Phase 8 — 특이점 코어 보스
pub const SINGULARITY_HEALTH_MUL: f32 = 1.3;
pub const SINGULARITY_ATTACK_INTERVAL: f32 = 0.5; // 나선 탄 발사 주기(짧게)
pub const SPIRAL_STEP: f32 = 0.4;                 // 발사마다 회전량(rad)
pub const SPIRAL_ARMS: u32 = 5;                   // 발사당 탄 수
```

- [ ] **Step 2: 아트 SVG 작성**

`assets/sprites/src/bg_void.svg`(기존 bg viewBox 규격 확인 후, 매우 어두운 공간 + 중심으로 늘어진 별):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180">
  <rect width="320" height="180" fill="#050308"/>
  <g fill="#b9a6ff" opacity="0.8">
    <circle cx="40" cy="30" r="1.2"/><circle cx="280" cy="40" r="1"/>
    <circle cx="90" cy="140" r="1"/><circle cx="240" cy="150" r="1.3"/>
    <circle cx="160" cy="20" r="0.9"/><circle cx="200" cy="100" r="1"/>
  </g>
  <g stroke="#3a2f66" stroke-width="1" opacity="0.5">
    <path d="M60 40 Q160 90 60 60" fill="none"/>
    <path d="M260 140 Q160 90 260 120" fill="none"/>
  </g>
  <circle cx="160" cy="90" r="10" fill="#0a0614" opacity="0.7"/>
</svg>
```

`assets/sprites/src/boss_singularity.svg`(강착원반 두른 특이점, 정사각 viewBox):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <g stroke="#0a0618" stroke-width="4" stroke-linejoin="round">
    <ellipse cx="50" cy="50" rx="46" ry="18" fill="none" stroke="#ff8a2c" stroke-width="5" opacity="0.7"/>
    <circle cx="50" cy="50" r="24" fill="#06040e"/>
  </g>
  <ellipse cx="50" cy="50" rx="46" ry="18" fill="none" stroke="#7cf5ff" stroke-width="2" opacity="0.6"/>
  <circle cx="50" cy="50" r="24" fill="none" stroke="#c8a2ff" stroke-width="1.5" opacity="0.7"/>
</svg>
```

> viewBox는 기존 `bg_belt.svg`/`boss_blazing_core.svg`와 동일 규격을 쓴다.

- [ ] **Step 3: `SpriteAssets`에 `bg_void`/`boss_singularity` 추가**

`src/fx/sprites.rs`의 구조체·build·dummy에 각각 추가(`black_hole` 근처):

구조체:
```rust
    pub black_hole: Handle<Image>,
    pub bg_void: Handle<Image>,
    pub boss_singularity: Handle<Image>,
```
build:
```rust
        black_hole: asset_server.load("sprites/black_hole.png"),
        bg_void: asset_server.load("sprites/bg_void.png"),
        boss_singularity: asset_server.load("sprites/boss_singularity.png"),
```
dummy:
```rust
        black_hole: h.clone(),
        bg_void: h.clone(),
        boss_singularity: h.clone(),
```

- [ ] **Step 4: 실패 테스트 작성**

`src/systems/stage.rs`의 `mod tests`:
```rust
#[test]
fn black_hole_params_are_softened() {
    let p = theme_params(ThemeId::BlackHole);
    assert!(p.count_mul < 1.0 && p.speed_mul < 1.0);
}
```

`src/entities/boss.rs`의 `mod tests`:
```rust
#[test]
fn singularity_is_black_hole_boss() {
    assert_eq!(boss_for_theme(ThemeId::BlackHole), BossKind::SingularityCore);
    assert!(boss_max_health(BossKind::SingularityCore, 0) > 0.0);
}
```

- [ ] **Step 5: stage.rs — BlackHole 변형 + 파라미터 + 중력 동기화**

`src/systems/stage.rs`:

import 추가: `use crate::entities::black_hole::BlackHoleActive;`

`ThemeId`에 변형 추가:
```rust
    EmStorm,
    BlackHole,
```

`THEME_POOL` 확장(길이 6):
```rust
pub const THEME_POOL: [ThemeId; 6] = [
    ThemeId::AsteroidBelt,
    ThemeId::AlienFleet,
    ThemeId::SolarFlare,
    ThemeId::FrozenField,
    ThemeId::EmStorm,
    ThemeId::BlackHole,
];
```

`theme_params` arm:
```rust
        ThemeId::BlackHole => ThemeParams { count_mul: 0.8, speed_mul: 0.9, ufo_interval_mul: 1.0 },
```

`theme_name` arm:
```rust
        ThemeId::BlackHole => "BLACK HOLE",
```

`stage_modifiers`: BlackHole은 `_ => StageModifiers::default()`가 처리(물리 트위스트 없음). 확인만.

`sync_fog_state` 근처에 중력 동기화 시스템 추가:
```rust
/// 블랙홀 스테이지에서만 중력을 켠다(Playing 중에만). 상태 비의존 동기화.
fn sync_black_hole(
    state: Res<State<GameState>>,
    prog: Res<Progression>,
    mut bh: ResMut<BlackHoleActive>,
) {
    bh.active = *state.get() == GameState::Playing && prog.current_theme() == ThemeId::BlackHole;
}
```

`StagePlugin::build`의 상시 Update 등록(`sync_fog_state`와 함께)에 추가:
```rust
        .add_systems(Update, (sync_fog_state, sync_black_hole));
```
> `sync_fog_state`가 단독 등록돼 있으면 튜플로 묶어 함께 상시 등록한다(둘 다 `run_if(Playing)` 밖).

- [ ] **Step 6: boss.rs — SingularityCore 변형 + 모든 arm + 나선 탄**

`src/entities/boss.rs`:

config import에 `SINGULARITY_ATTACK_INTERVAL, SINGULARITY_HEALTH_MUL, SPIRAL_ARMS, SPIRAL_STEP` 추가.

`BossKind`에 변형 추가:
```rust
    TeslaCore,
    SingularityCore,
```

`boss_for_theme` arm:
```rust
        ThemeId::EmStorm => BossKind::TeslaCore,
        ThemeId::BlackHole => BossKind::SingularityCore,
```

`boss_max_health` arm:
```rust
        BossKind::TeslaCore => BOSS_BASE_HEALTH * TESLA_HEALTH_MUL,
        BossKind::SingularityCore => BOSS_BASE_HEALTH * SINGULARITY_HEALTH_MUL,
```

`boss_image` arm:
```rust
        BossKind::TeslaCore => assets.boss_tesla_core.clone(),
        BossKind::SingularityCore => assets.boss_singularity.clone(),
```

`boss_radius` arm:
```rust
        BossKind::TeslaCore => 56.0,
        BossKind::SingularityCore => 52.0,
```

`attack_interval` arm:
```rust
        BossKind::TeslaCore => TESLA_ATTACK_INTERVAL,
        BossKind::SingularityCore => SINGULARITY_ATTACK_INTERVAL,
```

`boss_movement`의 match에 arm 추가(중앙 블랙홀 위 고정 + 약한 부유):
```rust
            // 특이점 코어: 중앙 블랙홀 위 고정 + 약한 부유.
            BossKind::SingularityCore => {
                tf.translation.x = (t * 0.7).sin() * 20.0;
                tf.translation.y = crate::core::config::BLACK_HOLE_POS_Y + (t * 1.1).sin() * 12.0;
            }
```

`boss_attack`의 match에 arm 추가(나선 탄: shots로 회전하는 방사 → 나선):
```rust
            // 특이점 코어: 회전하는 방사 = 나선 탄.
            BossKind::SingularityCore => {
                let base = shots as f32 * SPIRAL_STEP;
                for i in 0..SPIRAL_ARMS {
                    let ang = base + i as f32 / SPIRAL_ARMS as f32 * std::f32::consts::TAU;
                    let d = Vec2::new(ang.cos(), ang.sin());
                    spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                }
            }
```
> `shots`는 이미 `boss_attack` 공유 발사 지점에서 증가한다(테슬라/골렘과 동일). 매 발사 `base` 각이 `SPIRAL_STEP`씩 돌아 나선이 된다.

- [ ] **Step 7: background.rs — BlackHole 배경 분기**

`src/fx/background.rs`의 `theme_bg` arm 추가:
```rust
        ThemeId::EmStorm => assets.bg_storm.clone(),
        ThemeId::BlackHole => assets.bg_void.clone(),
```

- [ ] **Step 8: 빌드·테스트·클리피 확인**

Run: `cargo build 2>&1 | tail -5 && ls assets/sprites/bg_void.png assets/sprites/boss_singularity.png && cargo test 2>&1 | tail -6 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: PNG 2개 생성, 전부 통과, 경고 0.

- [ ] **Step 9: 커밋**

```bash
git add src/core/config.rs src/systems/stage.rs src/entities/boss.rs src/fx/sprites.rs src/fx/background.rs assets/sprites/src/bg_void.svg assets/sprites/src/boss_singularity.svg
git commit -m "feat: 블랙홀(Black Hole) 테마 + 특이점 코어 보스 도입

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 3: 주기적 흡인 강화 (연출 정교화)

**Files:**
- Modify: `src/core/config.rs`, `src/core/logic.rs`, `src/entities/boss.rs`
- Test: `src/core/logic.rs`

**Interfaces:**
- Consumes: `BlackHoleActive`(T1), `SingularityCore`(T2).
- Produces: `gravity_boost(t)->f32`, `singularity_gravity_pulse` 시스템.

- [ ] **Step 1: 상수 추가**

`src/core/config.rs`의 Phase 8 특이점 블록 아래에 추가:
```rust
pub const GRAVITY_INTENSIFY: f32 = 2.2;    // 흡인 강화 피크 배율
pub const GRAVITY_PULSE_PERIOD: f32 = 6.0; // 강화 주기(초)
```

- [ ] **Step 2: 실패 테스트 작성**

`src/core/logic.rs`의 `mod tests`:
```rust
#[test]
fn gravity_boost_between_one_and_intensify_with_peak() {
    use crate::core::config::GRAVITY_INTENSIFY;
    let mut saw_peak = false;
    for i in 0..300 {
        let t = i as f32 * 0.05;
        let b = gravity_boost(t);
        assert!(b >= 1.0 - 1e-6 && b <= GRAVITY_INTENSIFY + 1e-6);
        if b > (1.0 + GRAVITY_INTENSIFY) / 2.0 {
            saw_peak = true;
        }
    }
    assert!(saw_peak, "주기 안에 흡인 강화 피크가 존재해야 한다");
}
```

- [ ] **Step 3: 테스트 실패 확인**

Run: `cargo test gravity_boost 2>&1 | tail -8`
Expected: `gravity_boost` 미정의로 컴파일 에러.

- [ ] **Step 4: `gravity_boost` + 강화 시스템 구현**

`src/core/logic.rs`에 순수 함수 추가:
```rust
/// 흡인 강화 배율: 평소 1.0, 주기(GRAVITY_PULSE_PERIOD)마다 GRAVITY_INTENSIFY까지 치솟았다 회복. [1.0, INTENSIFY].
pub fn gravity_boost(t: f32) -> f32 {
    use std::f32::consts::TAU;
    let pulse = (TAU * t / crate::core::config::GRAVITY_PULSE_PERIOD).sin().max(0.0).powi(4);
    1.0 + (crate::core::config::GRAVITY_INTENSIFY - 1.0) * pulse
}
```

`src/entities/boss.rs`:

import 추가: `use crate::core::config::{GRAVITY_STRENGTH, ...};`(기존 config import에 병합), `use crate::core::logic::gravity_boost;`(기존 logic import에 병합), `use crate::entities::black_hole::BlackHoleActive;`.

`boss_combat`/`storm_emp` 근처에 시스템 추가:
```rust
/// 특이점 코어가 존재하면 블랙홀 흡인력을 주기적으로 강화한다(gravity_boost). 없으면 기본 세기.
fn singularity_gravity_pulse(
    time: Res<Time>,
    mut bh: ResMut<BlackHoleActive>,
    bosses: Query<&Boss>,
) {
    let has_singularity = bosses.iter().any(|b| b.kind == BossKind::SingularityCore);
    bh.strength = if has_singularity {
        GRAVITY_STRENGTH * gravity_boost(time.elapsed_secs())
    } else {
        GRAVITY_STRENGTH
    };
}
```

`BossPlugin::build`의 Update 튜플에 `singularity_gravity_pulse` 추가.

> `BlackHoleActive.strength`는 `gravity_pull`이 읽어 흡인력에 반영된다. 보스가 없으면 항상 `GRAVITY_STRENGTH`(기본)로 유지되어 웨이브 중엔 강화 없음.

- [ ] **Step 5: 테스트·클리피 통과 확인**

Run: `cargo test 2>&1 | tail -6 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과(신규 `gravity_boost` 포함), 경고 0.

- [ ] **Step 6: 실기 확인(선택, 사람 검증)**

Run: `cargo run` — 블랙홀 스테이지 진입 시 중력 흡인·시야 내 강착원반, 소행성 빨려듦, 중심 즉사, 특이점 나선 탄·주기적 흡인 강화 확인 후 종료.

- [ ] **Step 7: 커밋**

```bash
git add src/core/config.rs src/core/logic.rs src/entities/boss.rs
git commit -m "feat: 특이점 코어 주기적 흡인 강화(gravity_boost)

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Self-Review (작성자 체크)

- **스펙 커버리지**: 중력 인프라(T1) / 사건의 지평선 우주선·소행성(T1) / 진행·풀6·파라미터(T2) / 중력 on/off 동기화(T2) / 특이점 나선 탄(T2) / 흡인 강화(T3) / 아트(T1 black_hole, T2 bg_void·boss) / 배경(T2) — 스펙 §3 전 항목 태스크 존재. ✓
- **플레이스홀더**: 모든 스텝에 실제 코드/명령/기대값. 상수는 Global Constraints에 확정. ✓
- **dead_code 회피**: 상수는 소비 태스크에서(GRAVITY/EVENT/POS/VISUAL=T1, SINGULARITY/SPIRAL=T2, GRAVITY_INTENSIFY/PULSE=T3). T1의 `gravity_accel`은 `gravity_pull`이, `GravityBody`는 `gravity_pull`이, `BlackHole`은 Startup 스폰+쿼리가, `EVENT_HORIZON`은 지평선·콜라이더가 소비 → 임시 allow 불필요. 두 변형(BlackHole·SingularityCore)은 T2에서 함께 도입. `gravity_boost`는 T3 `singularity_gravity_pulse`가 소비. ✓
- **타입 일관성**: `BlackHoleActive{active,strength}`는 T1 정의, T2 `sync_black_hole`이 active 기록, T3 `singularity_gravity_pulse`가 strength 기록, `gravity_pull`이 둘 다 읽음. `GravityBody`는 T1 정의·부착·쿼리. `gravity_accel`는 T1, `gravity_boost`는 T3. ✓
- **회귀**: `BlackHoleActive` 기본 off → 비블랙홀 중력·지평선 무동작. `player_damage` 지평선 체크는 `bh_active.active` 게이트. 기존 보스 5종 arm 미변경. `sync_black_hole`이 Playing且BlackHole에서만 켜고 GameOver/타테마 끔. ✓
- **주의**: player.rs 부모 번들이 커지므로(GravityBody 추가) Bevy 15-튜플 한도 내인지 `cargo build`로 확인(초과 시 일부를 중첩 튜플 `()`로 감싸기). T2에서 boss.rs/stage.rs config·import 중복 병합 주의. `sync_black_hole`은 `run_if(Playing)` 밖 상시 등록(GameOver에서 끄기 위함).
