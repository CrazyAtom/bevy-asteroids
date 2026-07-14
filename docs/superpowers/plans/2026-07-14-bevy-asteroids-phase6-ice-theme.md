# Phase 6 — 얼음 테마(Frozen Field) + 얼음 골렘 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 테마 풀에 얼음(Frozen Field) 테마와 얼음 골렘 보스를 추가한다. 얼음 스테이지에선 소행성·파편·보스가 벽에서 튕기고(반사), 우주선은 순환을 유지하되 미끄러워진다.

**Architecture:** 매 프레임 물리 트위스트를 위해 `StageModifiers`(리소스) + `EdgeReflect`(마커)를 신설한다. 경계 처리를 `wrap_around`(우주선·총알)와 신규 `reflect_or_wrap`(반사 대상)로 분리하고, `StageModifiers`는 배경 동기화와 동일한 **상태 비의존 매-프레임 동기화**로 현재 테마를 반영한다. 보스는 기존 `BossKind` 분기에 `IceGolem`을 더하고, 체력 구간별 다단계 페이즈(껍질 깨짐 → 축소·가속)를 도입한다.

**Tech Stack:** Rust 2021 / Bevy 0.19 / rand 0.10. 스프라이트는 `assets/sprites/src/*.svg` → `build.rs`(resvg) 자동 래스터화.

## Global Constraints

- **Bevy 0.19 API**: 메시지는 `MessageWriter`, 단일 쿼리는 `query.single()`(Result), 리소스는 플러그인 `build()`에서 확보. 불확실한 API는 설치된 크레이트 소스로 검증.
- **비(非)얼음 스테이지 동작 불변**: `StageModifiers` 기본값(wall_bounce=false, ship_damping=SHIP_DAMPING)에서 기존과 100% 동일해야 한다.
- **재사용**: 파편/탄은 기존 `spawn_asteroid`/`spawn_enemy_bullet`을 재사용한다.
- **테스트**: 순수 로직만 테스트. 기존 68개 테스트는 계속 통과. 각 태스크 후 `cargo test`(전부 통과) + `cargo clippy --all-targets -- -D warnings`(경고 0).
- **Git**: 커밋 메시지 한국어, 말미에 `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. 브랜치 `feat/phase6-ice-theme`.
- **상수 값(확정)**: `SHIP_DAMPING_ICE=0.997`, `ICE_GOLEM_HEALTH_MUL=1.4`, `GOLEM_DRIFT_SPEED=55.0`, `GOLEM_ATTACK_INTERVAL=2.5`, 골렘 반경 `50.0`, `theme_params(FrozenField)={count:0.9, speed:0.95, ufo:1.0}`, 페이즈 임계값 `2/3·1/3`, `golem_scale=[1.0,0.8,0.62]`, `golem_speed_mul=[1.0,1.35,1.75]`, `golem_attack_mul=[1.0,0.8,0.62]`.

---

## File Structure

- `src/core/components.rs` — `EdgeReflect` 마커 추가.
- `src/core/logic.rs` — `reflect_edge` 순수 함수 추가.
- `src/core/config.rs` — 얼음/골렘 상수 추가.
- `src/systems/movement.rs` — `StageModifiers` 리소스, `reflect_or_wrap` 시스템, `wrap_around` 제외 필터.
- `src/entities/player.rs` — `apply_ship_damping`이 `StageModifiers` 참조.
- `src/entities/asteroid.rs` — 스폰 시 `Wrapping`→`EdgeReflect`.
- `src/systems/stage.rs` — `ThemeId::FrozenField`, 풀 확장, `theme_params`/`theme_name`/`stage_modifiers`/`sync_stage_modifiers`.
- `src/entities/boss.rs` — `BossKind::IceGolem`, `Boss.phase`, 각 match arm, 골렘 공격, 다단계 페이즈.
- `src/fx/sprites.rs` — `bg_ice`/`boss_ice_golem` 핸들.
- `src/fx/background.rs` — `theme_bg` 얼음 분기.
- `assets/sprites/src/bg_ice.svg`, `assets/sprites/src/boss_ice_golem.svg` — 신규 아트.

---

## Task 1: `EdgeReflect` 마커 + `reflect_edge` 순수 함수

**Files:**
- Modify: `src/core/components.rs`
- Modify: `src/core/logic.rs`
- Test: `src/core/logic.rs` (동일 파일 tests 모듈)

**Interfaces:**
- Produces: `EdgeReflect`(마커 컴포넌트), `reflect_edge(pos: Vec2, vel: Vec2, half: Vec2) -> (Vec2, Vec2)`.

- [ ] **Step 1: `EdgeReflect` 마커 추가**

`src/core/components.rs` 끝에 추가(기존 컴포넌트 정의 스타일에 맞춰):

```rust
/// 경계에서 반사(벽 튕김) 대상임을 표시하는 마커. 실제 반사/순환 여부는
/// `StageModifiers.wall_bounce`가 결정한다(얼음 스테이지에서만 반사).
#[derive(Component)]
pub struct EdgeReflect;
```

- [ ] **Step 2: 실패하는 테스트 작성**

`src/core/logic.rs`의 `mod tests`에 추가:

```rust
#[test]
fn reflect_bounces_off_right_edge_and_flips_x() {
    let half = Vec2::new(640.0, 360.0);
    let (p, v) = reflect_edge(Vec2::new(700.0, 0.0), Vec2::new(50.0, 10.0), half);
    assert!((p.x - 640.0).abs() < 1e-4); // 경계로 클램프
    assert!(v.x < 0.0);                  // x 속도 반전(안쪽으로)
    assert!((v.y - 10.0).abs() < 1e-4);  // y는 불변
}

