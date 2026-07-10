# Bevy Asteroids Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rust 게임 엔진 Bevy 0.19를 학습하기 위해 벡터 라인 그래픽 스타일의 클래식 애스토로이드(우주선 슈터) 게임을 만든다.

**Architecture:** 기능별 Bevy `Plugin`으로 모듈을 분리한다. 순수 계산 로직(`logic.rs`)은 Bevy World와 무관하게 단위 테스트하고, 그 위에 ECS 시스템을 얹는다. 렌더링은 메시 대신 **Gizmos 즉시 모드**로 흰색 벡터 라인을 매 프레임 그린다(시각 결과는 설계 문서의 "LineStrip 벡터 라인"과 동일, 파이프라인은 더 견고). 물리 이동은 `FixedUpdate`, 입력·충돌·렌더·UI는 `Update`.

**Tech Stack:** Rust (edition 2021) · Bevy `0.19` · rand `0.10` · macOS arm64 (Metal)

## Global Constraints

- Bevy 버전은 정확히 `0.19` (Cargo.toml `bevy = "0.19"`). 아래 모든 코드는 0.19 API 기준이며, 부록 "Verified Bevy 0.19 API"에 실제 소스로 검증한 시그니처를 정리했다.
- rand 버전은 `0.10` (`rand::rng()`, `random_range`, `random_bool` — 0.9+ 리네이밍 API).
- 렌더링은 Gizmos만 사용한다. Mesh/Material/에셋 파이프라인은 쓰지 않는다(외부 이미지 파일 0개).
- 좌표계: 화면 중심이 원점(0,0). 창 크기 1280×720, 반너비 `HALF_WIDTH=640`, 반높이 `HALF_HEIGHT=360`.
- 모든 gizmo 색은 `Color::WHITE`.
- 게임 진행 중 스폰되는 모든 엔티티(우주선·소행성·총알·HUD)는 `GameplayEntity` 마커를 포함한다(상태 전환 시 일괄 정리용).
- 커밋 메시지는 한국어로 작성한다.

---

## File Structure

```
Cargo.toml           # bevy 0.19, rand 0.10, dev 최적화 프로파일
src/
├── main.rs          # App 조립, DefaultPlugins+윈도우, 카메라, FixedUpdate 60Hz, 플러그인 등록
├── config.rs        # 게임플레이 상수 (창 크기, 속도, 크기, 목숨, 총알 수명 등)
├── logic.rs         # 순수 함수 + AsteroidSize enum (Bevy World 불필요, 단위 테스트 대상)
├── components.rs     # 공유 컴포넌트: Velocity, Collider, Wrapping
├── state.rs         # GameState(States), Score/Lives 리소스, GameplayEntity 정리
├── movement.rs      # MovementPlugin: apply_velocity + wrap_around (FixedUpdate)
├── player.rs        # PlayerPlugin: Player, 스폰, 입력(회전/추진), gizmo 렌더
├── bullet.rs        # BulletPlugin: Bullet(Timer), 발사, 수명 소멸, gizmo 렌더
├── asteroid.rs      # AsteroidPlugin: Asteroid, 웨이브 스폰, 다각형 생성, 렌더, 웨이브 제어
├── collision.rs     # CollisionPlugin: 총알↔소행성(분열+점수), 우주선↔소행성(목숨/게임오버)
└── ui.rs            # UiPlugin: HUD 점수/목숨, 게임오버 화면, R 재시작
```

각 모듈은 대응 `Plugin`을 노출하고 `main.rs`에서 조립한다. `logic.rs`만 Bevy 시스템이 아닌 순수 함수 모음이다.

---

## Task 1: 프로젝트 스캐폴드 — 검은 창 띄우기

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Interfaces:**
- Produces: 실행 가능한 바이너리 `bevy-asteroids`. `setup_camera` 시스템이 `Camera2d`를 스폰. `config` 모듈 아직 없음(Task 2에서 추가) — 이 태스크에서는 창 크기를 리터럴로 둔다.

- [ ] **Step 1: Cargo.toml 작성**

```toml
[package]
name = "bevy-asteroids"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.19"
rand = "0.10"

# 내 코드는 빠르게 증분 컴파일하되, 의존성(bevy 등)은 최적화해 런타임 성능 확보
[profile.dev.package."*"]
opt-level = 3
```

- [ ] **Step 2: src/main.rs 작성 (검은 창 + 2D 카메라)**

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Asteroids".into(),
                // Bevy 0.19: WindowResolution는 (u32,u32)/[u32;2]/UVec2에만 From 구현 (float 튜플 불가)
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

- [ ] **Step 3: 빌드 확인 (첫 빌드는 수 분 소요)**

Run: `cargo build`
Expected: 컴파일 성공 (bevy 의존성 다운로드/빌드 포함).

- [ ] **Step 4: 실행해 검은 창 확인**

Run: `cargo run`
Expected: "Bevy Asteroids" 제목의 1280×720 검은 창이 뜬다. 확인 후 창을 닫는다.
(개발 중 재컴파일을 빠르게 하려면: `cargo run --features bevy/dynamic_linking`)

