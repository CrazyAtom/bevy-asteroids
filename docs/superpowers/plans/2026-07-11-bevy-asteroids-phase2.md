# Bevy Asteroids Phase 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Phase 1 애스토로이드 게임에 브레이크·소행성 속도/회전·우주선 색/화염·폭발/피격 이펙트·적 UFO 2종·웨이브 난이도·최고점수 저장을 추가한다.

**Architecture:** 기존 플러그인 모듈 구조를 확장한다. 신규 `ufo.rs`(UfoPlugin)·`effects.rs`(EffectsPlugin)를 추가하고 나머지는 기존 파일에 시스템/컴포넌트를 더한다. 순수 계산 로직(브레이크·크기별 속도·난이도 공식·조준·최고점수 갱신)은 `logic.rs`에 넣어 단위 테스트한다. 렌더는 계속 Gizmos.

**Tech Stack:** Rust 2021 · Bevy `0.19` · rand `0.10` · bevy-persistent `0.11`(json) · serde `1`(derive) · dirs

## Global Constraints

- Bevy `0.19`, rand `0.10`. Phase 1에서 검증한 API를 그대로 재사용한다: `Res<ButtonInput<KeyCode>>`(`.pressed`/`.just_pressed`), `Res<Time>`(`time.delta_secs()`, `time.delta()`, `time.elapsed_secs()`), `transform.rotate_z`/`rotation * Vec3::Y`/`transform_point`, States(`OnEnter`/`OnExit`/`run_if(in_state)`/`NextState.set`), Gizmos(`linestrip_2d`, `circle_2d(Isometry2d::from_translation(pos), r, color)`), `commands.entity(e).despawn()`, `Query::single() -> Result`, `Timer`(`.tick`, `.is_finished()`, `TimerMode`), `rand::rng()` + `use rand::RngExt` + `random_range`, 테스트는 `use bevy::ecs::system::RunSystemOnce;` + `world.run_system_once(sys).unwrap()`, `Time::<()>::default()` + `advance_by`, 상태 테스트엔 `app.add_plugins(bevy::state::app::StatesPlugin)` 후 `init_state`.
- **렌더는 Gizmos만.** Mesh/Material/스프라이트 금지.
- **`apply_velocity`(기존, FixedUpdate)는 `Velocity`를 가진 모든 엔티티를 이동시킨다. `wrap_around`(기존)는 `Wrapping` 마커가 있는 엔티티만 순환시킨다.** Phase 2에서 추가하는 UFO·적총알·파티클은 `Velocity`를 갖되 `Wrapping`은 갖지 않는다(순환하지 않고 수명/화면이탈로 소멸).
- 한 판(Playing) 동안 생성되는 모든 엔티티(UFO·적총알·파티클 포함)는 `GameplayEntity` 마커를 붙인다.
- `Color::srgb(r,g,b)`가 0.19에서 `const fn`이 아니면, 색 상수를 `const` 대신 반환 함수(`fn ship_color() -> Color`)로 바꾼다(구현 시 컴파일러 기준으로 판단, 의미는 동일).
- bevy-persistent: `Persistent::<T>::builder().name(..).format(StorageFormat::Json).path(..).default(..).build()`, `T: Resource + Serialize + Deserialize + Default`, 읽기 Deref, 저장 `.persist()`/`.set()`. `dirs::config_dir()`로 경로 구성.
- 커밋 메시지는 한국어.

---

## File Structure

```
src/
├── ufo.rs        (신규) Ufo/UfoSize, UfoSpawnTimer, 스폰·이동·사격, UfoPlugin
├── effects.rs    (신규) Particle, spawn_explosion, 수명·렌더, EffectsPlugin
├── config.rs     (+) 브레이크/색/화염/각속도/이펙트/UFO/난이도 상수
├── logic.rs      (+) apply_brake, AsteroidSize::speed_scale, 난이도 공식 4종, aim_direction, update_high_score
├── components.rs (+) AngularVelocity
├── movement.rs   (+) apply_spin (FixedUpdate)
├── player.rs     (+) EngineState, 브레이크, 색/화염 렌더
├── asteroid.rs   (+) 크기별 속도, 각속도 부여, 난이도 연동 웨이브
├── bullet.rs     (+) EnemyBullet, 수명·렌더
├── collision.rs  (+) 총알↔UFO, 적총알↔플레이어, UFO↔플레이어, 폭발/피격 이펙트
├── state.rs      (+) Wave 리소스, HighScore(Persistent)
├── ui.rs         (+) HUD·게임오버에 최고점수 표시
└── main.rs       (+) UfoPlugin/EffectsPlugin 등록, HighScore 초기화
```

---

## Task 1: 소행성 크기별 속도

**Files:** Modify `src/logic.rs`, `src/config.rs`, `src/asteroid.rs`

**Interfaces:**
- Produces: `AsteroidSize::speed_scale(self) -> f32` (Large=1.0, Medium=1.5, Small=2.2). `asteroid.rs::random_velocity(size: AsteroidSize) -> Vec2` 가 크기 배수를 곱한다.

- [ ] **Step 1: 실패 테스트 — logic.rs tests에 추가**

```rust
    #[test]
    fn smaller_asteroids_are_faster() {
        assert!(AsteroidSize::Small.speed_scale() > AsteroidSize::Medium.speed_scale());
        assert!(AsteroidSize::Medium.speed_scale() > AsteroidSize::Large.speed_scale());
        assert_eq!(AsteroidSize::Large.speed_scale(), 1.0);
    }
```

- [ ] **Step 2: 실패 확인** — Run: `cargo test smaller_asteroids_are_faster` → FAIL (`speed_scale` 미정의)

- [ ] **Step 3: 구현 — logic.rs `impl AsteroidSize`에 메서드 추가**

```rust
    pub fn speed_scale(self) -> f32 {
        match self {
            AsteroidSize::Large => 1.0,
            AsteroidSize::Medium => 1.5,
            AsteroidSize::Small => 2.2,
        }
    }
```

- [ ] **Step 4: asteroid.rs — `random_velocity`가 크기 배수 반영**

`random_velocity`를 다음으로 교체(크기 인자 추가). `spawn_wave`/`spawn_initial_wave`의 호출부는 `AsteroidSize::Large`를 넘기고, 분열 자식은 Task 이후 collision에서 크기별로 호출된다. 현재 collision의 고정 분열 속도는 Task 없이 남지만 Task 10에서 정리 — 여기서는 `random_velocity` 시그니처만 확장:

```rust
fn random_velocity(size: AsteroidSize) -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED) * size.speed_scale();
    Vec2::new(angle.cos(), angle.sin()) * speed
}
```

`spawn_wave` 내부 호출을 `random_velocity(AsteroidSize::Large)` 로 수정. `use crate::logic::AsteroidSize;` 는 이미 존재.

- [ ] **Step 5: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 6: 커밋**

```bash
git add src/logic.rs src/config.rs src/asteroid.rs
git commit -m "feat: 소행성 크기별 속도(작을수록 빠름)"
```

---

## Task 2: 소행성 회전

**Files:** Modify `src/components.rs`, `src/movement.rs`, `src/asteroid.rs`, `src/config.rs`