#[test]
fn reflect_bounces_off_bottom_edge_and_flips_y() {
    let half = Vec2::new(640.0, 360.0);
    let (p, v) = reflect_edge(Vec2::new(0.0, -400.0), Vec2::new(0.0, -20.0), half);
    assert!((p.y + 360.0).abs() < 1e-4);
    assert!(v.y > 0.0);
}

#[test]
fn reflect_leaves_inside_point_unchanged() {
    let half = Vec2::new(640.0, 360.0);
    let (p, v) = reflect_edge(Vec2::new(10.0, -20.0), Vec2::new(3.0, 4.0), half);
    assert_eq!(p, Vec2::new(10.0, -20.0));
    assert_eq!(v, Vec2::new(3.0, 4.0));
}
```

- [ ] **Step 3: 테스트 실패 확인**

Run: `cargo test -p bevy-asteroids reflect_ 2>&1 | tail -20`
Expected: 컴파일 에러(`reflect_edge` 미정의).

- [ ] **Step 4: `reflect_edge` 구현**

`src/core/logic.rs`의 `wrap_position` 함수 바로 아래에 추가:

```rust
/// 경계를 넘은 좌표를 경계로 클램프하고 해당 축 속도를 안쪽으로 반전한다(벽 반사).
/// 두 축을 독립 처리하며, 경계 안의 좌표는 위치·속도 모두 불변.
pub fn reflect_edge(pos: Vec2, vel: Vec2, half: Vec2) -> (Vec2, Vec2) {
    let mut p = pos;
    let mut v = vel;
    if p.x > half.x {
        p.x = half.x;
        v.x = -v.x.abs();
    } else if p.x < -half.x {
        p.x = -half.x;
        v.x = v.x.abs();
    }
    if p.y > half.y {
        p.y = half.y;
        v.y = -v.y.abs();
    } else if p.y < -half.y {
        p.y = -half.y;
        v.y = v.y.abs();
    }
    (p, v)
}
```

- [ ] **Step 5: 테스트 통과 확인**

Run: `cargo test 2>&1 | tail -5`
Expected: 전부 통과(기존 68 + 신규 3).

- [ ] **Step 6: 커밋**

```bash
git add src/core/components.rs src/core/logic.rs
git commit -m "feat: EdgeReflect 마커 + reflect_edge 순수 함수 추가

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 2: `StageModifiers` 리소스 + 미끄러움 연결

**Files:**
- Modify: `src/core/config.rs`
- Modify: `src/systems/movement.rs`
- Modify: `src/entities/player.rs`
- Test: `src/systems/movement.rs`, `src/entities/player.rs`

**Interfaces:**
- Consumes: 없음.
- Produces: `StageModifiers { wall_bounce: bool, ship_damping: f32 }`(리소스, `Default`), `MovementPlugin`이 `init_resource::<StageModifiers>()`.

- [ ] **Step 1: 얼음 감쇠 상수 추가**

`src/core/config.rs`의 `SHIP_BRAKE_RATE` 줄 아래에 추가:

```rust
// 얼음 테마 감쇠(기본보다 마찰↓ → 더 미끄러움).
pub const SHIP_DAMPING_ICE: f32 = 0.997;
```

- [ ] **Step 2: `StageModifiers` 리소스 정의 + 삽입**

`src/systems/movement.rs` 상단 `use` 아래에 `SHIP_DAMPING` import를 추가하고 리소스를 정의한다. 먼저 import 수정:

```rust
use crate::core::config::{HALF_HEIGHT, HALF_WIDTH, SHIP_DAMPING};
```

리소스 정의(플러그인 위, `pub struct MovementPlugin;` 바로 앞):

```rust
/// 현재 스테이지의 물리 트위스트. 배경처럼 매 프레임 현재 테마로 동기화된다.
#[derive(Resource)]
pub struct StageModifiers {
    pub wall_bounce: bool,  // 위험요소가 경계에서 반사되는가(얼음 = true)
    pub ship_damping: f32,  // 우주선 마찰(작을수록 잘 미끄러짐)
}

impl Default for StageModifiers {
    fn default() -> Self {
        Self { wall_bounce: false, ship_damping: SHIP_DAMPING }
    }
}
```

`MovementPlugin::build`에서 리소스 초기화(기존 `add_systems` 앞에 체이닝):

```rust
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StageModifiers>()
            .add_systems(FixedUpdate, ((apply_velocity, wrap_around).chain(), apply_spin));
    }
}
```

- [ ] **Step 3: `apply_ship_damping`이 `StageModifiers` 참조하도록 실패 테스트 갱신**

`src/entities/player.rs`의 두 감쇠 테스트가 이제 `StageModifiers` 리소스를 요구하도록 수정한다. 두 테스트 `ship_damping_reduces_speed_but_keeps_direction`, `ship_speed_is_capped_at_max`에서 `let mut app = App::new();` 다음 줄에 각각 삽입:

```rust
        app.insert_resource(crate::systems::movement::StageModifiers::default());
```

- [ ] **Step 4: 테스트 실패 확인**