- [ ] **Step 5: 커밋**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: 프로젝트 스캐폴드 및 검은 창 렌더링"
```

---

## Task 2: 순수 로직 & 핵심 타입 (TDD)

**Files:**
- Create: `src/config.rs`
- Create: `src/logic.rs`
- Modify: `src/main.rs` (모듈 선언 추가)

**Interfaces:**
- Produces:
  - `config` 상수: `WINDOW_WIDTH, WINDOW_HEIGHT, HALF_WIDTH, HALF_HEIGHT, SHIP_ROTATION_SPEED, SHIP_THRUST, SHIP_COLLIDER_RADIUS, BULLET_SPEED, BULLET_LIFETIME_SECS, BULLET_COLLIDER_RADIUS, STARTING_LIVES, INITIAL_ASTEROIDS, ASTEROID_MIN_SPEED, ASTEROID_MAX_SPEED` (모두 `f32`/`u32`/`usize`)
  - `logic::AsteroidSize` enum `{ Large, Medium, Small }` + `radius(self)->f32`, `score(self)->u32`
  - `logic::next_asteroid_size(AsteroidSize)->Option<AsteroidSize>`
  - `logic::wrap_position(pos: Vec2, half: Vec2)->Vec2`
  - `logic::circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32)->bool`

- [ ] **Step 1: 실패하는 테스트 작성 — logic.rs 하단**

```rust
// src/logic.rs
use bevy::math::Vec2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_size_shrinks_then_none() {
        assert_eq!(next_asteroid_size(AsteroidSize::Large), Some(AsteroidSize::Medium));
        assert_eq!(next_asteroid_size(AsteroidSize::Medium), Some(AsteroidSize::Small));
        assert_eq!(next_asteroid_size(AsteroidSize::Small), None);
    }

    #[test]
    fn wrap_moves_past_right_edge_to_left() {
        let half = Vec2::new(640.0, 360.0);
        let wrapped = wrap_position(Vec2::new(700.0, 0.0), half);
        assert!(wrapped.x < 0.0);
        assert!((wrapped.x - (-580.0)).abs() < 1e-4);
    }

    #[test]
    fn wrap_moves_past_bottom_edge_to_top() {
        let half = Vec2::new(640.0, 360.0);
        let wrapped = wrap_position(Vec2::new(0.0, -400.0), half);
        assert!(wrapped.y > 0.0);
    }

    #[test]
    fn wrap_leaves_inside_point_unchanged() {
        let half = Vec2::new(640.0, 360.0);
        let p = Vec2::new(10.0, -20.0);
        assert_eq!(wrap_position(p, half), p);
    }

    #[test]
    fn circles_overlap_true_when_close() {
        assert!(circles_overlap(Vec2::ZERO, 5.0, Vec2::new(8.0, 0.0), 5.0));
    }

    #[test]
    fn circles_overlap_false_when_far() {
        assert!(!circles_overlap(Vec2::ZERO, 5.0, Vec2::new(20.0, 0.0), 5.0));
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 추가 (테스트 컴파일 위해)**

`src/main.rs` 최상단에 추가:

```rust
mod config;
mod logic;
```

- [ ] **Step 3: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `AsteroidSize`, `next_asteroid_size`, `wrap_position`, `circles_overlap` 미정의로 컴파일 에러.

- [ ] **Step 4: config.rs 구현**

```rust
// src/config.rs
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const HALF_WIDTH: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_HEIGHT: f32 = WINDOW_HEIGHT / 2.0;

pub const SHIP_ROTATION_SPEED: f32 = 4.0; // rad/s
pub const SHIP_THRUST: f32 = 320.0;       // units/s^2
pub const SHIP_COLLIDER_RADIUS: f32 = 12.0;

pub const BULLET_SPEED: f32 = 620.0;
pub const BULLET_LIFETIME_SECS: f32 = 1.2;
pub const BULLET_COLLIDER_RADIUS: f32 = 2.0;

pub const STARTING_LIVES: u32 = 3;

pub const INITIAL_ASTEROIDS: usize = 4;
pub const ASTEROID_MIN_SPEED: f32 = 40.0;
pub const ASTEROID_MAX_SPEED: f32 = 120.0;
```

- [ ] **Step 5: logic.rs 구현 (테스트 모듈 위에)**

`src/logic.rs`의 `use bevy::math::Vec2;` 아래, `#[cfg(test)]` 위에 삽입:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    pub fn radius(self) -> f32 {
        match self {
            AsteroidSize::Large => 45.0,
            AsteroidSize::Medium => 25.0,
            AsteroidSize::Small => 14.0,
        }
    }

    pub fn score(self) -> u32 {
        match self {
            AsteroidSize::Large => 20,
            AsteroidSize::Medium => 50,
            AsteroidSize::Small => 100,
        }
    }
}

pub fn next_asteroid_size(size: AsteroidSize) -> Option<AsteroidSize> {
    match size {
        AsteroidSize::Large => Some(AsteroidSize::Medium),
        AsteroidSize::Medium => Some(AsteroidSize::Small),
        AsteroidSize::Small => None,
    }
}

/// 화면 밖으로 나간 좌표를 반대편으로 순환시킨다. half는 반너비/반높이.
pub fn wrap_position(pos: Vec2, half: Vec2) -> Vec2 {
    let mut p = pos;
    let full = half * 2.0;
    if p.x > half.x {
        p.x -= full.x;
    } else if p.x < -half.x {
        p.x += full.x;
    }
    if p.y > half.y {
        p.y -= full.y;
    } else if p.y < -half.y {
        p.y += full.y;
    }
    p
}