**Interfaces:**
- Produces: `components::AngularVelocity(pub f32)` (Component). `movement::apply_spin` 시스템(FixedUpdate)이 `AngularVelocity`를 가진 엔티티를 회전. 소행성 스폰 시 랜덤 각속도 부여.
- Consumes: `config::ASTEROID_SPIN_MAX`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
pub const ASTEROID_SPIN_MAX: f32 = 1.5; // rad/s (회전 각속도 범위 ±)
```

- [ ] **Step 2: components.rs에 컴포넌트 추가**

```rust
#[derive(Component, Clone, Copy)]
pub struct AngularVelocity(pub f32);
```

- [ ] **Step 3: 실패 테스트 — movement.rs tests에 추가**

```rust
    #[test]
    fn spin_rotates_entity() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);
        let e = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), AngularVelocity(2.0)))
            .id();
        app.world_mut().run_system_once(apply_spin).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        let (_, angle) = t.rotation.to_axis_angle();
        assert!(angle > 0.0); // 회전 발생
    }
```

`movement.rs` 상단 `use` 에 `AngularVelocity` 포함되도록 `use crate::components::{AngularVelocity, Velocity, Wrapping};` 로 수정.

- [ ] **Step 4: 실패 확인** — Run: `cargo test spin_rotates_entity` → FAIL (`apply_spin` 미정의)

- [ ] **Step 5: 구현 — movement.rs에 시스템 추가 + 플러그인 등록**

```rust
fn apply_spin(time: Res<Time>, mut query: Query<(&mut Transform, &AngularVelocity)>) {
    let dt = time.delta_secs();
    for (mut transform, angular) in &mut query {
        transform.rotate_z(angular.0 * dt);
    }
}
```

`MovementPlugin::build` 를 다음으로 수정:

```rust
        app.add_systems(FixedUpdate, ((apply_velocity, wrap_around).chain(), apply_spin));
```

(위치 파이프라인 `apply_velocity → wrap_around` 는 `.chain()` 으로 순서를 반드시 유지. `apply_spin` 은 회전만 다뤄 위치와 무관하므로 체인 밖 독립 시스템으로 둔다.)

- [ ] **Step 6: asteroid.rs — 스폰 시 랜덤 각속도 부여**

`spawn_asteroid` 의 스폰 튜플에 `AngularVelocity` 추가. 상단 `use` 에 `use crate::components::{AngularVelocity, Collider, Velocity, Wrapping};` 및 `use crate::config::ASTEROID_SPIN_MAX;` 추가. 함수 내 스폰 직전 각속도 계산:

```rust
pub fn spawn_asteroid(
    commands: &mut Commands,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
) {
    let mut rng = rand::rng();
    let spin = rng.random_range(-ASTEROID_SPIN_MAX..ASTEROID_SPIN_MAX);
    commands.spawn((
        Asteroid { size },
        AsteroidShape { points: asteroid_shape(size.radius()) },
        Transform::from_translation(position.extend(0.0)),
        Velocity(velocity),
        AngularVelocity(spin),
        Collider { radius: size.radius() },
        Wrapping,
        GameplayEntity,
    ));
}
```

- [ ] **Step 7: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공. (수동: `cargo run` 은 실행자가 아닌 사용자 몫)

- [ ] **Step 8: 커밋**

```bash
git add src/components.rs src/movement.rs src/asteroid.rs src/config.rs
git commit -m "feat: 소행성 회전(각속도) 추가"
```

---

## Task 3: 브레이크(↓ 급감속)

**Files:** Modify `src/logic.rs`, `src/config.rs`, `src/player.rs`

**Interfaces:**
- Produces: `logic::apply_brake(velocity: Vec2, rate: f32, dt: f32) -> Vec2` (0쪽으로 lerp). `player_input` 이 ↓ 입력 시 이를 적용.
- Consumes: `config::SHIP_BRAKE_RATE`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
pub const SHIP_BRAKE_RATE: f32 = 3.0; // 브레이크 감쇠율(1/s)
```

- [ ] **Step 2: 실패 테스트 — logic.rs tests에 추가**

```rust
    #[test]
    fn brake_reduces_speed_toward_zero() {
        let v = Vec2::new(100.0, 0.0);
        let braked = apply_brake(v, 3.0, 0.1);
        assert!(braked.length() < v.length());
        assert!(braked.x > 0.0); // 방향 유지, 아직 0 아님
    }

    #[test]
    fn brake_never_overshoots_past_zero() {
        let v = Vec2::new(10.0, 0.0);
        let braked = apply_brake(v, 3.0, 100.0); // 큰 dt
        assert!(braked.length() <= v.length());
        assert!(braked.x >= 0.0); // 반대로 튀지 않음
    }
```

- [ ] **Step 3: 실패 확인** — Run: `cargo test brake` → FAIL (`apply_brake` 미정의)

- [ ] **Step 4: 구현 — logic.rs에 함수 추가**

```rust
/// 속도를 0쪽으로 감쇠(브레이크). t는 [0,1]로 클램프해 반대로 튀지 않게 한다.
pub fn apply_brake(velocity: Vec2, rate: f32, dt: f32) -> Vec2 {
    let t = (rate * dt).clamp(0.0, 1.0);
    velocity.lerp(Vec2::ZERO, t)
}
```

- [ ] **Step 5: player.rs — player_input에 ↓ 브레이크 반영**

`use crate::config::{...}` 에 `SHIP_BRAKE_RATE` 추가, `use crate::logic::apply_brake;` 추가. `player_input` 의 추진 블록 뒤에 추가:

```rust
        if keys.pressed(KeyCode::ArrowDown) {
            velocity.0 = apply_brake(velocity.0, SHIP_BRAKE_RATE, dt);
        }
```

- [ ] **Step 6: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 7: 커밋**

```bash
git add src/logic.rs src/config.rs src/player.rs
git commit -m "feat: 반대추진(↓) 브레이크 급감속"
```

---

## Task 4: 우주선 컬러 + 추진/역추진 화염

**Files:** Modify `src/config.rs`, `src/player.rs`

**Interfaces:**
- Produces: `player::EngineState { thrusting: bool, braking: bool }` (Component). `player_input` 이 매 프레임 갱신, `draw_player` 가 색/화염 렌더.
- Consumes: `config::SHIP_COLOR`, `config::FLAME_COLOR`.

- [ ] **Step 1: config.rs에 색 상수 추가**

```rust
pub const SHIP_COLOR: Color = Color::srgb(0.40, 0.90, 1.00);  // 청록
pub const FLAME_COLOR: Color = Color::srgb(1.00, 0.55, 0.15); // 주황
```

`config.rs` 상단에 `use bevy::prelude::Color;` 추가. (`Color::srgb` 가 const가 아니면 `pub fn ship_color() -> Color { Color::srgb(..) }` 형태로 바꾸고 사용처도 함수 호출로 변경.)

- [ ] **Step 2: 실패 테스트 — player.rs tests에 추가 (EngineState 갱신 검증)**