Run: `cargo test 2>&1 | tail -20`
Expected: `apply_ship_damping`이 아직 `StageModifiers`를 받지 않아 컴파일 에러(또는 리소스 미사용). 다음 스텝에서 시그니처를 바꾼다.

- [ ] **Step 5: `apply_ship_damping` 구현 변경**

`src/entities/player.rs`:

먼저 config import에서 `SHIP_DAMPING`을 제거하고(더 이상 상수 직접 사용 안 함) `StageModifiers` import를 추가한다. import 블록에서 `SHIP_DAMPING,`를 삭제하고, `use` 목록 마지막(sprites 줄 아래)에 추가:

```rust
use crate::systems::movement::StageModifiers;
```

함수 교체:

```rust
fn apply_ship_damping(mods: Res<StageModifiers>, mut query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 *= mods.ship_damping;
        velocity.0 = velocity.0.clamp_length_max(SHIP_MAX_SPEED);
    }
}
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test 2>&1 | tail -5 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과, 경고 0. (기본 감쇠값이 SHIP_DAMPING이라 동작 불변.)

- [ ] **Step 7: 커밋**

```bash
git add src/core/config.rs src/systems/movement.rs src/entities/player.rs
git commit -m "feat: StageModifiers 리소스 + 우주선 미끄러움 연결

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 3: `reflect_or_wrap` 시스템 + 소행성 EdgeReflect 전환

**Files:**
- Modify: `src/systems/movement.rs`
- Modify: `src/entities/asteroid.rs`
- Test: `src/systems/movement.rs`

**Interfaces:**
- Consumes: `StageModifiers`(Task 2), `EdgeReflect`·`reflect_edge`(Task 1).
- Produces: `reflect_or_wrap` 시스템. 소행성은 `EdgeReflect`를 가진다.

- [ ] **Step 1: 실패하는 테스트 작성**

`src/systems/movement.rs`의 `mod tests`에 추가:

```rust
#[test]
fn edge_reflect_entity_wraps_when_bounce_off() {
    let mut app = App::new();
    app.insert_resource(StageModifiers { wall_bounce: false, ship_damping: 0.985 });
    let e = app
        .world_mut()
        .spawn((
            Transform::from_xyz(HALF_WIDTH + 50.0, 0.0, 0.0),
            Velocity(Vec2::new(100.0, 0.0)),
            crate::core::components::EdgeReflect,
        ))
        .id();
    app.world_mut().run_system_once(reflect_or_wrap).unwrap();
    let t = app.world().entity(e).get::<Transform>().unwrap();
    assert!(t.translation.x < 0.0); // 순환(반대편)
}

#[test]
fn edge_reflect_entity_bounces_when_on() {
    let mut app = App::new();
    app.insert_resource(StageModifiers { wall_bounce: true, ship_damping: 0.985 });
    let e = app
        .world_mut()
        .spawn((
            Transform::from_xyz(HALF_WIDTH + 50.0, 0.0, 0.0),
            Velocity(Vec2::new(100.0, 0.0)),
            crate::core::components::EdgeReflect,
        ))
        .id();
    app.world_mut().run_system_once(reflect_or_wrap).unwrap();
    let t = app.world().entity(e).get::<Transform>().unwrap();
    let v = app.world().entity(e).get::<Velocity>().unwrap();
    assert!((t.translation.x - HALF_WIDTH).abs() < 1e-3); // 경계로 클램프
    assert!(v.0.x < 0.0); // 반사
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test 2>&1 | tail -20`
Expected: `reflect_or_wrap` 미정의로 컴파일 에러.

- [ ] **Step 3: `reflect_or_wrap` 구현 + `wrap_around` 제외 + 스케줄 등록**

`src/systems/movement.rs`:

import 갱신(컴포넌트·로직):

```rust
use crate::core::components::{AngularVelocity, EdgeReflect, Velocity, Wrapping};
use crate::core::logic::{reflect_edge, wrap_position};
```

`wrap_around`에 `Without<EdgeReflect>` 추가:

```rust
fn wrap_around(mut query: Query<&mut Transform, (With<Wrapping>, Without<EdgeReflect>)>) {
```

`wrap_around` 아래에 신규 시스템 추가:

```rust
/// EdgeReflect 대상의 경계 처리. wall_bounce가 켜지면 반사, 아니면 순환(기존과 동일).
fn reflect_or_wrap(
    mods: Res<StageModifiers>,
    mut query: Query<(&mut Transform, &mut Velocity), With<EdgeReflect>>,
) {
    let half = Vec2::new(HALF_WIDTH, HALF_HEIGHT);
    for (mut transform, mut velocity) in &mut query {
        let pos = transform.translation.truncate();
        if mods.wall_bounce {
            let (p, v) = reflect_edge(pos, velocity.0, half);
            transform.translation.x = p.x;
            transform.translation.y = p.y;
            velocity.0 = v;
        } else {
            let w = wrap_position(pos, half);
            transform.translation.x = w.x;
            transform.translation.y = w.y;
        }
    }
}
```

스케줄에 체이닝(FixedUpdate):

```rust
        app.init_resource::<StageModifiers>()
            .add_systems(
                FixedUpdate,
                ((apply_velocity, wrap_around, reflect_or_wrap).chain(), apply_spin),
            );
```

- [ ] **Step 4: 소행성 스폰을 EdgeReflect로 전환**

`src/entities/asteroid.rs`:

import에서 `Wrapping`→`EdgeReflect`:

```rust
use crate::core::components::{AngularVelocity, Collider, EdgeReflect, Velocity};
```

`spawn_asteroid`의 번들에서 `Wrapping,`을 `EdgeReflect,`로 교체:

```rust
        Collider { radius: size.radius() },
        EdgeReflect,
        GameplayEntity,
```

- [ ] **Step 5: 테스트·클리피 통과 확인**

Run: `cargo test 2>&1 | tail -5 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과, 경고 0. (기본값 wall_bounce=false → 소행성은 기존처럼 순환.)

- [ ] **Step 6: 커밋**

```bash
git add src/systems/movement.rs src/entities/asteroid.rs
git commit -m "feat: reflect_or_wrap 경계 처리 + 소행성 EdgeReflect 전환

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 4: 얼음 아트 에셋 + SpriteAssets 필드

**Files:**
- Create: `assets/sprites/src/bg_ice.svg`
- Create: `assets/sprites/src/boss_ice_golem.svg`
- Modify: `src/fx/sprites.rs`

**Interfaces:**
- Produces: `SpriteAssets.bg_ice`, `SpriteAssets.boss_ice_golem`(Handle<Image>).

- [ ] **Step 1: 배경 SVG 작성**

`assets/sprites/src/bg_ice.svg` 생성(기존 `bg_*.svg`와 동일한 viewBox 320×180 가정, 창백한 청/백 얼음 톤):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180">
  <rect width="320" height="180" fill="#0b2233"/>
  <rect width="320" height="180" fill="#12405c" opacity="0.35"/>
  <g stroke="#bfe6ff" stroke-width="1.2" opacity="0.5" fill="none">
    <path d="M20 40 L60 70 L40 110"/>
    <path d="M250 30 L280 60 L300 100"/>
    <path d="M120 20 L150 50 L130 90"/>
  </g>
  <g fill="#eaf7ff" opacity="0.9">
    <circle cx="40" cy="30" r="1.5"/>
    <circle cx="90" cy="120" r="1.2"/>
    <circle cx="200" cy="60" r="1.6"/>
    <circle cx="270" cy="140" r="1.3"/>
    <circle cx="150" cy="150" r="1.1"/>
  </g>
  <g fill="#7fc7e8" opacity="0.25">
    <polygon points="0,180 80,140 160,180"/>
    <polygon points="180,180 250,150 320,180"/>
  </g>
</svg>
```

> viewBox 크기는 기존 `assets/sprites/src/bg_belt.svg`를 열어 실제 값에 맞춘다(다르면 그 값으로 대체). 배경은 `custom_size`로 창 크기에 늘려 그려지므로 종횡비만 대략 맞으면 된다.

- [ ] **Step 2: 보스 SVG 작성**

`assets/sprites/src/boss_ice_golem.svg` 생성(정사각 viewBox, 카툰: 플랫 채색 + 두꺼운 어두운 외곽선 + 하이라이트 1톤):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <g stroke="#0a2a3a" stroke-width="4" stroke-linejoin="round">
    <polygon points="50,8 74,26 78,60 60,90 40,90 22,60 26,26" fill="#5fb6d9"/>
    <polygon points="50,20 64,32 66,58 54,74 46,74 34,58 36,32" fill="#8fd6ef"/>
    <polygon points="43,44 57,44 53,58 47,58" fill="#0a2a3a" stroke="none"/>
  </g>
  <g fill="#eaffff" opacity="0.85" stroke="none">
    <polygon points="30,24 38,30 32,38"/>
    <polygon points="68,30 74,36 66,42"/>
  </g>
</svg>
```

> 정사각 viewBox면 `boss_radius(IceGolem)*2` 크기로 그려질 때 왜곡이 없다. 기존 `boss_blazing_core.svg`의 viewBox를 참고해 동일 규격을 쓴다.

- [ ] **Step 3: `SpriteAssets`에 필드 추가**

`src/fx/sprites.rs`:

구조체에 `bg_flare` 아래 추가:

```rust
    pub bg_flare: Handle<Image>,
    pub bg_ice: Handle<Image>,
    pub boss_ice_golem: Handle<Image>,
```

`build_sprite_assets`의 `bg_flare` 로드 아래 추가:

```rust
        bg_flare: asset_server.load("sprites/bg_flare.png"),
        bg_ice: asset_server.load("sprites/bg_ice.png"),
        boss_ice_golem: asset_server.load("sprites/boss_ice_golem.png"),
```

`dummy_sprite_assets`의 `bg_flare` 아래 추가:

```rust
        bg_flare: h.clone(),
        bg_ice: h.clone(),
        boss_ice_golem: h.clone(),
```

- [ ] **Step 4: 빌드·래스터화 확인**

Run: `cargo build 2>&1 | tail -5 && ls assets/sprites/bg_ice.png assets/sprites/boss_ice_golem.png`
Expected: 빌드 성공, 두 PNG 생성됨(build.rs가 src의 SVG를 자동 래스터화).

- [ ] **Step 5: 테스트·클리피 확인**

Run: `cargo test 2>&1 | tail -5 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과, 경고 0.

- [ ] **Step 6: 커밋**

```bash
git add assets/sprites/src/bg_ice.svg assets/sprites/src/boss_ice_golem.svg src/fx/sprites.rs
git commit -m "feat: 얼음 배경/보스 SVG 아트 + SpriteAssets 핸들 추가

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 5: 얼음 테마 + 골렘 turn-on (플레이 가능)