/// 두 원이 겹치는지 판정 (제곱 거리 비교로 sqrt 회피).
pub fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    let r = ra + rb;
    a.distance_squared(b) <= r * r
}
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test`
Expected: PASS — 6개 테스트 모두 통과.

- [ ] **Step 7: 커밋**

```bash
git add src/config.rs src/logic.rs src/main.rs
git commit -m "feat: 순수 로직(순환/충돌/분열 크기)과 게임 상수 추가"
```

---

## Task 3: 공유 컴포넌트 & 이동 플러그인

**Files:**
- Create: `src/components.rs`
- Create: `src/movement.rs`
- Modify: `src/main.rs` (모듈 선언 + `MovementPlugin` 등록 + FixedUpdate 60Hz)

**Interfaces:**
- Consumes: `logic::wrap_position`, `config::{HALF_WIDTH, HALF_HEIGHT}`
- Produces:
  - `components::Velocity(pub Vec2)` (Component, Clone, Copy)
  - `components::Collider { pub radius: f32 }` (Component, Clone, Copy)
  - `components::Wrapping` (Component, 마커)
  - `movement::MovementPlugin` — `FixedUpdate`에 `apply_velocity` → `wrap_around` 순서로 등록
  - 시스템 함수(모듈 내부, 테스트용): `apply_velocity(Res<Time>, Query<(&mut Transform, &Velocity)>)`, `wrap_around(Query<&mut Transform, With<Wrapping>>)`

- [ ] **Step 1: components.rs 작성**

```rust
// src/components.rs
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
}

#[derive(Component)]
pub struct Wrapping;
```

- [ ] **Step 2: 실패하는 테스트 작성 — movement.rs 하단**

```rust
// src/movement.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn velocity_moves_entity() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);
        let e = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), Velocity(Vec2::new(10.0, -20.0))))
            .id();
        app.world_mut().run_system_once(apply_velocity).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        assert!((t.translation.x - 5.0).abs() < 1e-3);
        assert!((t.translation.y + 10.0).abs() < 1e-3);
    }

    #[test]
    fn wrapping_teleports_out_of_bounds_entity() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((Transform::from_xyz(HALF_WIDTH + 50.0, 0.0, 0.0), Wrapping))
            .id();
        app.world_mut().run_system_once(wrap_around).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        assert!(t.translation.x < 0.0);
    }
}
```

- [ ] **Step 3: main.rs에 모듈 선언 추가**

`src/main.rs`의 `mod logic;` 아래에 추가:

```rust
mod components;
mod movement;
```

- [ ] **Step 4: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `apply_velocity`, `wrap_around` 미정의.

- [ ] **Step 5: movement.rs 구현 (테스트 모듈 위에)**

`src/movement.rs` 최상단에 삽입:

```rust
use bevy::prelude::*;

use crate::components::{Velocity, Wrapping};
use crate::config::{HALF_HEIGHT, HALF_WIDTH};
use crate::logic::wrap_position;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (apply_velocity, wrap_around).chain());
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

fn wrap_around(mut query: Query<&mut Transform, With<Wrapping>>) {
    let half = Vec2::new(HALF_WIDTH, HALF_HEIGHT);
    for mut transform in &mut query {
        let wrapped = wrap_position(transform.translation.truncate(), half);
        transform.translation.x = wrapped.x;
        transform.translation.y = wrapped.y;
    }
}
```

- [ ] **Step 6: main.rs에 플러그인 등록 + FixedUpdate 60Hz 설정**

`src/main.rs`의 `.insert_resource(ClearColor(Color::BLACK))` 아래에 추가:

```rust
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(movement::MovementPlugin)
```

- [ ] **Step 7: 테스트 통과 + 빌드 확인**

Run: `cargo test`
Expected: PASS — 이동/순환 2개 포함 전체 통과.
Run: `cargo build`
Expected: 성공.

- [ ] **Step 8: 커밋**

```bash
git add src/components.rs src/movement.rs src/main.rs
git commit -m "feat: 속도 적분과 화면 순환 이동 플러그인"
```

---

## Task 4: 게임 상태 & 리소스

**Files:**
- Create: `src/state.rs`
- Modify: `src/main.rs` (모듈 선언 + `GameStatePlugin` 등록)

**Interfaces:**
- Consumes: `config::STARTING_LIVES`
- Produces:
  - `state::GameState` enum `{ Playing(기본), GameOver }` (`States` 파생)
  - `state::Score(pub u32)` (Resource), `state::Lives(pub u32)` (Resource)
  - `state::GameplayEntity` (Component 마커)
  - `state::GameStatePlugin` — `init_state::<GameState>()`, Score/Lives 삽입, `OnEnter(Playing)`에서 `reset_game`(점수/목숨 초기화), `OnExit(Playing)`에서 `despawn_gameplay_entities`

- [ ] **Step 1: state.rs 작성**

```rust
// src/state.rs
use bevy::prelude::*;

use crate::config::STARTING_LIVES;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Resource)]
pub struct Lives(pub u32);

/// 한 판(Playing) 동안 존재하는 모든 엔티티에 붙는 마커. 판이 끝나면 일괄 정리된다.
#[derive(Component)]
pub struct GameplayEntity;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(Score(0))
            .insert_resource(Lives(STARTING_LIVES))
            .add_systems(OnEnter(GameState::Playing), reset_game)
            .add_systems(OnExit(GameState::Playing), despawn_gameplay_entities);
    }
}

fn reset_game(mut score: ResMut<Score>, mut lives: ResMut<Lives>) {
    score.0 = 0;
    lives.0 = STARTING_LIVES;
}