```rust
    #[test]
    fn thrust_key_sets_engine_state() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowUp);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((Player, Transform::default(), Velocity(Vec2::ZERO), EngineState::default()))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let s = app.world().entity(e).get::<EngineState>().unwrap();
        assert!(s.thrusting);
        assert!(!s.braking);
    }
```

- [ ] **Step 3: 실패 확인** — Run: `cargo test thrust_key_sets_engine_state` → FAIL (`EngineState` 미정의)

- [ ] **Step 4: 구현 — player.rs**

`use crate::config::{...}` 에 `SHIP_COLOR, FLAME_COLOR` 추가. 컴포넌트 정의 추가:

```rust
#[derive(Component, Default)]
pub struct EngineState {
    pub thrusting: bool,
    pub braking: bool,
}
```

`spawn_player_entity` 의 스폰 튜플에 `EngineState::default()` 추가.

`player_input` 의 시그니처에 `EngineState` 를 포함하도록 쿼리 변경 후, 매 프레임 상태를 갱신:

```rust
fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut EngineState), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity, mut engine) in &mut query {
        let mut turn = 0.0;
        if keys.pressed(KeyCode::ArrowLeft) { turn += 1.0; }
        if keys.pressed(KeyCode::ArrowRight) { turn -= 1.0; }
        transform.rotate_z(turn * SHIP_ROTATION_SPEED * dt);

        engine.thrusting = keys.pressed(KeyCode::ArrowUp);
        engine.braking = keys.pressed(KeyCode::ArrowDown);

        if engine.thrusting {
            let forward = (transform.rotation * Vec3::Y).truncate();
            velocity.0 += forward * SHIP_THRUST * dt;
        }
        if engine.braking {
            velocity.0 = apply_brake(velocity.0, SHIP_BRAKE_RATE, dt);
        }
    }
}
```

`draw_player` 를 색상 + 화염 렌더로 교체. 화염은 `EngineState` 를 읽어 추진 시 뒤쪽(-Y), 브레이크 시 앞쪽(+Y)에 삼각형을 그린다. 깜빡임은 `time.elapsed_secs()` 기반:

```rust
fn draw_player(
    mut gizmos: Gizmos,
    time: Res<Time>,
    query: Query<(&Transform, &EngineState), With<Player>>,
) {
    for (transform, engine) in &query {
        let points = SHIP_POINTS.map(|p| transform.transform_point(p.extend(0.0)).truncate());
        gizmos.linestrip_2d(points, SHIP_COLOR);

        // 깜빡임 계수(0.6~1.0)
        let flicker = 0.6 + 0.4 * (time.elapsed_secs() * 30.0).sin().abs();
        if engine.thrusting {
            let flame = [
                Vec2::new(-6.0, -12.0),
                Vec2::new(0.0, -12.0 - 10.0 * flicker),
                Vec2::new(6.0, -12.0),
            ]
            .map(|p| transform.transform_point(p.extend(0.0)).truncate());
            gizmos.linestrip_2d(flame, FLAME_COLOR);
        }
        if engine.braking {
            let flame = [
                Vec2::new(-4.0, 14.0),
                Vec2::new(0.0, 14.0 + 7.0 * flicker),
                Vec2::new(4.0, 14.0),
            ]
            .map(|p| transform.transform_point(p.extend(0.0)).truncate());
            gizmos.linestrip_2d(flame, FLAME_COLOR);
        }
    }
}
```

- [ ] **Step 5: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공. (수동: 우주선 색·추진/브레이크 화염은 사용자 `cargo run` 확인)

- [ ] **Step 6: 커밋**

```bash
git add src/config.rs src/player.rs
git commit -m "feat: 우주선 컬러와 추진/역추진 화염 표시"
```

---

## Task 5: 이펙트 시스템 (폭발 파티클)

**Files:** Create `src/effects.rs`; Modify `src/main.rs`, `src/config.rs`

**Interfaces:**
- Produces: `effects::Particle { life: Timer }` (Component). `effects::spawn_explosion(commands: &mut Commands, position: Vec2, count: usize)` (공용, collision에서 호출). `effects::EffectsPlugin`. 파티클은 `Velocity`(기존 `apply_velocity`로 이동) + `GameplayEntity` 보유, `Wrapping` 없음.
- Consumes: `config::{PARTICLE_LIFETIME_SECS, PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX}`, `components::Velocity`, `state::GameplayEntity`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
pub const EXPLOSION_PARTICLES: usize = 10;
pub const PARTICLE_LIFETIME_SECS: f32 = 0.6;
pub const PARTICLE_SPEED_MIN: f32 = 60.0;
pub const PARTICLE_SPEED_MAX: f32 = 200.0;
```

- [ ] **Step 2: 실패 테스트 — effects.rs 하단**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn spawn_explosion_creates_particles() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_explosion(&mut commands, Vec2::ZERO, 8);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Particle>();
        assert_eq!(q.iter(app.world()).count(), 8);
    }

    #[test]
    fn expired_particle_is_despawned() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(0.5, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(1.0));
        let e = app.world_mut().spawn(Particle { life: timer }).id();
        app.world_mut().run_system_once(particle_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
```

- [ ] **Step 3: main.rs에 `mod effects;` 추가 후 실패 확인** — Run: `cargo test` → FAIL (`Particle`/`spawn_explosion`/`particle_lifetime` 미정의)

- [ ] **Step 4: effects.rs 구현 (테스트 모듈 위)**

```rust
use bevy::prelude::*;
use rand::RngExt;

use crate::components::Velocity;
use crate::config::{PARTICLE_LIFETIME_SECS, PARTICLE_SPEED_MAX, PARTICLE_SPEED_MIN};
use crate::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Particle {
    pub life: Timer,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (particle_lifetime, draw_particles).run_if(in_state(GameState::Playing)),
        );
    }
}

/// 지정 위치에서 바깥 방향으로 흩어지는 단명 파티클을 count개 스폰한다.
pub fn spawn_explosion(commands: &mut Commands, position: Vec2, count: usize) {
    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED_MIN..PARTICLE_SPEED_MAX);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        commands.spawn((
            Particle { life: Timer::from_seconds(PARTICLE_LIFETIME_SECS, TimerMode::Once) },
            Transform::from_translation(position.extend(0.0)),
            Velocity(velocity),
            GameplayEntity,
        ));
    }
}

fn particle_lifetime(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Particle)>) {
    for (entity, mut particle) in &mut query {
        particle.life.tick(time.delta());
        if particle.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_particles(mut gizmos: Gizmos, query: Query<(&Transform, &Particle)>) {
    for (transform, particle) in &query {
        // 남은 수명 비율로 밝기 감소
        let frac = particle.life.fraction_remaining();
        let color = Color::srgb(frac, frac, frac * 0.6);
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            1.5,
            color,
        );
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록** — 플러그인 등록부에 `.add_plugins(effects::EffectsPlugin)` 추가.

- [ ] **Step 6: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 7: 커밋**

```bash
git add src/effects.rs src/main.rs src/config.rs
git commit -m "feat: 폭발 파티클 이펙트 시스템"
```

---

## Task 6: 이펙트 연동 (소행성 파괴 + 피격)

**Files:** Modify `src/collision.rs`

**Interfaces:**
- Consumes: `effects::spawn_explosion`.
- 변경: `bullet_vs_asteroid` 가 소행성 파괴 지점에서 `spawn_explosion`. `player_vs_asteroid` 가 피격 시 우주선 위치에서 `spawn_explosion`.

- [ ] **Step 1: 실패 테스트 — collision.rs tests에 추가**

```rust
    #[test]
    fn destroying_asteroid_spawns_particles() {
        let mut app = App::new();
        app.insert_resource(Score(0));
        app.world_mut().spawn((
            Bullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 2.0 },
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Small },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Small.radius() },
        ));
        app.world_mut().run_system_once(bullet_vs_asteroid).unwrap();
        let mut q = app.world_mut().query::<&crate::effects::Particle>();
        assert!(q.iter(app.world()).count() > 0);
    }