이 태스크는 `ThemeId::FrozenField`와 `BossKind::IceGolem` 두 변형을 함께 도입한다(Rust exhaustive match 때문에 분리 불가). 완료 시 얼음 스테이지가 실제로 등장하고, 벽 반사·미끄러움이 켜지며, 탱키한 골렘이 방사 탄을 쏜다. 정교한 공격/페이즈는 Task 6·7에서.

**Files:**
- Modify: `src/core/config.rs`
- Modify: `src/systems/stage.rs`
- Modify: `src/entities/boss.rs`
- Modify: `src/fx/background.rs`

**Interfaces:**
- Consumes: `StageModifiers`·`SHIP_DAMPING_ICE`(Task 2), `bg_ice`·`boss_ice_golem`(Task 4).
- Produces: `ThemeId::FrozenField`, `BossKind::IceGolem`, `Boss.phase: u8`, `stage_modifiers(theme)`, `sync_stage_modifiers` 시스템.

- [ ] **Step 1: 골렘 상수 추가**

`src/core/config.rs`의 Phase 5 블록(파일 끝) 아래에 추가:

```rust
// Phase 6 — 얼음 골렘
pub const ICE_GOLEM_HEALTH_MUL: f32 = 1.4; // 느린 대신 높은 체력
pub const GOLEM_DRIFT_SPEED: f32 = 55.0;   // 느린 드리프트 속도(u/s)
pub const GOLEM_ATTACK_INTERVAL: f32 = 2.5;
```

- [ ] **Step 2: 실패 테스트 작성(stage) — 테마 파라미터/수정자**

`src/systems/stage.rs`의 `mod tests`에 추가:

```rust
#[test]
fn frozen_field_params_and_modifiers() {
    let p = theme_params(ThemeId::FrozenField);
    assert!(p.count_mul < 1.0 && p.speed_mul < 1.0);
    let m = stage_modifiers(ThemeId::FrozenField);
    assert!(m.wall_bounce);
    assert!(m.ship_damping < crate::core::config::SHIP_DAMPING);
    // 비얼음은 기본값
    let d = stage_modifiers(ThemeId::AsteroidBelt);
    assert!(!d.wall_bounce);
}
```

- [ ] **Step 3: stage.rs — 변형·파라미터·수정자·동기화 구현**

`src/systems/stage.rs`:

import 추가(상단 `use` 블록):

```rust
use crate::core::config::{CYCLE_LEN, SHIP_DAMPING_ICE, STARTING_LIVES, WAVES_PER_STAGE};
use crate::systems::movement::StageModifiers;
```

`ThemeId`에 변형 추가:

```rust
pub enum ThemeId {
    AsteroidBelt,
    AlienFleet,
    SolarFlare,
    FrozenField,
}
```

`THEME_POOL` 확장(길이 4):

```rust
pub const THEME_POOL: [ThemeId; 4] = [
    ThemeId::AsteroidBelt,
    ThemeId::AlienFleet,
    ThemeId::SolarFlare,
    ThemeId::FrozenField,
];
```

`theme_params` match에 arm 추가(닫는 `}` 앞):

```rust
        ThemeId::FrozenField => ThemeParams { count_mul: 0.9, speed_mul: 0.95, ufo_interval_mul: 1.0 },
```

`theme_name` match에 arm 추가:

```rust
        ThemeId::FrozenField => "FROZEN FIELD",
```

`theme_name` 함수 아래에 신규 순수 함수 + 동기화 시스템 추가:

```rust
/// 테마별 물리 트위스트. 얼음만 반사+미끄럼, 그 외는 기본값.
pub fn stage_modifiers(theme: ThemeId) -> StageModifiers {
    match theme {
        ThemeId::FrozenField => StageModifiers { wall_bounce: true, ship_damping: SHIP_DAMPING_ICE },
        _ => StageModifiers::default(),
    }
}

/// 배경 동기화와 동일하게, 매 프레임 현재 테마로 StageModifiers를 맞춘다(상태 비의존).
fn sync_stage_modifiers(prog: Res<Progression>, mut mods: ResMut<StageModifiers>) {
    let want = stage_modifiers(prog.current_theme());
    mods.wall_bounce = want.wall_bounce;
    mods.ship_damping = want.ship_damping;
}
```

`StagePlugin::build`의 `Update` 시스템에 `sync_stage_modifiers` 추가:

```rust
        .add_systems(
            Update,
            (stage_control, sync_stage_modifiers).run_if(in_state(GameState::Playing)),
        );
```

> `use crate::core::config::{CYCLE_LEN, STARTING_LIVES, WAVES_PER_STAGE};`가 이미 있으면 `SHIP_DAMPING_ICE`만 그 목록에 추가하면 된다(중복 import 주의).

- [ ] **Step 4: 실패 테스트 작성(boss) — 골렘 데이터**

`src/entities/boss.rs`의 `mod tests`에 추가:

```rust
#[test]
fn ice_golem_is_frozen_field_boss_and_tanky() {
    assert_eq!(boss_for_theme(ThemeId::FrozenField), BossKind::IceGolem);
    // 같은 사이클에서 골렘이 모함(기준)보다 체력이 높다.
    assert!(boss_max_health(BossKind::IceGolem, 0) > boss_max_health(BossKind::Mothership, 0));
}
```