fn despawn_gameplay_entities(mut commands: Commands, query: Query<Entity, With<GameplayEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 + 플러그인 등록**

`src/main.rs`의 모듈 선언부에 `mod state;` 추가. 그리고 플러그인 등록부에 추가:

```rust
        .add_plugins(state::GameStatePlugin)
```

- [ ] **Step 3: 빌드 확인**

Run: `cargo build`
Expected: 성공. (아직 눈에 보이는 변화 없음 — 상태/리소스만 등록)

- [ ] **Step 4: 커밋**

```bash
git add src/state.rs src/main.rs
git commit -m "feat: 게임 상태(Playing/GameOver)와 점수/목숨 리소스"
```

---

## Task 5: 플레이어 우주선 — 스폰·조작·렌더

**Files:**
- Create: `src/player.rs`
- Modify: `src/main.rs` (모듈 선언 + `PlayerPlugin` 등록)

**Interfaces:**
- Consumes: `components::{Velocity, Collider, Wrapping}`, `config::{SHIP_ROTATION_SPEED, SHIP_THRUST, SHIP_COLLIDER_RADIUS}`, `state::{GameState, GameplayEntity}`
- Produces:
  - `player::Player` (Component 마커)
  - `player::spawn_player_entity(commands: &mut Commands)` — 중심(0,0)에 우주선 스폰 (스폰 시스템과 충돌 리스폰이 공유)
  - `player::PlayerPlugin` — `OnEnter(Playing)`에서 `spawn_player`, `Update`(Playing 한정)에서 `player_input`, `draw_player`
  - 시스템(모듈 내부): `player_input(Res<ButtonInput<KeyCode>>, Res<Time>, Query<(&mut Transform, &mut Velocity), With<Player>>)`

- [ ] **Step 1: 실패하는 테스트 작성 — player.rs 하단**

```rust
// src/player.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Velocity;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn thrust_accelerates_forward() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowUp);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0), Velocity(Vec2::ZERO)))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        // 회전 없음 → 정면은 +Y. 위로 가속.
        assert!(v.0.y > 0.0);
        assert!(v.0.x.abs() < 1e-3);
    }

    #[test]
    fn left_key_rotates_ship() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowLeft);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0), Velocity(Vec2::ZERO)))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        // 좌회전 → z축 회전각 > 0
        let (_, angle) = t.rotation.to_axis_angle();
        assert!(angle > 0.0);
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 추가**

`src/main.rs`에 `mod player;` 추가.

- [ ] **Step 3: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `Player`, `player_input` 미정의.

- [ ] **Step 4: player.rs 구현 (테스트 모듈 위에)**

```rust
// src/player.rs
use bevy::prelude::*;

use crate::components::{Collider, Velocity, Wrapping};
use crate::config::{SHIP_COLLIDER_RADIUS, SHIP_ROTATION_SPEED, SHIP_THRUST};
use crate::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Player;

/// 우주선 로컬 좌표(정면 = +Y). 마지막 점은 첫 점과 같아 닫힌 외곽선을 만든다.
const SHIP_POINTS: [Vec2; 5] = [
    Vec2::new(0.0, 16.0),
    Vec2::new(-11.0, -12.0),
    Vec2::new(0.0, -6.0),
    Vec2::new(11.0, -12.0),
    Vec2::new(0.0, 16.0),
];

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (player_input, draw_player).run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn spawn_player_entity(commands: &mut Commands) {
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity(Vec2::ZERO),
        Collider { radius: SHIP_COLLIDER_RADIUS },
        Wrapping,
        GameplayEntity,
    ));
}

fn spawn_player(mut commands: Commands) {
    spawn_player_entity(&mut commands);
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut query {
        let mut turn = 0.0;
        if keys.pressed(KeyCode::ArrowLeft) {
            turn += 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            turn -= 1.0;
        }
        transform.rotate_z(turn * SHIP_ROTATION_SPEED * dt);

        if keys.pressed(KeyCode::ArrowUp) {
            let forward = (transform.rotation * Vec3::Y).truncate();
            velocity.0 += forward * SHIP_THRUST * dt;
        }
    }
}

fn draw_player(mut gizmos: Gizmos, query: Query<&Transform, With<Player>>) {
    for transform in &query {
        let points = SHIP_POINTS.map(|p| transform.transform_point(p.extend(0.0)).truncate());
        gizmos.linestrip_2d(points, Color::WHITE);
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록**

`src/main.rs` 플러그인 등록부에 추가:

```rust
        .add_plugins(player::PlayerPlugin)
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test`
Expected: PASS — 추진/회전 2개 포함 전체 통과.

- [ ] **Step 7: 실행해 조작 확인 (수동)**

Run: `cargo run`
Expected: 화면 중앙에 흰색 삼각형 우주선. **← →** 로 회전, **↑** 로 추진(놓아도 관성으로 미끄러짐), 화면 끝에서 반대편으로 순환. 확인 후 종료.

- [ ] **Step 8: 커밋**

```bash
git add src/player.rs src/main.rs
git commit -m "feat: 우주선 스폰·회전/추진 조작·벡터 렌더"
```

---

## Task 6: 총알 — 발사·수명·렌더

**Files:**
- Create: `src/bullet.rs`
- Modify: `src/main.rs` (모듈 선언 + `BulletPlugin` 등록)

**Interfaces:**
- Consumes: `components::{Velocity, Collider, Wrapping}`, `config::{BULLET_SPEED, BULLET_LIFETIME_SECS, BULLET_COLLIDER_RADIUS}`, `state::{GameState, GameplayEntity}`, `player::Player`
- Produces:
  - `bullet::Bullet { pub life: Timer }` (Component)
  - `bullet::BulletPlugin` — `Update`(Playing 한정)에 `fire_bullet`, `bullet_lifetime`, `draw_bullets`
  - 시스템(모듈 내부): `bullet_lifetime(Commands, Res<Time>, Query<(Entity, &mut Bullet)>)`

- [ ] **Step 1: 실패하는 테스트 작성 — bullet.rs 하단**

```rust
// src/bullet.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn expired_bullet_is_despawned() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(2.0)); // 이미 만료
        let e = app.world_mut().spawn(Bullet { life: timer }).id();
        app.world_mut().run_system_once(bullet_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }

    #[test]
    fn live_bullet_survives() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let timer = Timer::from_seconds(1.0, TimerMode::Once); // 아직 살아있음
        let e = app.world_mut().spawn(Bullet { life: timer }).id();
        app.world_mut().run_system_once(bullet_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_ok());
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 추가**

`src/main.rs`에 `mod bullet;` 추가.

- [ ] **Step 3: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `Bullet`, `bullet_lifetime` 미정의.

- [ ] **Step 4: bullet.rs 구현 (테스트 모듈 위에)**

```rust
// src/bullet.rs
use bevy::prelude::*;

use crate::components::{Collider, Velocity, Wrapping};
use crate::config::{BULLET_COLLIDER_RADIUS, BULLET_LIFETIME_SECS, BULLET_SPEED};
use crate::player::Player;
use crate::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Bullet {
    pub life: Timer,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (fire_bullet, bullet_lifetime, draw_bullets).run_if(in_state(GameState::Playing)),
        );
    }
}