```

- [ ] **Step 2: 실패 확인** — Run: `cargo test destroying_asteroid_spawns_particles` → FAIL (파티클 0개)

- [ ] **Step 3: 구현 — collision.rs**

상단 `use crate::effects::spawn_explosion;` 추가. `use crate::config::EXPLOSION_PARTICLES;` 추가. `bullet_vs_asteroid` 의 소행성 파괴 블록(점수 가산 직후, 분열 전)에서:

```rust
                score.0 += asteroid.size.score();
                spawn_explosion(&mut commands, asteroid_tf.translation.truncate(), EXPLOSION_PARTICLES);
```

`player_vs_asteroid` 의 목숨 감소/피격 처리 블록에서 우주선 위치에 폭발:

```rust
            spawn_explosion(&mut commands, player_tf.translation.truncate(), EXPLOSION_PARTICLES);
            lives.0 = lives.0.saturating_sub(1);
```

- [ ] **Step 4: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 5: 커밋**

```bash
git add src/collision.rs
git commit -m "feat: 소행성 파괴·우주선 피격 시 폭발 이펙트 연동"
```

---

## Task 7: 적 총알(EnemyBullet)

**Files:** Modify `src/bullet.rs`, `src/config.rs`

**Interfaces:**
- Produces: `bullet::EnemyBullet { life: Timer }` (Component). `bullet::spawn_enemy_bullet(commands, position, velocity)` 공용 헬퍼(ufo에서 사용). `enemy_bullet_lifetime`/`draw_enemy_bullets` 시스템(BulletPlugin에 등록). EnemyBullet는 `Velocity` 보유, `Wrapping` 없음.
- Consumes: `config::{ENEMY_BULLET_COLLIDER_RADIUS, UFO_BULLET_LIFETIME_SECS}`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
pub const ENEMY_BULLET_COLLIDER_RADIUS: f32 = 2.5;
pub const UFO_BULLET_LIFETIME_SECS: f32 = 2.5;
pub const UFO_BULLET_SPEED: f32 = 320.0;
```

- [ ] **Step 2: 실패 테스트 — bullet.rs tests에 추가**

```rust
    #[test]
    fn spawn_enemy_bullet_creates_entity() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_enemy_bullet(&mut commands, Vec2::ZERO, Vec2::new(0.0, -100.0));
            })
            .unwrap();
        let mut q = app.world_mut().query::<&EnemyBullet>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn expired_enemy_bullet_is_despawned() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(2.0));
        let e = app.world_mut().spawn(EnemyBullet { life: timer }).id();
        app.world_mut().run_system_once(enemy_bullet_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
```

- [ ] **Step 3: 실패 확인** — Run: `cargo test enemy_bullet` → FAIL (미정의)

- [ ] **Step 4: 구현 — bullet.rs**

`use crate::config::{...}` 에 `ENEMY_BULLET_COLLIDER_RADIUS, UFO_BULLET_LIFETIME_SECS` 추가. 컴포넌트/헬퍼/시스템 추가:

```rust
#[derive(Component)]
pub struct EnemyBullet {
    pub life: Timer,
}

pub fn spawn_enemy_bullet(commands: &mut Commands, position: Vec2, velocity: Vec2) {
    commands.spawn((
        EnemyBullet {
            life: Timer::from_seconds(UFO_BULLET_LIFETIME_SECS, TimerMode::Once),
        },
        Transform::from_translation(position.extend(0.0)),
        Velocity(velocity),
        Collider { radius: ENEMY_BULLET_COLLIDER_RADIUS },
        GameplayEntity,
    ));
}

fn enemy_bullet_lifetime(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut EnemyBullet)>) {
    for (entity, mut bullet) in &mut query {
        bullet.life.tick(time.delta());
        if bullet.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_enemy_bullets(mut gizmos: Gizmos, query: Query<&Transform, With<EnemyBullet>>) {
    for transform in &query {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            2.5,
            Color::srgb(1.0, 0.3, 0.3),
        );
    }
}
```

`BulletPlugin::build` 에 시스템 추가:

```rust
        app.add_systems(
            Update,
            (fire_bullet, bullet_lifetime, draw_bullets, enemy_bullet_lifetime, draw_enemy_bullets)
                .run_if(in_state(GameState::Playing)),
        );
```

- [ ] **Step 5: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 6: 커밋**

```bash
git add src/bullet.rs src/config.rs
git commit -m "feat: 적 총알(EnemyBullet) 컴포넌트·수명·렌더"
```

---

## Task 8: UFO 스폰·이동

**Files:** Create `src/ufo.rs`; Modify `src/main.rs`, `src/config.rs`, `src/logic.rs`

**Interfaces:**
- Produces: `ufo::UfoSize { Large, Small }` (+ `radius()`, `score()`, `collider_radius()`), `ufo::Ufo { size, fire_timer }`, `ufo::UfoSpawnTimer(Timer)` 리소스, `ufo::spawn_ufo(commands, size, from_left)`, `ufo::UfoPlugin`. UFO는 `Velocity`(수평) 보유, `Wrapping` 없음. 화면 밖으로 나가면 `despawn_offscreen_ufo` 가 제거.
- Consumes: `config::{UFO_SPEED, UFO_LARGE_RADIUS, UFO_SMALL_RADIUS, UFO_FIRE_INTERVAL_SECS, HALF_WIDTH, HALF_HEIGHT}`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
pub const UFO_SPEED: f32 = 140.0;
pub const UFO_LARGE_RADIUS: f32 = 20.0;
pub const UFO_SMALL_RADIUS: f32 = 12.0;
pub const UFO_FIRE_INTERVAL_SECS: f32 = 1.4;
pub const UFO_SPAWN_INTERVAL_BASE: f32 = 12.0;
```

- [ ] **Step 2: 실패 테스트 — ufo.rs 하단**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn ufo_scores_small_higher_than_large() {
        assert_eq!(UfoSize::Large.score(), 200);
        assert_eq!(UfoSize::Small.score(), 1000);
        assert!(UfoSize::Small.score() > UfoSize::Large.score());
    }

    #[test]
    fn spawn_ufo_creates_one_moving_ufo() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_ufo(&mut commands, UfoSize::Large, true);
            })
            .unwrap();
        let mut q = app.world_mut().query::<(&Ufo, &Velocity)>();
        let items: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(items.len(), 1);
        assert!(items[0].1 .0.x > 0.0); // from_left → 오른쪽으로 이동
    }

    #[test]
    fn offscreen_ufo_is_despawned() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((
                Ufo { size: UfoSize::Large, fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
                Transform::from_xyz(HALF_WIDTH + 100.0, 0.0, 0.0),
            ))
            .id();
        app.world_mut().run_system_once(despawn_offscreen_ufo).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
```