- [ ] **Step 5: boss.rs — IceGolem 변형 + 모든 arm 구현**

`src/entities/boss.rs`:

import에 `ICE_GOLEM_HEALTH_MUL`, `GOLEM_ATTACK_INTERVAL`, `GOLEM_DRIFT_SPEED` 및 컴포넌트 추가:

```rust
use crate::core::components::{Collider, EdgeReflect, Velocity};
```

config import 블록에 상수 추가:

```rust
    BOSS_SCORE_BONUS, BULLET_BOSS_DAMAGE, EXPLOSION_PARTICLES, GOLEM_ATTACK_INTERVAL,
    GOLEM_DRIFT_SPEED, ICE_GOLEM_HEALTH_MUL, SHAKE_EXPLOSION, UFO_BULLET_SPEED, Z_ENTITY,
```

`BossKind`에 변형 추가:

```rust
pub enum BossKind {
    MotherRock,
    Mothership,
    BlazingCore,
    IceGolem,
}
```

`boss_for_theme` match에 arm 추가:

```rust
        ThemeId::SolarFlare => BossKind::BlazingCore,
        ThemeId::FrozenField => BossKind::IceGolem,
```

`Boss` 구조체에 `phase` 필드 추가:

```rust
#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
    pub phase: u8,
}
```

`boss_max_health` match에 arm 추가:

```rust
        BossKind::BlazingCore => BOSS_BASE_HEALTH * 1.2,
        BossKind::IceGolem => BOSS_BASE_HEALTH * ICE_GOLEM_HEALTH_MUL,
```

`boss_image` match에 arm 추가:

```rust
        BossKind::BlazingCore => assets.boss_blazing_core.clone(),
        BossKind::IceGolem => assets.boss_ice_golem.clone(),
```

`boss_radius` match에 arm 추가:

```rust
        BossKind::BlazingCore => 45.0,
        BossKind::IceGolem => 50.0,
```

`attack_interval` match에 arm 추가:

```rust
        BossKind::BlazingCore => 2.4,
        BossKind::IceGolem => GOLEM_ATTACK_INTERVAL,
```

`spawn_boss` 교체(phase 필드 + 골렘 전용 Velocity·EdgeReflect 삽입):

```rust
pub fn spawn_boss(commands: &mut Commands, assets: &SpriteAssets, kind: BossKind, cycle: u32) {
    let hp = boss_max_health(kind, cycle);
    let r = boss_radius(kind);
    let mut e = commands.spawn((
        Boss { kind, health: hp, max_health: hp, phase: 0 },
        BossAttack { timer: Timer::from_seconds(attack_interval(kind), TimerMode::Repeating) },
        Sprite {
            image: boss_image(kind, assets),
            custom_size: Some(Vec2::splat(r * 2.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 120.0, Z_ENTITY),
        Collider { radius: r },
        GameplayEntity,
    ));
    // 얼음 골렘: 속도 기반 드리프트 + 벽 반사(EdgeReflect). 이동은 apply_velocity+reflect_or_wrap가 처리.
    if kind == BossKind::IceGolem {
        e.insert((Velocity(Vec2::new(GOLEM_DRIFT_SPEED, GOLEM_DRIFT_SPEED * 0.55)), EdgeReflect));
    }
}
```

`boss_movement`의 match에 IceGolem arm 추가(velocity로 움직이므로 위치 직접 조작 없음):

```rust
            BossKind::BlazingCore => {
                tf.translation.x = (t * 1.2).sin() * 30.0;
                tf.translation.y = 150.0 + (t * 2.0).sin() * 15.0;
            }
            // 얼음 골렘: Velocity + reflect_or_wrap로 드리프트/반사(여기선 조작 없음).
            BossKind::IceGolem => {}
```

`boss_attack`의 match에 IceGolem arm 추가(Task 5는 최소 = 방사형 12발):

```rust
            // 얼음 골렘(기본): 방사형 얼음 탄(Task 6에서 팔 휘두르기/내려찍기로 확장).
            BossKind::IceGolem => {
                let n = 12;
                for i in 0..n {
                    let ang = i as f32 / n as f32 * std::f32::consts::TAU;
                    let d = Vec2::new(ang.cos(), ang.sin());
                    spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                }
            }
```

`damage_reduces_and_defeats` 테스트의 `Boss { ... }` 리터럴에 `phase: 0`을 추가(필드 누락 컴파일 에러 방지):

```rust
        let mut b = Boss { kind: BossKind::Mothership, health: 3.0, max_health: 10.0, phase: 0 };
```

- [ ] **Step 6: background.rs — 얼음 배경 분기**

`src/fx/background.rs`의 `theme_bg` match에 arm 추가:

```rust
        ThemeId::SolarFlare => assets.bg_flare.clone(),
        ThemeId::FrozenField => assets.bg_ice.clone(),
```

- [ ] **Step 7: 테스트·클리피 통과 확인**

Run: `cargo test 2>&1 | tail -8 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과(신규 테마/보스 테스트 포함), 경고 0. `pick_cycle_order`·`advance_stage` 등 기존 테스트는 풀 4에서도 통과.

- [ ] **Step 8: 커밋**

```bash
git add src/core/config.rs src/systems/stage.rs src/entities/boss.rs src/fx/background.rs
git commit -m "feat: 얼음 테마(Frozen Field) + 얼음 골렘 보스 도입

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 6: 골렘 공격 정교화 — 팔 휘두르기 + 내려찍기 교대