fn fire_bullet(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    let Ok(ship) = query.single() else {
        return;
    };
    let forward = (ship.rotation * Vec3::Y).truncate();
    let nose = ship.translation + (ship.rotation * Vec3::Y) * 18.0;
    commands.spawn((
        Bullet {
            life: Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once),
        },
        Transform::from_translation(nose),
        Velocity(forward * BULLET_SPEED),
        Collider { radius: BULLET_COLLIDER_RADIUS },
        Wrapping,
        GameplayEntity,
    ));
}

fn bullet_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in &mut query {
        bullet.life.tick(time.delta());
        if bullet.life.is_finished() { // Bevy 0.19: Timer::finished() → is_finished()
            commands.entity(entity).despawn();
        }
    }
}

fn draw_bullets(mut gizmos: Gizmos, query: Query<&Transform, With<Bullet>>) {
    for transform in &query {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            2.0,
            Color::WHITE,
        );
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록**

```rust
        .add_plugins(bullet::BulletPlugin)
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 7: 실행해 발사 확인 (수동)**

Run: `cargo run`
Expected: **Space** 로 우주선 정면에서 흰 점(총알)이 발사돼 날아가고, ~1.2초 뒤 사라진다. 확인 후 종료.

- [ ] **Step 8: 커밋**

```bash
git add src/bullet.rs src/main.rs
git commit -m "feat: 총알 발사·수명 소멸·렌더"
```

---

## Task 7: 소행성 — 웨이브 스폰·다각형 생성·렌더·재생성

**Files:**
- Create: `src/asteroid.rs`
- Modify: `src/main.rs` (모듈 선언 + `AsteroidPlugin` 등록)

**Interfaces:**
- Consumes: `components::{Velocity, Collider, Wrapping}`, `config::*`, `logic::AsteroidSize`, `state::{GameState, GameplayEntity}`
- Produces:
  - `asteroid::Asteroid { pub size: AsteroidSize }` (Component)
  - `asteroid::AsteroidShape { pub points: Vec<Vec2> }` (Component, 로컬 외곽선)
  - `asteroid::spawn_asteroid(commands: &mut Commands, size: AsteroidSize, position: Vec2, velocity: Vec2)` — 충돌 분열에서 재사용
  - `asteroid::AsteroidPlugin` — `OnEnter(Playing)`에서 `spawn_initial_wave`, `Update`(Playing 한정)에 `wave_control`, `draw_asteroids`

- [ ] **Step 1: 실패하는 테스트 작성 — asteroid.rs 하단**

```rust
// src/asteroid.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_is_closed_loop() {
        let pts = asteroid_shape(40.0);
        assert!(pts.len() >= 4);
        // 닫힌 외곽선: 첫 점 == 마지막 점
        assert_eq!(pts.first().unwrap(), pts.last().unwrap());
    }

    #[test]
    fn shape_points_near_radius() {
        let radius = 40.0;
        let pts = asteroid_shape(radius);
        for p in &pts {
            let len = p.length();
            // 반지름의 0.75~1.15배 범위 안에 있어야 함
            assert!(len >= radius * 0.7 && len <= radius * 1.2, "len={len}");
        }
    }

    #[test]
    fn spawn_asteroid_creates_entity_with_size() {
        let mut app = App::new();
        {
            let world = app.world_mut();
            let mut commands_queue = bevy::ecs::world::CommandQueue::default();
            let mut commands = Commands::new(&mut commands_queue, world);
            spawn_asteroid(&mut commands, AsteroidSize::Medium, Vec2::ZERO, Vec2::X);
            commands_queue.apply(world);
        }
        let mut q = app.world_mut().query::<&Asteroid>();
        let sizes: Vec<_> = q.iter(app.world()).map(|a| a.size).collect();
        assert_eq!(sizes, vec![AsteroidSize::Medium]);
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 추가**

`src/main.rs`에 `mod asteroid;` 추가.

- [ ] **Step 3: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `asteroid_shape`, `spawn_asteroid`, `Asteroid` 미정의.

- [ ] **Step 4: asteroid.rs 구현 (테스트 모듈 위에)**

```rust
// src/asteroid.rs
use bevy::prelude::*;
use rand::RngExt; // rand 0.10: random_range는 RngExt 트레이트에 있음 (Rng 아님)

use crate::components::{Collider, Velocity, Wrapping};
use crate::config::{
    ASTEROID_MAX_SPEED, ASTEROID_MIN_SPEED, HALF_HEIGHT, HALF_WIDTH, INITIAL_ASTEROIDS,
};
use crate::logic::AsteroidSize;
use crate::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

#[derive(Component)]
pub struct AsteroidShape {
    pub points: Vec<Vec2>,
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_initial_wave)
            .add_systems(
                Update,
                (wave_control, draw_asteroids).run_if(in_state(GameState::Playing)),
            );
    }
}

/// 반지름을 무작위로 흔든 닫힌 다각형 외곽선(로컬 좌표)을 만든다.
pub fn asteroid_shape(radius: f32) -> Vec<Vec2> {
    let mut rng = rand::rng();
    let sides = 10;
    let mut points: Vec<Vec2> = Vec::with_capacity(sides + 1);
    for i in 0..sides {
        let angle = i as f32 / sides as f32 * std::f32::consts::TAU;
        let r = radius * rng.random_range(0.75..1.15);
        points.push(Vec2::new(angle.cos() * r, angle.sin() * r));
    }
    let first = points[0];
    points.push(first); // 닫기
    points
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
) {
    commands.spawn((
        Asteroid { size },
        AsteroidShape { points: asteroid_shape(size.radius()) },
        Transform::from_translation(position.extend(0.0)),
        Velocity(velocity),
        Collider { radius: size.radius() },
        Wrapping,
        GameplayEntity,
    ));
}

fn random_spawn_position() -> Vec2 {
    let mut rng = rand::rng();
    let pos = Vec2::new(
        rng.random_range(-HALF_WIDTH..HALF_WIDTH),
        rng.random_range(-HALF_HEIGHT..HALF_HEIGHT),
    );
    // 중심(우주선 스폰 위치) 근처면 바깥으로 밀어 즉사 방지
    if pos.length() < 150.0 {
        pos.normalize_or_zero() * 200.0
    } else {
        pos
    }
}

fn random_velocity() -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED);
    Vec2::new(angle.cos(), angle.sin()) * speed
}

fn spawn_wave(commands: &mut Commands) {
    for _ in 0..INITIAL_ASTEROIDS {
        spawn_asteroid(
            commands,
            AsteroidSize::Large,
            random_spawn_position(),
            random_velocity(),
        );
    }
}

fn spawn_initial_wave(mut commands: Commands) {
    spawn_wave(&mut commands);
}

fn wave_control(mut commands: Commands, asteroids: Query<(), With<Asteroid>>) {
    if asteroids.iter().count() == 0 {
        spawn_wave(&mut commands);
    }
}

fn draw_asteroids(mut gizmos: Gizmos, query: Query<(&Transform, &AsteroidShape)>) {
    for (transform, shape) in &query {
        let points = shape
            .points
            .iter()
            .map(|p| transform.transform_point(p.extend(0.0)).truncate());
        gizmos.linestrip_2d(points, Color::WHITE);
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록**

```rust
        .add_plugins(asteroid::AsteroidPlugin)
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test`
Expected: PASS — 다각형/스폰 3개 포함 전체 통과.

- [ ] **Step 7: 실행해 소행성 확인 (수동)**

Run: `cargo run`
Expected: 울퉁불퉁한 흰색 다각형 소행성 4개가 떠다니며 화면을 순환한다. 확인 후 종료.

- [ ] **Step 8: 커밋**

```bash
git add src/asteroid.rs src/main.rs
git commit -m "feat: 소행성 웨이브 스폰·다각형 생성·렌더·재생성"
```

---

## Task 8: 충돌 — 분열/점수 & 목숨/게임오버

**Files:**
- Create: `src/collision.rs`
- Modify: `src/main.rs` (모듈 선언 + `CollisionPlugin` 등록)

**Interfaces:**
- Consumes: `components::Collider`, `logic::{circles_overlap, next_asteroid_size}`, `state::{GameState, GameplayEntity, Score, Lives}`, `asteroid::{Asteroid, spawn_asteroid}`, `bullet::Bullet`, `player::{Player, spawn_player_entity}`
- Produces:
  - `collision::CollisionPlugin` — `Update`(Playing 한정)에 `bullet_vs_asteroid`, `player_vs_asteroid`
  - 시스템(모듈 내부): 위 두 시스템

- [ ] **Step 1: 실패하는 테스트 작성 — collision.rs 하단**

```rust
// src/collision.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Collider;
    use crate::logic::AsteroidSize;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn bullet_splits_large_asteroid_and_scores() {
        let mut app = App::new();
        app.insert_resource(Score(0));
        let bullet = app
            .world_mut()
            .spawn((
                Bullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Collider { radius: 2.0 },
            ))
            .id();
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(bullet_vs_asteroid).unwrap();

        // 총알 소멸
        assert!(app.world().get_entity(bullet).is_err());
        // 점수 증가
        assert_eq!(app.world().resource::<Score>().0, AsteroidSize::Large.score());
        // 큰 소행성 → 중간 2개
        let mut q = app.world_mut().query::<&Asteroid>();
        let sizes: Vec<_> = q.iter(app.world()).map(|a| a.size).collect();
        assert_eq!(sizes.len(), 2);
        assert!(sizes.iter().all(|s| *s == AsteroidSize::Medium));
    }

    #[test]
    fn small_asteroid_vanishes_without_children() {
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

        let mut q = app.world_mut().query::<&Asteroid>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn player_hit_loses_life_and_respawns() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin); // 0.19: bare App엔 StateTransition 스케줄 없음 → init_state 전 필요
        app.init_state::<GameState>();
        app.insert_resource(Lives(3));
        app.world_mut().spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 12.0 },
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(player_vs_asteroid).unwrap();

        assert_eq!(app.world().resource::<Lives>().0, 2);
        let mut q = app.world_mut().query::<&Player>();
        assert_eq!(q.iter(app.world()).count(), 1); // 리스폰됨
    }

    #[test]
    fn last_life_triggers_game_over() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin); // 0.19: bare App엔 StateTransition 스케줄 없음 → init_state 전 필요
        app.init_state::<GameState>();
        app.insert_resource(Lives(1));
        app.world_mut().spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 12.0 },
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(player_vs_asteroid).unwrap();

        assert_eq!(app.world().resource::<Lives>().0, 0);
        assert!(matches!(
            app.world().resource::<NextState<GameState>>(),
            NextState::Pending(GameState::GameOver)
        ));
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 추가**