- [ ] **Step 3: main.rs에 `mod ufo;` 추가 후 실패 확인** — Run: `cargo test` → FAIL (미정의)

- [ ] **Step 4: ufo.rs 구현 (테스트 모듈 위)**

```rust
use bevy::prelude::*;

use crate::components::{Collider, Velocity};
use crate::config::{
    HALF_HEIGHT, HALF_WIDTH, UFO_FIRE_INTERVAL_SECS, UFO_LARGE_RADIUS, UFO_SMALL_RADIUS, UFO_SPEED,
};
use crate::state::{GameState, GameplayEntity};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UfoSize {
    Large,
    Small,
}

impl UfoSize {
    pub fn radius(self) -> f32 {
        match self {
            UfoSize::Large => UFO_LARGE_RADIUS,
            UfoSize::Small => UFO_SMALL_RADIUS,
        }
    }
    pub fn score(self) -> u32 {
        match self {
            UfoSize::Large => 200,
            UfoSize::Small => 1000,
        }
    }
}

#[derive(Component)]
pub struct Ufo {
    pub size: UfoSize,
    pub fire_timer: Timer,
}

pub struct UfoPlugin;

impl Plugin for UfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (ufo_wobble, despawn_offscreen_ufo, draw_ufos).run_if(in_state(GameState::Playing)),
        );
    }
}

/// 화면 좌/우 가장자리에서 등장해 반대편으로 수평 이동하는 UFO 1기를 스폰.
pub fn spawn_ufo(commands: &mut Commands, size: UfoSize, from_left: bool) {
    let mut rng = rand::rng();
    let dir = if from_left { 1.0 } else { -1.0 };
    let x = -dir * (HALF_WIDTH + size.radius());
    let y = rng.random_range(-HALF_HEIGHT * 0.6..HALF_HEIGHT * 0.6);
    commands.spawn((
        Ufo {
            size,
            fire_timer: Timer::from_seconds(UFO_FIRE_INTERVAL_SECS, TimerMode::Repeating),
        },
        Transform::from_xyz(x, y, 0.0),
        Velocity(Vec2::new(dir * UFO_SPEED, 0.0)),
        Collider { radius: size.radius() },
        GameplayEntity,
    ));
}

/// 수직 사인 흔들림(수평 이동은 apply_velocity가 처리).
fn ufo_wobble(time: Res<Time>, mut query: Query<&mut Transform, With<Ufo>>) {
    let dt = time.delta_secs();
    let wobble = (time.elapsed_secs() * 2.0).cos() * 40.0 * dt;
    for mut transform in &mut query {
        transform.translation.y += wobble;
    }
}

fn despawn_offscreen_ufo(mut commands: Commands, query: Query<(Entity, &Transform), With<Ufo>>) {
    for (entity, transform) in &query {
        if transform.translation.x.abs() > HALF_WIDTH + 60.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_ufos(mut gizmos: Gizmos, query: Query<(&Transform, &Ufo)>) {
    for (transform, ufo) in &query {
        let r = ufo.size.radius();
        let pos = transform.translation.truncate();
        // 몸통(원)
        gizmos.circle_2d(Isometry2d::from_translation(pos), r, Color::srgb(0.7, 1.0, 0.7));
        // 상단 돔(작은 원)
        gizmos.circle_2d(
            Isometry2d::from_translation(pos + Vec2::new(0.0, r * 0.5)),
            r * 0.5,
            Color::srgb(0.7, 1.0, 0.7),
        );
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록** — `.add_plugins(ufo::UfoPlugin)` 추가.

- [ ] **Step 6: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 7: 커밋**

```bash
git add src/ufo.rs src/main.rs src/config.rs
git commit -m "feat: 적 UFO 2종 스폰·이동·렌더"
```

---

## Task 9: UFO 사격 + 조준

**Files:** Modify `src/logic.rs`, `src/ufo.rs`

**Interfaces:**
- Produces: `logic::aim_direction(from: Vec2, to: Vec2) -> Vec2` (정규화, 영벡터면 `Vec2::Y`). `ufo::ufo_fire` 시스템(UfoPlugin, Update). 큰 UFO=무작위, 작은 UFO=플레이어 조준. `bullet::spawn_enemy_bullet` 사용.
- Consumes: `bullet::spawn_enemy_bullet`, `config::UFO_BULLET_SPEED`, `player::Player`.

- [ ] **Step 1: 실패 테스트 — logic.rs tests에 추가**

```rust
    #[test]
    fn aim_direction_points_toward_target() {
        let d = aim_direction(Vec2::ZERO, Vec2::new(0.0, 10.0));
        assert!((d - Vec2::new(0.0, 1.0)).length() < 1e-4);
    }

    #[test]
    fn aim_direction_zero_defaults_up() {
        let d = aim_direction(Vec2::ZERO, Vec2::ZERO);
        assert!((d - Vec2::Y).length() < 1e-4);
    }
```

- [ ] **Step 2: 실패 확인** — Run: `cargo test aim_direction` → FAIL (미정의)

- [ ] **Step 3: 구현 — logic.rs**

```rust
/// from에서 to를 향하는 단위 벡터. 같은 지점이면 기본값 Vec2::Y.
pub fn aim_direction(from: Vec2, to: Vec2) -> Vec2 {
    (to - from).try_normalize().unwrap_or(Vec2::Y)
}
```

- [ ] **Step 4: ufo.rs — ufo_fire 시스템 추가 + 등록**

상단 `use`에 추가: `use crate::bullet::spawn_enemy_bullet;`, `use crate::config::UFO_BULLET_SPEED;`, `use crate::logic::aim_direction;`, `use crate::player::Player;`, `use rand::RngExt;`.

```rust
fn ufo_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut ufos: Query<(&Transform, &mut Ufo)>,
    players: Query<&Transform, With<Player>>,
) {
    let player_pos = players.single().ok().map(|t| t.translation.truncate());
    for (ufo_tf, mut ufo) in &mut ufos {
        ufo.fire_timer.tick(time.delta());
        if !ufo.fire_timer.is_finished() {
            continue;
        }
        let origin = ufo_tf.translation.truncate();
        let dir = match ufo.size {
            UfoSize::Small => match player_pos {
                Some(p) => aim_direction(origin, p),
                None => {
                    let mut rng = rand::rng();
                    let a = rng.random_range(0.0..std::f32::consts::TAU);
                    Vec2::new(a.cos(), a.sin())
                }
            },
            UfoSize::Large => {
                let mut rng = rand::rng();
                let a = rng.random_range(0.0..std::f32::consts::TAU);
                Vec2::new(a.cos(), a.sin())
            }
        };
        spawn_enemy_bullet(&mut commands, origin, dir * UFO_BULLET_SPEED);
    }
}
```

`UfoPlugin::build` 의 Update 튜플에 `ufo_fire` 추가:

```rust
            (ufo_wobble, ufo_fire, despawn_offscreen_ufo, draw_ufos)
                .run_if(in_state(GameState::Playing)),