**Files:**
- Modify: `src/entities/boss.rs`
- Test: `src/entities/boss.rs`

**Interfaces:**
- Consumes: `BossKind::IceGolem`, `BossAttack`(Task 5).
- Produces: `BossAttack.shots: u32`, `golem_attack_is_sweep(shots) -> bool`.

- [ ] **Step 1: 실패 테스트 작성 — 공격 교대 판정**

`src/entities/boss.rs`의 `mod tests`에 추가:

```rust
#[test]
fn golem_alternates_sweep_and_slam() {
    assert!(golem_attack_is_sweep(0));  // 첫 발 = 팔 휘두르기
    assert!(!golem_attack_is_sweep(1)); // 다음 = 내려찍기
    assert!(golem_attack_is_sweep(2));
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test golem_alternates 2>&1 | tail -10`
Expected: `golem_attack_is_sweep` 미정의로 컴파일 에러.

- [ ] **Step 3: `BossAttack.shots` 필드 + 판정 함수 + 공격 arm 교체**

`src/entities/boss.rs`:

`BossAttack`에 `shots` 추가:

```rust
#[derive(Component)]
pub struct BossAttack {
    pub timer: Timer,
    pub shots: u32,
}
```

`spawn_boss`의 `BossAttack { timer: ... }`에 `shots: 0` 추가:

```rust
        BossAttack { timer: Timer::from_seconds(attack_interval(kind), TimerMode::Repeating), shots: 0 },
```

`attack_interval` 함수 아래에 판정 순수 함수 추가:

```rust
/// 골렘 공격 교대: 짝수 발 = 팔 휘두르기(부채꼴 파편), 홀수 발 = 내려찍기(방사 탄).
pub fn golem_attack_is_sweep(shots: u32) -> bool {
    shots % 2 == 0
}
```

`boss_attack`에서 각 보스 발사 시 `atk.shots`를 증가시키도록, 타이머 발사 직후 라인을 추가한다. `let pos = tf.translation.truncate();` **바로 위**에 삽입:

```rust
        let shots = atk.shots;
        atk.shots = atk.shots.wrapping_add(1);
```

IceGolem arm을 교대 공격으로 교체:

```rust
            // 얼음 골렘: 팔 휘두르기(부채꼴 파편) ↔ 내려찍기(방사 탄) 교대.
            BossKind::IceGolem => {
                if golem_attack_is_sweep(shots) {
                    // 팔 휘두르기: 조준 방향 ±35° 부채꼴로 소형 소행성 파편 5개(벽 반사).
                    let base = player_pos
                        .map(|pp| aim_direction(pos, pp))
                        .unwrap_or(Vec2::NEG_Y);
                    for a in [-0.61f32, -0.305, 0.0, 0.305, 0.61] {
                        let d = (Quat::from_rotation_z(a) * base.extend(0.0)).truncate();
                        let size = AsteroidSize::Small;
                        spawn_asteroid(&mut commands, &assets, size, pos, d * random_velocity(size).length());
                    }
                } else {
                    // 내려찍기: 방사형 얼음 탄 12발.
                    let n = 12;
                    for i in 0..n {
                        let ang = i as f32 / n as f32 * std::f32::consts::TAU;
                        let d = Vec2::new(ang.cos(), ang.sin());
                        spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                    }
                }
            }
```

> `random_velocity(size).length()`는 파편 속력만 취해 방향은 부채꼴로 지정한다. `AsteroidSize`·`random_velocity`·`aim_direction`은 이미 import되어 있다(파일 상단 확인, 없으면 추가).

- [ ] **Step 4: 테스트·클리피 통과 확인**

Run: `cargo test 2>&1 | tail -6 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과, 경고 0.

- [ ] **Step 5: 커밋**

```bash
git add src/entities/boss.rs
git commit -m "feat: 얼음 골렘 공격 정교화(팔 휘두르기/내려찍기 교대)

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 7: 골렘 다단계 페이즈 — 껍질 깨짐(축소·가속)

**Files:**
- Modify: `src/entities/boss.rs`
- Test: `src/entities/boss.rs`

**Interfaces:**
- Consumes: `Boss.phase`(Task 5), `BossAttack`(Task 6).
- Produces: `golem_phase(health, max) -> u8`, `golem_scale/golem_speed_mul/golem_attack_mul(phase) -> f32`, `golem_phase_transition` 시스템.

- [ ] **Step 1: 실패 테스트 작성 — 페이즈 순수 함수**

`src/entities/boss.rs`의 `mod tests`에 추가:

```rust
#[test]
fn golem_phase_thresholds() {
    let m = 30.0;
    assert_eq!(golem_phase(30.0, m), 0); // 만피
    assert_eq!(golem_phase(20.1, m), 0); // >2/3
    assert_eq!(golem_phase(15.0, m), 1); // 1/3~2/3
    assert_eq!(golem_phase(5.0, m), 2);  // <1/3
}

#[test]
fn golem_phase_scales_monotonic() {
    assert!(golem_scale(2) < golem_scale(0));       // 작아짐
    assert!(golem_speed_mul(2) > golem_speed_mul(0)); // 빨라짐
    assert!(golem_attack_mul(2) < golem_attack_mul(0)); // 주기 짧아짐
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test golem_phase 2>&1 | tail -10`
Expected: 미정의 함수로 컴파일 에러.