`src/main.rs`에 `mod collision;` 추가.

- [ ] **Step 3: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `bullet_vs_asteroid`, `player_vs_asteroid` 미정의.

- [ ] **Step 4: collision.rs 구현 (테스트 모듈 위에)**

```rust
// src/collision.rs
use std::collections::HashSet;

use bevy::prelude::*;

use crate::asteroid::{spawn_asteroid, Asteroid};
use crate::bullet::Bullet;
use crate::components::Collider;
use crate::logic::{circles_overlap, next_asteroid_size};
use crate::player::{spawn_player_entity, Player};
use crate::state::{GameState, Lives, Score};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (bullet_vs_asteroid, player_vs_asteroid).run_if(in_state(GameState::Playing)),
        );
    }
}

fn bullet_vs_asteroid(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    asteroids: Query<(Entity, &Transform, &Collider, &Asteroid)>,
) {
    let mut spent_bullets: HashSet<Entity> = HashSet::new();
    let mut destroyed: HashSet<Entity> = HashSet::new();

    for (bullet_entity, bullet_tf, bullet_col) in &bullets {
        if spent_bullets.contains(&bullet_entity) {
            continue;
        }
        for (asteroid_entity, asteroid_tf, asteroid_col, asteroid) in &asteroids {
            if destroyed.contains(&asteroid_entity) {
                continue;
            }
            if circles_overlap(
                bullet_tf.translation.truncate(),
                bullet_col.radius,
                asteroid_tf.translation.truncate(),
                asteroid_col.radius,
            ) {
                spent_bullets.insert(bullet_entity);
                destroyed.insert(asteroid_entity);
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();
                score.0 += asteroid.size.score();

                if let Some(next) = next_asteroid_size(asteroid.size) {
                    let base = asteroid_tf.translation.truncate();
                    for dir in [1.0_f32, -1.0] {
                        let velocity = Vec2::new(dir * 70.0, dir * 45.0);
                        spawn_asteroid(&mut commands, next, base, velocity);
                    }
                }
                break; // 이 총알은 소진됨
            }
        }
    }
}

fn player_vs_asteroid(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    asteroids: Query<(&Transform, &Collider), With<Asteroid>>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (asteroid_tf, asteroid_col) in &asteroids {
        if circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            asteroid_tf.translation.truncate(),
            asteroid_col.radius,
        ) {
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

- [ ] **Step 5: main.rs에 플러그인 등록**

```rust
        .add_plugins(collision::CollisionPlugin)
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test`
Expected: PASS — 충돌 4개 포함 전체 통과.

- [ ] **Step 7: 실행해 충돌 확인 (수동)**

Run: `cargo run`
Expected: 총알로 큰 소행성을 맞히면 중간 2개로 쪼개지고, 다시 맞히면 작은 2개, 작은 건 사라진다. 우주선이 소행성에 닿으면 사라졌다 중앙에 다시 나타난다(목숨 소모). 모두 부수면 새 웨이브가 뜬다. 확인 후 종료.

- [ ] **Step 8: 커밋**

```bash
git add src/collision.rs src/main.rs
git commit -m "feat: 충돌 처리 — 소행성 분열/점수와 목숨/게임오버"
```

---

## Task 9: UI — HUD·게임오버·재시작 (게임 루프 완성)

**Files:**
- Create: `src/ui.rs`
- Modify: `src/main.rs` (모듈 선언 + `UiPlugin` 등록)

**Interfaces:**
- Consumes: `state::{GameState, GameplayEntity, Score, Lives}`
- Produces:
  - `ui::UiPlugin` — `OnEnter(Playing)`에서 `spawn_hud`, `Update`(Playing)에서 `update_hud`, `OnEnter(GameOver)`에서 `spawn_game_over`, `OnExit(GameOver)`에서 `despawn_game_over`, `Update`(GameOver)에서 `restart_input`
  - 시스템(모듈 내부): `update_hud(Res<Score>, Res<Lives>, Query<&mut Text, With<Hud>>)`

- [ ] **Step 1: 실패하는 테스트 작성 — ui.rs 하단**

```rust
// src/ui.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn hud_shows_score_and_lives() {
        let mut app = App::new();
        app.insert_resource(Score(150));
        app.insert_resource(Lives(2));
        let e = app.world_mut().spawn((Hud, Text::new("초기값"))).id();
        app.world_mut().run_system_once(update_hud).unwrap();
        let text = app.world().entity(e).get::<Text>().unwrap();
        assert!(text.0.contains("150"));
        assert!(text.0.contains('2'));
    }
}
```

- [ ] **Step 2: main.rs에 모듈 선언 추가**

`src/main.rs`에 `mod ui;` 추가.

- [ ] **Step 3: 테스트 실행해 실패 확인**

Run: `cargo test`
Expected: FAIL — `Hud`, `update_hud` 미정의.

- [ ] **Step 4: ui.rs 구현 (테스트 모듈 위에)**

```rust
// src/ui.rs
use bevy::prelude::*;
use bevy::text::FontSize;