```

- [ ] **Step 5: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 6: 커밋**

```bash
git add src/logic.rs src/ufo.rs
git commit -m "feat: UFO 사격(큰=무작위, 작은=플레이어 조준)"
```

---

## Task 10: UFO/적총알 충돌 + 점수 + 분열 속도 통일

**Files:** Modify `src/collision.rs`

**Interfaces:**
- Consumes: `ufo::Ufo`, `bullet::EnemyBullet`, `effects::spawn_explosion`, `logic::AsteroidSize::speed_scale`.
- 신규 시스템(CollisionPlugin, Update, in_state Playing): `bullet_vs_ufo`, `enemy_bullet_vs_player`, `ufo_vs_player`. 그리고 `bullet_vs_asteroid` 의 분열 자식 속도를 크기 기반 랜덤으로 통일.

- [ ] **Step 1: 실패 테스트 — collision.rs tests에 추가**

```rust
    #[test]
    fn bullet_destroys_ufo_and_scores() {
        use crate::ufo::{Ufo, UfoSize};
        let mut app = App::new();
        app.insert_resource(Score(0));
        let bullet = app.world_mut().spawn((
            Bullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 2.0 },
        )).id();
        app.world_mut().spawn((
            Ufo { size: UfoSize::Small, fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: UfoSize::Small.radius() },
        ));
        app.world_mut().run_system_once(bullet_vs_ufo).unwrap();
        assert!(app.world().get_entity(bullet).is_err());
        assert_eq!(app.world().resource::<Score>().0, 1000);
        let mut q = app.world_mut().query::<&Ufo>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn enemy_bullet_hits_player_loses_life() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(Lives(3));
        app.world_mut().spawn((
            Player, Transform::from_xyz(0.0, 0.0, 0.0), Collider { radius: 12.0 },
        ));
        app.world_mut().spawn((
            EnemyBullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 2.5 },
        ));
        app.world_mut().run_system_once(enemy_bullet_vs_player).unwrap();
        assert_eq!(app.world().resource::<Lives>().0, 2);
    }
```

- [ ] **Step 2: 실패 확인** — Run: `cargo test bullet_destroys_ufo_and_scores enemy_bullet_hits_player_loses_life` → FAIL (미정의)

- [ ] **Step 3: 구현 — collision.rs**

상단 `use` 추가: `use crate::ufo::Ufo;`, `use crate::bullet::{Bullet, EnemyBullet};`(기존 Bullet import에 EnemyBullet 병합), `use rand::RngExt;`. `spawn_asteroid`/`spawn_explosion`/`next_asteroid_size`/`AsteroidSize` 는 기존/직전 태스크에서 import됨.

`bullet_vs_ufo` 추가:

```rust
fn bullet_vs_ufo(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo)>,
) {
    let mut destroyed: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for (bullet_entity, bullet_tf, bullet_col) in &bullets {
        for (ufo_entity, ufo_tf, ufo_col, ufo) in &ufos {
            if destroyed.contains(&ufo_entity) {
                continue;
            }
            if circles_overlap(
                bullet_tf.translation.truncate(),
                bullet_col.radius,
                ufo_tf.translation.truncate(),
                ufo_col.radius,
            ) {
                destroyed.insert(ufo_entity);
                commands.entity(bullet_entity).despawn();
                commands.entity(ufo_entity).despawn();
                score.0 += ufo.size.score();
                spawn_explosion(&mut commands, ufo_tf.translation.truncate(), EXPLOSION_PARTICLES);
                break;
            }
        }
    }
}
```

`enemy_bullet_vs_player` 추가:

```rust
fn enemy_bullet_vs_player(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    bullets: Query<(Entity, &Transform, &Collider), With<EnemyBullet>>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (bullet_entity, bullet_tf, bullet_col) in &bullets {
        if circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            bullet_tf.translation.truncate(),
            bullet_col.radius,
        ) {
            commands.entity(bullet_entity).despawn();
            spawn_explosion(&mut commands, player_tf.translation.truncate(), EXPLOSION_PARTICLES);
            lives.0 = lives.0.saturating_sub(1);
            commands.entity(player_entity).despawn();
            if lives.0 == 0 {
                next_state.set(GameState::GameOver);
            } else {
                spawn_player_entity(&mut commands);
            }
            break;
        }
    }
}
```

`ufo_vs_player` 추가(우주선-UFO 몸통 충돌):

```rust
fn ufo_vs_player(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    ufos: Query<(&Transform, &Collider), With<Ufo>>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (ufo_tf, ufo_col) in &ufos {
        if circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            ufo_tf.translation.truncate(),
            ufo_col.radius,
        ) {
            spawn_explosion(&mut commands, player_tf.translation.truncate(), EXPLOSION_PARTICLES);
            lives.0 = lives.0.saturating_sub(1);
            commands.entity(player_entity).despawn();
            if lives.0 == 0 {
                next_state.set(GameState::GameOver);
            } else {
                spawn_player_entity(&mut commands);
            }
            break;
        }
    }
}
```

`bullet_vs_asteroid` 의 분열 속도를 크기 기반 랜덤으로 통일 — 기존 `for dir in [1.0, -1.0]` 블록을 교체:

```rust
                if let Some(next) = next_asteroid_size(asteroid.size) {
                    let base = asteroid_tf.translation.truncate();
                    let mut rng = rand::rng();
                    for _ in 0..2 {
                        let angle = rng.random_range(0.0..std::f32::consts::TAU);
                        let speed = rng.random_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED)
                            * next.speed_scale();
                        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
                        spawn_asteroid(&mut commands, next, base, velocity);
                    }
                }
```

`use crate::config::{ASTEROID_MAX_SPEED, ASTEROID_MIN_SPEED, EXPLOSION_PARTICLES};` 추가.

`CollisionPlugin::build` 에 신규 시스템 등록:

```rust
        app.add_systems(
            Update,
            (bullet_vs_asteroid, player_vs_asteroid, bullet_vs_ufo, enemy_bullet_vs_player, ufo_vs_player)
                .run_if(in_state(GameState::Playing)),
        );
```

- [ ] **Step 4: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 5: 커밋**

```bash
git add src/collision.rs
git commit -m "feat: UFO/적총알 충돌 처리와 분열 자식 크기별 속도"
```

---

## Task 11: 웨이브 난이도 상승

**Files:** Modify `src/logic.rs`, `src/config.rs`, `src/state.rs`, `src/asteroid.rs`, `src/ufo.rs`, `src/main.rs`

**Interfaces:**
- Produces: `state::Wave(pub u32)` (Resource). 순수 공식 `logic::{asteroid_count_for_wave, asteroid_speed_scale_for_wave, ufo_interval_for_wave, small_ufo_probability_for_wave}`. `asteroid.rs::spawn_wave` 가 `Wave` 를 읽어 개수/속도 반영. UFO 스폰 시스템이 `Wave` 로 간격/소형 확률 결정.
- Consumes: `config::{BASE_ASTEROIDS, MAX_ASTEROIDS, UFO_SPAWN_INTERVAL_BASE, UFO_SPAWN_INTERVAL_MIN}`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
pub const BASE_ASTEROIDS: usize = 4;
pub const MAX_ASTEROIDS: usize = 10;
pub const UFO_SPAWN_INTERVAL_MIN: f32 = 5.0;
```