- [ ] **Step 3: 페이즈 순수 함수 + 전이 시스템 구현**

`src/entities/boss.rs`:

import에 `Duration`·효과 함수가 필요하다. 상단에 추가:

```rust
use std::time::Duration;
```

(그리고 이미 import된 `spawn_explosion`, `ShakeEvent`, `EXPLOSION_PARTICLES`, `SHAKE_EXPLOSION`를 재사용한다.)

`golem_attack_is_sweep` 아래에 순수 함수 추가:

```rust
/// 체력 비율로 골렘 페이즈 산출: >2/3 → 0, >1/3 → 1, 그 이하 → 2.
pub fn golem_phase(health: f32, max_health: f32) -> u8 {
    let r = if max_health > 0.0 { health / max_health } else { 0.0 };
    if r > 2.0 / 3.0 {
        0
    } else if r > 1.0 / 3.0 {
        1
    } else {
        2
    }
}

pub fn golem_scale(phase: u8) -> f32 {
    [1.0, 0.8, 0.62][phase.min(2) as usize]
}

pub fn golem_speed_mul(phase: u8) -> f32 {
    [1.0, 1.35, 1.75][phase.min(2) as usize]
}

pub fn golem_attack_mul(phase: u8) -> f32 {
    [1.0, 0.8, 0.62][phase.min(2) as usize]
}
```

`BossPlugin::build`의 Update 시스템 튜플에 `golem_phase_transition` 추가:

```rust
        app.add_systems(
            Update,
            (boss_movement, boss_attack, boss_combat, golem_phase_transition)
                .run_if(in_state(GameState::Playing)),
        );
```

`boss_combat` 아래에 전이 시스템 추가:

```rust
/// 골렘이 체력 구간을 넘으면 껍질이 깨진다: 스프라이트/콜라이더 축소, 가속, 공격 주기 단축, 파편 폭발 연출.
#[allow(clippy::type_complexity)]
fn golem_phase_transition(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut shake: MessageWriter<ShakeEvent>,
    mut q: Query<(&mut Boss, &mut Sprite, &mut Collider, &mut Velocity, &mut BossAttack, &Transform)>,
) {
    for (mut boss, mut sprite, mut collider, mut velocity, mut atk, tf) in &mut q {
        if boss.kind != BossKind::IceGolem {
            continue;
        }
        let want = golem_phase(boss.health, boss.max_health);
        if want <= boss.phase {
            continue;
        }
        boss.phase = want;
        let base_r = boss_radius(BossKind::IceGolem);
        let scale = golem_scale(want);
        sprite.custom_size = Some(Vec2::splat(base_r * 2.0 * scale));
        collider.radius = base_r * scale;
        // 가속: 방향 유지, 속력만 페이즈 배율로.
        let dir = velocity.0.normalize_or_zero();
        velocity.0 = dir * GOLEM_DRIFT_SPEED * golem_speed_mul(want);
        // 공격 주기 단축.
        let interval = attack_interval(BossKind::IceGolem) * golem_attack_mul(want);
        atk.timer.set_duration(Duration::from_secs_f32(interval));
        // 껍질 깨짐 연출.
        let pos = tf.translation.truncate();
        spawn_explosion(&mut commands, &assets, pos, EXPLOSION_PARTICLES);
        shake.write(ShakeEvent(SHAKE_EXPLOSION * 1.5));
    }
}
```

- [ ] **Step 4: 테스트·클리피 통과 확인**

Run: `cargo test 2>&1 | tail -8 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과, 경고 0.

- [ ] **Step 5: 실기 확인(선택, 사람 검증)**

Run: `cargo run` — 얼음 스테이지 진입 시 배경/미끄러움/소행성 반사, 골렘 등장·공격·껍질 깨짐 확인 후 종료. (렌더/체감은 사람이 확인.)

- [ ] **Step 6: 커밋**

```bash
git add src/entities/boss.rs
git commit -m "feat: 얼음 골렘 다단계 페이즈(껍질 깨짐 → 축소·가속)

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Self-Review (작성자 체크)

- **스펙 커버리지**: 트위스트 인프라(T1–T3) / 미끄러움(T2) / 벽 반사(T1,T3) / 진행 통합·풀 4(T5) / 골렘 이동·공격(T5,T6)·다단계(T7) / 아트(T4) / 배경(T5) — 스펙 §3 전 항목 태스크 존재. ✓
- **플레이스홀더**: 모든 스텝에 실제 코드/명령/기대값 기재. 상수는 Global Constraints에 확정. ✓
- **타입 일관성**: `Boss.phase`는 T5에서 추가되고 T7에서 사용. `BossAttack.shots`는 T6에서 추가·사용. `StageModifiers`는 T2 정의, T3·T5에서 사용. `EdgeReflect`/`reflect_edge`는 T1 정의, T3에서 사용. 시그니처 일관. ✓
- **컴파일/무경고 유지**: 두 enum 변형(FrozenField·IceGolem)은 T5에서 함께 도입되어 즉시 생성처(boss_for_theme)를 가지므로 dead_code 경고 없음. T1–T4는 변형 없이 독립 완결. ✓
- **주의**: T6에서 `AsteroidSize`/`random_velocity`/`aim_direction` import 존재 확인(boss.rs 상단에 이미 있음 — 없으면 추가). T5에서 config import 중복 방지.