use crate::state::{GameState, GameplayEntity, Lives, Score};

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct GameOverScreen;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(Update, restart_input.run_if(in_state(GameState::GameOver)));
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Hud,
        GameplayEntity,
        Text::new("Score: 0   Lives: 3"),
        TextFont { font_size: FontSize::Px(24.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn update_hud(score: Res<Score>, lives: Res<Lives>, mut query: Query<&mut Text, With<Hud>>) {
    for mut text in &mut query {
        text.0 = format!("Score: {}   Lives: {}", score.0, lives.0);
    }
}

fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        GameOverScreen,
        Text::new(format!("GAME OVER\nScore: {}\nPress R to restart", score.0)),
        TextFont { font_size: FontSize::Px(40.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(34.0),
            ..default()
        },
    ));
}

fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn restart_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록**

```rust
        .add_plugins(ui::UiPlugin)
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test`
Expected: PASS — HUD 포함 전체 통과.

- [ ] **Step 7: 전체 게임 루프 확인 (수동)**

Run: `cargo run`
Expected: 좌상단에 "Score / Lives" 표시가 실시간 갱신. 목숨 0이 되면 화면 중앙에 "GAME OVER / Score / Press R" 문구가 뜨고, 소행성/우주선/총알/HUD가 사라진다. **R** 을 누르면 점수·목숨이 초기화되고 새 판이 시작된다. 확인 후 종료.