(`UFO_SPAWN_INTERVAL_BASE` 는 Task 8에서 추가됨. `INITIAL_ASTEROIDS`(=4)는 이제 `BASE_ASTEROIDS` 로 대체 — asteroid.rs에서 참조를 바꾼다.)

- [ ] **Step 2: 실패 테스트 — logic.rs tests에 추가**

```rust
    #[test]
    fn wave_scaling_formulas() {
        // 개수: 기본4 + wave, 상한10
        assert_eq!(asteroid_count_for_wave(0), 4);
        assert_eq!(asteroid_count_for_wave(3), 7);
        assert_eq!(asteroid_count_for_wave(50), 10); // 상한
        // 속도 배수: 웨이브↑ → 증가
        assert!(asteroid_speed_scale_for_wave(5) > asteroid_speed_scale_for_wave(0));
        assert_eq!(asteroid_speed_scale_for_wave(0), 1.0);
        // UFO 간격: 웨이브↑ → 감소, 하한 존재
        assert!(ufo_interval_for_wave(5) < ufo_interval_for_wave(0));
        assert!(ufo_interval_for_wave(100) >= 5.0);
        // 소형 확률: 웨이브↑ → 증가, [0,1]
        assert!(small_ufo_probability_for_wave(10) > small_ufo_probability_for_wave(0));
        assert!(small_ufo_probability_for_wave(100) <= 1.0);
    }
```

- [ ] **Step 3: 실패 확인** — Run: `cargo test wave_scaling_formulas` → FAIL (미정의)

- [ ] **Step 4: 구현 — logic.rs (파일 상단에 `use crate::config::...` 없이 값은 인자/리터럴로; 상수는 config에서 별도 참조)**

```rust
pub fn asteroid_count_for_wave(wave: u32) -> usize {
    (crate::config::BASE_ASTEROIDS + wave as usize).min(crate::config::MAX_ASTEROIDS)
}

pub fn asteroid_speed_scale_for_wave(wave: u32) -> f32 {
    (1.0 + wave as f32 * 0.08).min(2.0)
}

pub fn ufo_interval_for_wave(wave: u32) -> f32 {
    (crate::config::UFO_SPAWN_INTERVAL_BASE - wave as f32 * 0.8)
        .max(crate::config::UFO_SPAWN_INTERVAL_MIN)
}

pub fn small_ufo_probability_for_wave(wave: u32) -> f32 {
    (0.2 + wave as f32 * 0.05).min(0.9)
}
```

- [ ] **Step 5: state.rs — Wave 리소스 추가 + 리셋**

```rust
#[derive(Resource, Default)]
pub struct Wave(pub u32);
```

`GameStatePlugin::build` 의 `insert_resource` 에 `.insert_resource(Wave(1))` 추가. `reset_game` 에 `wave.0 = 1;` 추가(시그니처에 `mut wave: ResMut<Wave>` 추가).

- [ ] **Step 6: asteroid.rs — spawn_wave가 Wave 반영, wave_control이 Wave 증가**

`use crate::state::{GameState, GameplayEntity, Wave};`, `use crate::logic::{asteroid_count_for_wave, asteroid_speed_scale_for_wave, AsteroidSize};`, `use crate::config::{...}` 정리. `spawn_wave` 를 Wave 인자 기반으로:

```rust
fn spawn_wave(commands: &mut Commands, wave: u32) {
    let count = asteroid_count_for_wave(wave);
    let scale = asteroid_speed_scale_for_wave(wave);
    for _ in 0..count {
        let base = random_velocity(AsteroidSize::Large);
        spawn_asteroid(commands, AsteroidSize::Large, random_spawn_position(), base * scale);
    }
}

fn spawn_initial_wave(mut commands: Commands, wave: Res<Wave>) {
    spawn_wave(&mut commands, wave.0);
}

fn wave_control(mut commands: Commands, mut wave: ResMut<Wave>, asteroids: Query<(), With<Asteroid>>) {
    if asteroids.iter().count() == 0 {
        wave.0 += 1;
        spawn_wave(&mut commands, wave.0);
    }
}
```

- [ ] **Step 7: ufo.rs — 스폰 타이머/소형 확률을 Wave로 결정**

`UfoSpawnTimer` 리소스와 스폰 시스템 추가. `UfoPlugin::build` 에 리소스 삽입 + 시스템 등록:

```rust
#[derive(Resource)]
pub struct UfoSpawnTimer(pub Timer);
```

`build` 에 추가:

```rust
        app.insert_resource(UfoSpawnTimer(Timer::from_seconds(
            crate::config::UFO_SPAWN_INTERVAL_BASE,
            TimerMode::Once,
        )))
        .add_systems(Update, ufo_spawn_system.run_if(in_state(GameState::Playing)));
```

`ufo_spawn_system`:

```rust
fn ufo_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    wave: Res<crate::state::Wave>,
    mut timer: ResMut<UfoSpawnTimer>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    let mut rng = rand::rng();
    let size = if rng.random_range(0.0..1.0) < crate::logic::small_ufo_probability_for_wave(wave.0) {
        UfoSize::Small
    } else {
        UfoSize::Large
    };
    let from_left = rng.random_range(0.0..1.0) < 0.5;
    spawn_ufo(&mut commands, size, from_left);
    // 다음 간격을 웨이브 기반으로 재설정
    let interval = crate::logic::ufo_interval_for_wave(wave.0);
    timer.0 = Timer::from_seconds(interval, TimerMode::Once);
}
```

(`use rand::RngExt;` 필요.)

- [ ] **Step 8: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공

- [ ] **Step 9: 커밋**

```bash
git add src/logic.rs src/config.rs src/state.rs src/asteroid.rs src/ufo.rs
git commit -m "feat: 웨이브별 난이도 상승(소행성 수/속도, UFO 간격/소형 비율)"
```

---

## Task 12: 최고점수 저장 (bevy-persistent)

**Files:** Modify `Cargo.toml`, `src/state.rs`, `src/main.rs`, `src/ui.rs`, `src/collision.rs`

**Interfaces:**
- Produces: `state::HighScore(pub u32)` (`Resource, Serialize, Deserialize, Default`). `logic::update_high_score(current, new) -> u32`. 앱 시작 시 `Persistent<HighScore>` 삽입, 게임오버 시 갱신·저장. HUD/게임오버에 표시.
- Consumes: bevy-persistent, serde, dirs.

- [ ] **Step 1: 의존성 추가**

Run:
```bash
cargo add bevy-persistent --features json
cargo add serde --features derive
cargo add dirs
```
Expected: `Cargo.toml` 에 세 의존성 추가. (버전은 해석되는 최신; bevy-persistent는 0.11대.)

- [ ] **Step 2: 실패 테스트 — logic.rs tests에 추가**