- [ ] **Step 8: 커밋 & 푸시**

```bash
git add src/ui.rs src/main.rs
git commit -m "feat: HUD·게임오버 화면·재시작으로 게임 루프 완성"
git push
```

---

## Self-Review (작성자 점검 결과)

**1. Spec coverage** — 설계 문서 각 요구사항 대응:
- 우주선 회전/추진(관성) → Task 5. 총알 발사 → Task 6. 소행성 분열(대→중→소, 소 소멸) → Task 7·8. 화면 순환 → Task 3. 충돌 목숨 감소·목숨 3 → Task 4·8. 점수 UI → Task 9. 게임오버·재시작 → Task 9. 벡터 라인 렌더 → Gizmos(Task 5·6·7). 순수 함수 단위 테스트(wrap/split/overlap) → Task 2. 완료 기준(DoD) 전 항목 대응됨.
- 설계 문서 대비 유일한 구현상 변경: 렌더링을 "Mesh LineStrip" → "Gizmos 라인"으로 조정(시각 결과 동일, 파이프라인 견고성 이유). Global Constraints에 명시.

**2. Placeholder scan** — "TBD/TODO/적절히 처리" 등 플레이스홀더 없음. 모든 스텝에 실제 코드/명령/기대값 포함.

**3. Type consistency** — 태스크 간 시그니처 일치 확인: `AsteroidSize`(logic) → asteroid/collision에서 동일 사용, `spawn_asteroid`/`spawn_player_entity`/`next_asteroid_size`/`circles_overlap`/`wrap_position` 이름·인자 일관. 컴포넌트 `Velocity`/`Collider`/`Wrapping`/`GameplayEntity` 전 모듈 동일.

**알려진 단순화(YAGNI, 범위 밖):** 우주선 리스폰 시 무적 시간 없음(중앙 근처에 소행성이 있으면 연속 피격 가능) — 초기 소행성은 중심에서 밀어내 스폰하므로 초반엔 드묾. 향후 확장 후보(무적, UFO, 하이퍼스페이스, 사운드, 최고점수)는 설계 문서의 "범위 밖" 절 참조.

---

## Verified Bevy 0.19 API (실제 소스로 확인한 근거)

이 계획의 Bevy 호출은 아래 시그니처를 기준으로 하며, 모두 `v0.19.0` 태그 공식 예제/docs.rs로 검증했다.

- 카메라: `commands.spawn(Camera2d);`
- 입력: `Res<ButtonInput<KeyCode>>`, `.pressed(KeyCode::X)`, `.just_pressed(KeyCode::X)`; `KeyCode::{ArrowUp, ArrowLeft, ArrowRight, Space, KeyR}`; 테스트에서 `ButtonInput::press`.
- 시간: `Res<Time>`, `time.delta_secs() -> f32`, `time.delta() -> Duration`; 고정 스텝 `insert_resource(Time::<Fixed>::from_hz(60.0))`; 테스트에서 `Time::<()>::default()` + `time.advance_by(Duration)`.
- 변환: `transform.rotate_z(f32)`, `transform.rotation * Vec3::Y`(정면), `transform.transform_point(Vec3) -> Vec3`, `Vec3::truncate()/Vec2::extend()`.
- 상태: `#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]`, `app.init_state::<T>()`, `OnEnter(S)/OnExit(S)`, `.run_if(in_state(S))`, `ResMut<NextState<T>>::set(...)`, `NextState::Pending(_)`.
- Gizmos: 시스템 파라미터 `mut gizmos: Gizmos`; `gizmos.linestrip_2d(impl IntoIterator<Item = Vec2>, impl Into<Color>)`; `gizmos.circle_2d(Isometry2d::from_translation(Vec2), f32, impl Into<Color>)`.
- UI 텍스트(0.19 변경점): `Text::new(String)`(내부 `Text(pub String)` → `text.0` 로 갱신), `TextFont { font_size: FontSize::Px(f32), ..default() }`(0.19에서 `font_size`가 `FontSize` enum), `TextColor(Color)`, `Node { position_type: PositionType::Absolute, top/left: Val::Px|Val::Percent, ..default() }`. `use bevy::text::FontSize;`.
- 엔티티: `commands.spawn((...,))`, `commands.entity(e).despawn()`(0.19 기본 재귀), `Query::single() -> Result<_, _>`.
- 테스트 유틸: `use bevy::ecs::system::RunSystemOnce;` → `world.run_system_once(system).unwrap()`(커맨드 자동 flush); `app.world().get_entity(e).is_err()`로 소멸 확인.
- rand 0.10: `use rand::Rng;` `let mut rng = rand::rng();` `rng.random_range(a..b)`, `rng.random_bool(p)`.