```rust
    #[test]
    fn high_score_keeps_maximum() {
        assert_eq!(update_high_score(100, 250), 250);
        assert_eq!(update_high_score(300, 250), 300);
        assert_eq!(update_high_score(0, 0), 0);
    }
```

- [ ] **Step 3: 실패 확인** — Run: `cargo test high_score_keeps_maximum` → FAIL (미정의)

- [ ] **Step 4: logic.rs — 순수 함수 추가**

```rust
pub fn update_high_score(current: u32, new: u32) -> u32 {
    current.max(new)
}
```

- [ ] **Step 5: state.rs — HighScore 정의**

상단에 `use serde::{Deserialize, Serialize};` 추가.

```rust
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct HighScore(pub u32);
```

- [ ] **Step 6: main.rs — Persistent<HighScore> 초기화 시스템**

`use bevy_persistent::prelude::*;` 추가. Startup 시스템 추가 등록:

```rust
        .add_systems(Startup, (setup_camera, setup_high_score))
```

(기존 `.add_systems(Startup, setup_camera)` 를 위 튜플로 교체.)

```rust
fn setup_high_score(mut commands: Commands) {
    let dir = dirs::config_dir()
        .map(|d| d.join("bevy-asteroids"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    commands.insert_resource(
        Persistent::<state::HighScore>::builder()
            .name("high score")
            .format(StorageFormat::Json)
            .path(dir.join("highscore.json"))
            .default(state::HighScore(0))
            .build()
            .expect("최고점수 리소스 초기화 실패"),
    );
}
```

- [ ] **Step 7: collision.rs — 게임오버 전환 시 최고점수 갱신·저장**

게임오버로 전환하는 두 지점(`player_vs_asteroid`, `enemy_bullet_vs_player`, `ufo_vs_player`)은 시스템 파라미터가 많아지므로, 별도 시스템으로 분리한다. `state.rs` 에 상태 전환 훅을 두는 대신 `OnEnter(GameState::GameOver)` 에서 갱신:

`ui.rs` 또는 `state.rs` 중 `state.rs` 의 `GameStatePlugin::build` 에 추가:

```rust
            .add_systems(OnEnter(GameState::GameOver), save_high_score)
```

`state.rs` 에 시스템 추가(상단 `use bevy_persistent::prelude::*;`, `use crate::logic::update_high_score;`):

```rust
fn save_high_score(score: Res<Score>, mut high: ResMut<Persistent<HighScore>>) {
    let updated = update_high_score(high.0, score.0);
    if updated != high.0 {
        high.0 = updated;
        let _ = high.persist();
    }
}
```

(이 방식이면 collision.rs 변경 불필요 — 게임오버 진입 시 자동 저장. Task Files의 collision.rs는 제외해도 됨.)

- [ ] **Step 8: ui.rs — HUD/게임오버에 최고점수 표시**

`use bevy_persistent::prelude::*;`, `use crate::state::HighScore;` 추가. `update_hud` 시그니처에 `high: Res<Persistent<HighScore>>` 추가하고 표시 문자열에 최고점수 포함:

```rust
fn update_hud(
    score: Res<Score>,
    lives: Res<Lives>,
    high: Res<Persistent<HighScore>>,
    mut query: Query<&mut Text, With<Hud>>,
) {
    for mut text in &mut query {
        text.0 = format!("Score: {}   Lives: {}   High: {}", score.0, lives.0, high.0);
    }
}
```

`spawn_game_over` 시그니처에 `high: Res<Persistent<HighScore>>` 추가하고 문구에 최고점수 포함:

```rust
fn spawn_game_over(mut commands: Commands, score: Res<Score>, high: Res<Persistent<HighScore>>) {
    commands.spawn((
        GameOverScreen,
        Text::new(format!(
            "GAME OVER\nScore: {}\nHigh: {}\nPress R to restart",
            score.0, high.0
        )),
        TextFont { font_size: FontSize::Px(40.0), ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Percent(36.0), left: Val::Percent(34.0), ..default() },
    ));
}
```

주의: `save_high_score`(OnEnter GameOver)와 `spawn_game_over`(OnEnter GameOver) 순서상 저장이 먼저 반영되려면, `spawn_game_over` 는 갱신된 값을 읽도록 `save_high_score` 이후에 등록한다. `state.rs` 에서 `save_high_score` 를 OnEnter(GameOver)에, `ui.rs` 의 `spawn_game_over` 도 OnEnter(GameOver)에 등록되어 있으니, 표시값이 한 프레임 늦어도 무방(게임오버 화면은 유지되며 다음 진입 시 정확). 정확성을 원하면 `spawn_game_over` 를 `.after(save_high_score)` 로 지정.

- [ ] **Step 9: 통과 확인 + 빌드** — Run: `cargo test` → PASS, `cargo build` → 성공. (수동: 게임오버 후 재실행 시 High 유지 확인은 사용자 몫)

- [ ] **Step 10: 커밋 (push 하지 않음 — 컨트롤러가 최종 리뷰 후 처리)**

```bash
git add Cargo.toml Cargo.lock src/state.rs src/main.rs src/ui.rs src/logic.rs
git commit -m "feat: 최고점수 저장(bevy-persistent) 및 HUD/게임오버 표시"
```

---

## Self-Review (작성자 점검 결과)

**1. Spec coverage:** 설계 문서 대응 — 브레이크(Task 3), 소행성 크기별 속도(1), 회전(2), 우주선 컬러·화염(4), 폭발 파티클(5) + 피격 연동(6), 적총알(7), UFO 스폰/이동(8)·사격/조준(9)·충돌/점수(10), 웨이브 난이도(11), 최고점수 저장(12). DoD 전 항목 대응.

**2. Placeholder scan:** "TBD/TODO/적절히" 없음. 각 스텝에 실제 코드·명령·기대값 포함.

**3. Type consistency:** `AsteroidSize::speed_scale`(1)은 2·10·11에서 동일 사용. `spawn_explosion`(5)은 6·10에서 동일 시그니처. `spawn_enemy_bullet`(7)은 9에서, `EnemyBullet`은 7·10에서, `Ufo`/`UfoSize`(8)는 9·10·11에서, 난이도 공식(11)은 asteroid/ufo에서, `HighScore`/`update_high_score`(12)는 state/ui에서 일관. `apply_velocity`가 UFO·적총알·파티클을 이동시키고 `Wrapping` 미부여로 순환 제외 — Global Constraints와 일치.

**알려진 판단 포인트(구현 시 컴파일러 기준 확인):** `Color::srgb` const 여부(→ 필요시 함수화), bevy-persistent/serde/dirs 실제 버전(→ `cargo add`가 해석), `Vec2::try_normalize`/`fraction_remaining` 존재(0.19/Timer API — 없으면 동등 대체). 이들은 Phase 1처럼 구현자가 소스 근거로 조정하고 보고한다.

---

## 빌드 순서 / 실행

**A·B(Task 1–4) → C(5–6) → D(7–10) → E(11–12).** 각 태스크는 `cargo test`로 검증(구현자는 `cargo run` 실행 금지 — GUI 블로킹, 시각 확인은 사용자). 각 태스크 후 커밋. 마지막 태스크만 push 보류(최종 리뷰 후 컨트롤러가 처리).
