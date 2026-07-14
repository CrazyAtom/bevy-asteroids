# Phase 7 — 전자기폭풍 테마(EM Storm) + 테슬라 코어 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 테마 풀에 전자기폭풍(EM Storm) 테마와 테슬라 코어 보스를 추가한다. EM Storm 스테이지에선 화면이 어둠으로 덮이고 우주선 주변만 밝은 원형 시야창이 생기며, 주기적 폭풍 피크에 시야창이 좁아졌다 회복한다.

**Architecture:** 안개는 순수 시각 효과 — 방사형 그라데이션 오버레이 스프라이트(`fx/fog.rs`)가 우주선을 추적하고, `storm_vision_scale(t)` 순수 함수로 시야창 크기를 진동시킨다. on/off는 `FogState`(리소스)로 제어하며, 진행 테마에 따라 `sync_fog_state`가 켠다(물리 `StageModifiers`는 미개입). 보스는 기존 `BossKind` 분기에 `TeslaCore`(블링크 이동 + EMP 방사 링 + 조준 체인 전격)를 더한다.

**Tech Stack:** Rust 2021 / Bevy 0.19 / rand 0.10. 스프라이트는 `assets/sprites/src/*.svg` → `build.rs`(resvg) 자동 래스터화(radialGradient 지원).

## Global Constraints

- **Bevy 0.19 API**: 메시지는 `MessageWriter`, 단일 쿼리는 `query.single()`(Result), 리소스는 플러그인 `build()`에서 확보.
- **비(非)EM-Storm 스테이지 동작 불변**: `FogState` 기본 off → 안개 없음, 기존과 100% 동일.
- **재사용**: 보스 탄은 기존 `spawn_enemy_bullet` 재사용.
- **바이너리 크레이트 dead_code**: 미사용 `pub` 항목/필드/상수는 `clippy -D warnings`에서 에러. 각 상수·함수는 **소비처가 있는 태스크에서** 추가한다(이 플랜은 그렇게 배치됨).
- **테스트**: 순수 로직만 테스트. 기존 79개 테스트 계속 통과. 각 태스크 후 `cargo test`(전부 통과) + `cargo clippy --all-targets -- -D warnings`(경고 0).
- **Git**: 커밋 메시지 한국어, 말미에 `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. 브랜치 `feat/phase7-emstorm-theme`.
- **상수 값(확정)**: `STORM_PERIOD=7.0`, `FOG_VISION_MIN=0.6`, `Z_FOG=20.0`, `FOG_BASE_SIZE=5200.0`, `TESLA_HEALTH_MUL=1.1`, 테슬라 반경 `56.0`, `TESLA_ATTACK_INTERVAL=1.8`, `BLINK_INTERVAL=3.0`, EMP 링 `14`발, 체인 `3`갈래(±0.25rad), `theme_params(EmStorm)={count:0.85, speed:0.9, ufo:1.0}`.

---

## File Structure

- `src/core/config.rs` — 안개/폭풍/테슬라 상수(각 소비 태스크에서 추가).
- `src/core/logic.rs` — `storm_pulse`/`storm_vision_scale` 순수 함수.
- `src/fx/fog.rs`(신규) + `src/fx.rs` — `FogState`, `FogOverlay`, `FogPlugin`, `update_fog`.
- `src/main.rs` — `FogPlugin` 등록.
- `src/systems/stage.rs` — `ThemeId::EmStorm`, 풀 5, `theme_params`/`theme_name`/`stage_modifiers` arm, `sync_fog_state`.
- `src/entities/boss.rs` — `BossKind::TeslaCore` + 각 arm, 블링크 이동, EMP/체인 공격.
- `src/fx/sprites.rs` — `fog`/`bg_storm`/`boss_tesla_core` 핸들.
- `src/fx/background.rs` — `theme_bg` EmStorm 분기.
- `assets/sprites/src/{fog,bg_storm,boss_tesla_core}.svg` — 신규 아트.

---

## Task 1: 안개 인프라 (순수 함수 + fog 모듈, 기본 off)

이 태스크는 안개 렌더링 전체를 깔되 **FogState 기본 off**라 어디서도 안개가 보이지 않는다(동작 불변). `FogState`는 리소스라 테마를 몰라도 동작하므로 `ThemeId` 변형 없이 완결된다.

**Files:**
- Modify: `src/core/config.rs`, `src/core/logic.rs`, `src/fx/sprites.rs`, `src/fx.rs`, `src/main.rs`
- Create: `src/fx/fog.rs`, `assets/sprites/src/fog.svg`
- Test: `src/core/logic.rs`, `src/fx/fog.rs`

**Interfaces:**
- Produces: `storm_pulse(f32)->f32`, `storm_vision_scale(f32)->f32`, `FogState{active:bool}`(리소스), `FogOverlay`(마커), `FogPlugin`.

- [ ] **Step 1: 상수 추가**

`src/core/config.rs`의 Phase 6 블록(파일 끝) 아래에 추가:

```rust
// Phase 7 — 전자기폭풍(안개 시야 제한)
pub const STORM_PERIOD: f32 = 7.0;       // 폭풍 피크 주기(초)
pub const FOG_VISION_MIN: f32 = 0.6;     // 피크 시 시야 배율(1.0=평소)
pub const Z_FOG: f32 = 20.0;             // 안개 오버레이 Z(모든 월드 스프라이트 위)
pub const FOG_BASE_SIZE: f32 = 5200.0;   // 안개 오버레이 기본 크기(px). 최소 배율에서도 화면 구석까지 덮음
```

- [ ] **Step 2: 실패 테스트 작성(순수 함수)**

`src/core/logic.rs`의 `mod tests`에 추가:

```rust
#[test]
fn storm_pulse_in_unit_range_with_a_peak() {
    let mut saw_peak = false;
    for i in 0..200 {
        let t = i as f32 * 0.05;
        let p = storm_pulse(t);
        assert!((0.0..=1.0).contains(&p));
        if p > 0.5 {
            saw_peak = true;
        }
    }
    assert!(saw_peak, "주기 안에 폭풍 피크가 존재해야 한다");
}

#[test]
fn storm_vision_scale_between_min_and_one() {
    for i in 0..200 {
        let t = i as f32 * 0.05;
        let s = storm_vision_scale(t);
        assert!(s <= 1.0 + 1e-6);
        assert!(s >= crate::core::config::FOG_VISION_MIN - 1e-6);
    }
}
```

- [ ] **Step 3: 테스트 실패 확인**

Run: `cargo test storm_ 2>&1 | tail -15`
Expected: `storm_pulse`/`storm_vision_scale` 미정의로 컴파일 에러.

- [ ] **Step 4: 순수 함수 구현**

`src/core/logic.rs`의 `reflect_edge` 아래(또는 파일 하단 순수 함수 구역)에 추가:

```rust
/// 폭풍 강도 펄스: 평소 0, 주기(STORM_PERIOD)마다 부드럽게 1로 치솟았다 가라앉음. [0,1].
pub fn storm_pulse(t: f32) -> f32 {
    use std::f32::consts::TAU;
    let phase = t / crate::core::config::STORM_PERIOD;
    (TAU * phase).sin().max(0.0).powi(4)
}

/// 시야 배율: 평소 1.0, 폭풍 피크에서 FOG_VISION_MIN까지 축소. [FOG_VISION_MIN, 1.0].
pub fn storm_vision_scale(t: f32) -> f32 {
    1.0 - (1.0 - crate::core::config::FOG_VISION_MIN) * storm_pulse(t)
}
```

- [ ] **Step 5: 순수 함수 테스트 통과 확인**

Run: `cargo test storm_ 2>&1 | tail -6`
Expected: 두 테스트 통과.

- [ ] **Step 6: 안개 SVG 작성**

`assets/sprites/src/fog.svg` 생성(방사형 그라데이션: 중심 투명 → 8% 투명 유지 → 16% 검정 → 가장자리 검정):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <defs>
    <radialGradient id="fog" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#000000" stop-opacity="0"/>
      <stop offset="8%" stop-color="#000000" stop-opacity="0"/>
      <stop offset="16%" stop-color="#000000" stop-opacity="1"/>
      <stop offset="100%" stop-color="#000000" stop-opacity="1"/>
    </radialGradient>
  </defs>
  <rect width="256" height="256" fill="url(#fog)"/>
</svg>
```

- [ ] **Step 7: `SpriteAssets`에 `fog` 필드 추가**

`src/fx/sprites.rs`:

구조체에 `boss_ice_golem` 아래(또는 마지막 핸들 필드 근처) 추가:

```rust
    pub boss_ice_golem: Handle<Image>,
    pub fog: Handle<Image>,
```

`build_sprite_assets`의 해당 위치에 추가:

```rust
        boss_ice_golem: asset_server.load("sprites/boss_ice_golem.png"),
        fog: asset_server.load("sprites/fog.png"),
```

`dummy_sprite_assets`의 해당 위치에 추가:

```rust
        boss_ice_golem: h.clone(),
        fog: h.clone(),
```

> `boss_ice_golem` 정확한 위치는 파일에서 확인하고 그 바로 아래에 `fog`를 넣는다(순서만 일치하면 됨).

- [ ] **Step 8: fog 모듈 작성**

`src/fx/fog.rs` 생성:

```rust
//! 시야 제한 안개(전자기폭풍 테마). 방사형 그라데이션 오버레이가 우주선을 따라다니며
//! 원형 시야창을 만든다. on/off는 `FogState`가 제어하고(테마에 무관), 진행 테마에 따라
//! `systems::stage::sync_fog_state`가 켠다. 시야창 크기는 폭풍 피크에 맞춰 진동한다.

use bevy::prelude::*;

use crate::core::config::{FOG_BASE_SIZE, Z_FOG};
use crate::core::logic::storm_vision_scale;
use crate::entities::player::Player;
use crate::fx::sprites::SpriteAssets;

/// 안개 활성 여부. 기본 off(안개 없음). 진행 테마가 EM Storm일 때만 켜진다.
#[derive(Resource, Default)]
pub struct FogState {
    pub active: bool,
}

#[derive(Component)]
struct FogOverlay;

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FogState>()
            .add_systems(Startup, spawn_fog)
            .add_systems(Update, update_fog);
    }
}

fn spawn_fog(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        FogOverlay,
        Sprite {
            image: assets.fog.clone(),
            custom_size: Some(Vec2::splat(FOG_BASE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_FOG),
        Visibility::Hidden,
    ));
}

/// 활성 시 오버레이를 우주선에 맞추고 시야창 크기를 폭풍 피크로 진동시킨다. 비활성 시 숨김.
fn update_fog(
    time: Res<Time>,
    fog_state: Res<FogState>,
    players: Query<&Transform, (With<Player>, Without<FogOverlay>)>,
    mut q: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<FogOverlay>>,
) {
    let Ok((mut tf, mut sprite, mut vis)) = q.single_mut() else { return };
    if !fog_state.active {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Visible;
    if let Ok(ship) = players.single() {
        tf.translation.x = ship.translation.x;
        tf.translation.y = ship.translation.y;
    }
    let scale = storm_vision_scale(time.elapsed_secs());
    sprite.custom_size = Some(Vec2::splat(FOG_BASE_SIZE * scale));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_state_defaults_off() {
        assert!(!FogState::default().active);
    }
}
```

- [ ] **Step 9: fog 모듈 등록**

`src/fx.rs`에 모듈 선언 추가(알파벳 순서 유지, 예: `effects` 다음):

```rust
pub mod fog;
```

`src/main.rs`의 플러그인 등록부에 추가(다른 fx 플러그인 근처):

```rust
        .add_plugins(fx::fog::FogPlugin)
```

- [ ] **Step 10: 빌드·테스트·클리피 확인**

Run: `cargo build 2>&1 | tail -5 && ls assets/sprites/fog.png && cargo test 2>&1 | tail -5 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: fog.png 생성, 전부 통과, 경고 0. (FogState off → 안개 안 보임, 동작 불변.)

- [ ] **Step 11: 커밋**

```bash
git add src/core/config.rs src/core/logic.rs src/fx/fog.rs src/fx.rs src/main.rs src/fx/sprites.rs assets/sprites/src/fog.svg
git commit -m "feat: 안개 시야 제한 인프라(fog 모듈 + storm 순수 함수, 기본 off)

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 2: EM Storm 테마 + 테슬라 코어 turn-on (플레이 가능)

`ThemeId::EmStorm`과 `BossKind::TeslaCore`를 함께 도입(exhaustive match 때문에 분리 불가). 완료 시 EM Storm 스테이지가 등장하고 안개가 켜지며 테슬라 코어가 블링크·EMP·체인으로 공격한다. 폭풍-동기 EMP와 블링크 예고는 Task 3에서.

**Files:**
- Modify: `src/core/config.rs`, `src/systems/stage.rs`, `src/entities/boss.rs`, `src/fx/sprites.rs`, `src/fx/background.rs`
- Create: `assets/sprites/src/bg_storm.svg`, `assets/sprites/src/boss_tesla_core.svg`

**Interfaces:**
- Consumes: `FogState`(Task 1), `fog`/`storm_*`(Task 1).
- Produces: `ThemeId::EmStorm`, `BossKind::TeslaCore`, `sync_fog_state`, `Blink` 컴포넌트.

- [ ] **Step 1: 테슬라 상수 추가**

`src/core/config.rs`의 Phase 7 블록 아래에 추가:

```rust
// Phase 7 — 테슬라 코어 보스
pub const TESLA_HEALTH_MUL: f32 = 1.1;
pub const TESLA_ATTACK_INTERVAL: f32 = 1.8;
pub const BLINK_INTERVAL: f32 = 3.0; // 순간이동 주기(초)
```

- [ ] **Step 2: 아트 SVG 작성**

`assets/sprites/src/bg_storm.svg`(기존 `bg_*.svg`와 같은 viewBox 규격 확인 후, 어두운 뇌운·번개):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180">
  <rect width="320" height="180" fill="#0a0a18"/>
  <rect width="320" height="180" fill="#1a1630" opacity="0.5"/>
  <g stroke="#9fb8ff" stroke-width="1.5" fill="none" opacity="0.8">
    <polyline points="80,10 70,50 90,55 76,100"/>
    <polyline points="230,20 245,60 225,66 240,110"/>
  </g>
  <g fill="#3a3560" opacity="0.6">
    <ellipse cx="60" cy="30" rx="70" ry="24"/>
    <ellipse cx="250" cy="40" rx="80" ry="26"/>
    <ellipse cx="160" cy="20" rx="60" ry="20"/>
  </g>
</svg>
```

`assets/sprites/src/boss_tesla_core.svg`(정사각 viewBox, 전기 코어/구체, 카툰):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <g stroke="#0a1a3a" stroke-width="4" stroke-linejoin="round">
    <circle cx="50" cy="50" r="30" fill="#4a6cff"/>
    <circle cx="50" cy="50" r="16" fill="#bcd4ff"/>
  </g>
  <g stroke="#eaf2ff" stroke-width="2.5" fill="none" opacity="0.95">
    <polyline points="50,20 44,34 56,38 48,50"/>
    <polyline points="50,80 56,66 44,62 52,50"/>
    <polyline points="20,50 34,44 38,56 50,50"/>
    <polyline points="80,50 66,56 62,44 50,50"/>
  </g>
</svg>
```

> viewBox는 기존 `bg_belt.svg`/`boss_blazing_core.svg`와 동일 규격을 쓴다(다르면 그 값으로).

- [ ] **Step 3: `SpriteAssets`에 `bg_storm`/`boss_tesla_core` 추가**

`src/fx/sprites.rs`의 구조체·`build_sprite_assets`·`dummy_sprite_assets`에 각각 추가(`fog` 근처):

구조체:
```rust
    pub fog: Handle<Image>,
    pub bg_storm: Handle<Image>,
    pub boss_tesla_core: Handle<Image>,
```
build:
```rust
        fog: asset_server.load("sprites/fog.png"),
        bg_storm: asset_server.load("sprites/bg_storm.png"),
        boss_tesla_core: asset_server.load("sprites/boss_tesla_core.png"),
```
dummy:
```rust
        fog: h.clone(),
        bg_storm: h.clone(),
        boss_tesla_core: h.clone(),
```

- [ ] **Step 4: 실패 테스트 작성(stage + boss)**

`src/systems/stage.rs`의 `mod tests`에 추가:

```rust
#[test]
fn em_storm_params_are_softened() {
    let p = theme_params(ThemeId::EmStorm);
    assert!(p.count_mul < 1.0 && p.speed_mul < 1.0);
}
```

`src/entities/boss.rs`의 `mod tests`에 추가:

```rust
#[test]
fn tesla_core_is_em_storm_boss() {
    assert_eq!(boss_for_theme(ThemeId::EmStorm), BossKind::TeslaCore);
    assert!(boss_max_health(BossKind::TeslaCore, 0) > 0.0);
}
```

- [ ] **Step 5: stage.rs — EmStorm 변형 + 파라미터 + 안개 동기화**

`src/systems/stage.rs`:

import에 `FogState`, `GameState` 추가(이미 `GameState`가 있으면 `FogState`만):

```rust
use crate::fx::fog::FogState;
```

`ThemeId`에 변형 추가:

```rust
pub enum ThemeId {
    AsteroidBelt,
    AlienFleet,
    SolarFlare,
    FrozenField,
    EmStorm,
}
```

`THEME_POOL` 확장(길이 5):

```rust
pub const THEME_POOL: [ThemeId; 5] = [
    ThemeId::AsteroidBelt,
    ThemeId::AlienFleet,
    ThemeId::SolarFlare,
    ThemeId::FrozenField,
    ThemeId::EmStorm,
];
```

`theme_params` match arm 추가:

```rust
        ThemeId::EmStorm => ThemeParams { count_mul: 0.85, speed_mul: 0.9, ufo_interval_mul: 1.0 },
```

`theme_name` match arm 추가:

```rust
        ThemeId::EmStorm => "EM STORM",
```

`stage_modifiers` match: EmStorm은 물리 트위스트 없음이므로 별도 arm 불필요(`_ => StageModifiers::default()`가 처리). 확인만.

`sync_stage_modifiers` 함수 아래에 안개 동기화 시스템 추가:

```rust
/// EM Storm 스테이지에서만 안개를 켠다(Playing 중에만). 상태 비의존 동기화.
fn sync_fog_state(
    state: Res<State<GameState>>,
    prog: Res<Progression>,
    mut fog: ResMut<FogState>,
) {
    fog.active = *state.get() == GameState::Playing && prog.current_theme() == ThemeId::EmStorm;
}
```

`StagePlugin::build`의 Update 튜플에 `sync_fog_state` 추가. 단, `sync_fog_state`는 GameOver에서도 꺼야 하므로 `run_if(in_state(Playing))`에 두지 말고 **별도로 상시 실행**한다:

```rust
        .add_systems(
            Update,
            (stage_control, sync_stage_modifiers).run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, sync_fog_state);
```

> `use crate::core::state::{GameState, Lives};`에 `State`는 bevy prelude로 이미 사용 가능. `Res<State<GameState>>` 사용.

- [ ] **Step 6: boss.rs — TeslaCore 변형 + 모든 arm + 블링크/공격**

`src/entities/boss.rs`:

import에 config 상수와 컴포넌트 추가:

config import 블록에 `BLINK_INTERVAL, TESLA_ATTACK_INTERVAL, TESLA_HEALTH_MUL`를 추가.

`BossKind`에 변형 추가:

```rust
pub enum BossKind {
    MotherRock,
    Mothership,
    BlazingCore,
    IceGolem,
    TeslaCore,
}
```

`boss_for_theme` arm:

```rust
        ThemeId::FrozenField => BossKind::IceGolem,
        ThemeId::EmStorm => BossKind::TeslaCore,
```

`boss_max_health` arm:

```rust
        BossKind::IceGolem => BOSS_BASE_HEALTH * ICE_GOLEM_HEALTH_MUL,
        BossKind::TeslaCore => BOSS_BASE_HEALTH * TESLA_HEALTH_MUL,
```

`boss_image` arm:

```rust
        BossKind::IceGolem => assets.boss_ice_golem.clone(),
        BossKind::TeslaCore => assets.boss_tesla_core.clone(),
```

`boss_radius` arm:

```rust
        BossKind::IceGolem => 66.0,
        BossKind::TeslaCore => 56.0,
```

`attack_interval` arm:

```rust
        BossKind::IceGolem => GOLEM_ATTACK_INTERVAL,
        BossKind::TeslaCore => TESLA_ATTACK_INTERVAL,
```

`Blink` 컴포넌트 정의(`BossAttack` 정의 근처에 추가):

```rust
/// 테슬라 코어 순간이동 타이머.
#[derive(Component)]
pub struct Blink {
    pub timer: Timer,
}
```

`spawn_boss`에서 TeslaCore일 때 `Blink` 삽입(골렘 `EdgeReflect` 분기 아래에 추가):

```rust
    if kind == BossKind::IceGolem {
        e.insert((Velocity(Vec2::new(GOLEM_DRIFT_SPEED, GOLEM_DRIFT_SPEED * 0.55)), EdgeReflect));
    }
    if kind == BossKind::TeslaCore {
        e.insert(Blink { timer: Timer::from_seconds(BLINK_INTERVAL, TimerMode::Repeating) });
    }
```

`boss_movement`의 match에 TeslaCore arm 추가(블링크는 별도 시스템이 처리하므로 여기선 조작 없음):

```rust
            // 테슬라 코어: 블링크(boss_blink)가 위치를 옮김. 여기선 조작 없음.
            BossKind::TeslaCore => {}
```

`boss_attack`의 match에 TeslaCore arm 추가(shots 패리티로 EMP 방사 ↔ 조준 체인 교대):

```rust
            // 테슬라 코어: EMP 방사 링 ↔ 조준 체인 전격 교대.
            BossKind::TeslaCore => {
                if shots.is_multiple_of(2) {
                    // EMP 방사 링 14발
                    let n = 14;
                    for i in 0..n {
                        let ang = i as f32 / n as f32 * std::f32::consts::TAU;
                        let d = Vec2::new(ang.cos(), ang.sin());
                        spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                    }
                } else if let Some(pp) = player_pos {
                    // 조준 체인 전격 3갈래
                    let dir = aim_direction(pos, pp);
                    for a in [-0.25f32, 0.0, 0.25] {
                        let d = (Quat::from_rotation_z(a) * dir.extend(0.0)).truncate();
                        spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                    }
                }
            }
```

`boss_combat` 아래(또는 `golem_phase_transition` 근처)에 블링크 시스템 추가:

```rust
/// 테슬라 코어 블링크: 타이머마다 화면 내 임의 위치로 순간이동.
fn boss_blink(time: Res<Time>, mut q: Query<(&mut Transform, &mut Blink), With<Boss>>) {
    use rand::RngExt;
    for (mut tf, mut blink) in &mut q {
        blink.timer.tick(time.delta());
        if blink.timer.is_finished() {
            let mut rng = rand::rng();
            tf.translation.x = rng.random_range(-crate::core::config::HALF_WIDTH * 0.8..crate::core::config::HALF_WIDTH * 0.8);
            tf.translation.y = rng.random_range(0.0..crate::core::config::HALF_HEIGHT * 0.7);
        }
    }
}
```

`BossPlugin::build`의 Update 튜플에 `boss_blink` 추가:

```rust
            (boss_movement, boss_attack, boss_combat, golem_phase_transition, boss_entrance, boss_blink)
                .run_if(in_state(GameState::Playing)),
```

- [ ] **Step 7: background.rs — EM Storm 배경 분기**

`src/fx/background.rs`의 `theme_bg` match arm 추가:

```rust
        ThemeId::FrozenField => assets.bg_ice.clone(),
        ThemeId::EmStorm => assets.bg_storm.clone(),
```

- [ ] **Step 8: 빌드·테스트·클리피 확인**

Run: `cargo build 2>&1 | tail -5 && ls assets/sprites/bg_storm.png assets/sprites/boss_tesla_core.png && cargo test 2>&1 | tail -6 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: PNG 2개 생성, 전부 통과(신규 테마/보스 테스트 포함), 경고 0.

- [ ] **Step 9: 커밋**

```bash
git add src/core/config.rs src/systems/stage.rs src/entities/boss.rs src/fx/sprites.rs src/fx/background.rs assets/sprites/src/bg_storm.svg assets/sprites/src/boss_tesla_core.svg
git commit -m "feat: 전자기폭풍(EM Storm) 테마 + 테슬라 코어 보스 도입

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 3: 폭풍-동기 EMP + 블링크 예고 (연출 정교화)

**Files:**
- Modify: `src/core/config.rs`, `src/entities/boss.rs`
- Test: `src/entities/boss.rs`

**Interfaces:**
- Consumes: `storm_pulse`(Task 1), `Blink`/TeslaCore 공격(Task 2).
- Produces: `storm_emp_triggers` 판정, 블링크 예고 섬광.

- [ ] **Step 1: 상수 추가**

`src/core/config.rs`의 Phase 7 테슬라 블록 아래에 추가:

```rust
pub const STORM_EMP_THRESHOLD: f32 = 0.6; // storm_pulse가 이 값을 상향 돌파하면 EMP 발동
pub const BLINK_TELEGRAPH_SECS: f32 = 0.4; // 블링크 직전 예고(축소) 시간
```

- [ ] **Step 2: 실패 테스트 작성**

`src/entities/boss.rs`의 `mod tests`에 추가:

```rust
#[test]
fn emp_triggers_on_rising_edge_only() {
    // 임계 아래→위로 오를 때만 true(상승 엣지), 이미 위이거나 내려갈 땐 false.
    assert!(storm_emp_triggers(0.5, 0.7));  // 상승 돌파
    assert!(!storm_emp_triggers(0.7, 0.8)); // 이미 위
    assert!(!storm_emp_triggers(0.8, 0.5)); // 하강
    assert!(!storm_emp_triggers(0.3, 0.5)); // 아래 유지
}
```

- [ ] **Step 3: 테스트 실패 확인**

Run: `cargo test emp_triggers 2>&1 | tail -8`
Expected: `storm_emp_triggers` 미정의로 컴파일 에러.

- [ ] **Step 4: 폭풍-동기 EMP + 블링크 예고 구현**

`src/core/config.rs` import는 이미 됨. `src/entities/boss.rs`:

config import에 `STORM_EMP_THRESHOLD`, `BLINK_TELEGRAPH_SECS` 추가. `use crate::core::logic::...`에 `storm_pulse` 추가.

`attack_interval` 근처(순수 함수 구역)에 판정 함수 추가:

```rust
/// storm_pulse가 임계를 상향 돌파하는 순간만 true(EMP 1회 발동용 상승 엣지).
pub fn storm_emp_triggers(prev_pulse: f32, cur_pulse: f32) -> bool {
    prev_pulse < STORM_EMP_THRESHOLD && cur_pulse >= STORM_EMP_THRESHOLD
}
```

`boss_attack`의 TeslaCore arm을 **체인 전격 전용**으로 바꾸고(EMP는 폭풍-동기로 분리), EMP는 별도 시스템에서 발동한다. 우선 TeslaCore arm을 교체:

```rust
            // 테슬라 코어: 조준 체인 전격(EMP는 폭풍 피크에 동기화되어 storm_emp에서 발동).
            BossKind::TeslaCore => {
                if let Some(pp) = player_pos {
                    let dir = aim_direction(pos, pp);
                    for a in [-0.25f32, 0.0, 0.25] {
                        let d = (Quat::from_rotation_z(a) * dir.extend(0.0)).truncate();
                        spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                    }
                }
            }
```

`boss_blink` 아래에 EMP 시스템 추가(폭풍 피크 상승 엣지에 방사 링 + 화면 흔들림):

```rust
/// 폭풍 피크(storm_pulse 상승 엣지)에 테슬라 코어가 EMP 방사 링을 쏜다.
fn storm_emp(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    time: Res<Time>,
    mut prev: Local<f32>,
    mut shake: MessageWriter<ShakeEvent>,
    bosses: Query<&Transform, With<Boss>>,
) {
    let cur = crate::core::logic::storm_pulse(time.elapsed_secs());
    let fire = storm_emp_triggers(*prev, cur);
    *prev = cur;
    if !fire {
        return;
    }
    let mut fired = false;
    for tf in &bosses {
        let pos = tf.translation.truncate();
        let n = 14;
        for i in 0..n {
            let ang = i as f32 / n as f32 * std::f32::consts::TAU;
            let d = Vec2::new(ang.cos(), ang.sin());
            spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
        }
        fired = true;
    }
    if fired {
        shake.write(ShakeEvent(SHAKE_EXPLOSION));
    }
}
```

> `storm_emp`는 모든 Boss를 대상으로 하지만, EM Storm 스테이지에는 테슬라 코어만 등장하므로 실질적으로 테슬라 전용이다. 다른 테마 보스전에서는 폭풍이 없으니(안개 off) 발동해도 무방하나, 명확성을 위해 쿼리에 `TeslaCore`만 걸고 싶으면 kind 체크를 추가한다. 여기서는 간결성을 위해 전체 Boss 대상으로 두되, storm_pulse는 EM Storm이 아니어도 계산되므로 **다른 보스전에서도 EMP가 나갈 수 있음** → kind 가드를 추가한다:

수정(테슬라만):

```rust
    bosses: Query<(&Boss, &Transform)>,
) {
    ...
    for (boss, tf) in &bosses {
        if boss.kind != BossKind::TeslaCore { continue; }
        ...
    }
```

`BossPlugin::build`의 Update 튜플에 `storm_emp` 추가:

```rust
            (boss_movement, boss_attack, boss_combat, golem_phase_transition, boss_entrance, boss_blink, storm_emp)
                .run_if(in_state(GameState::Playing)),
```

블링크 예고: `boss_blink`에서 타이머 잔여가 `BLINK_TELEGRAPH_SECS` 미만이면 스프라이트를 살짝 축소(예고)하고, 순간이동 직후 원복한다. `boss_blink`를 수정:

```rust
fn boss_blink(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut Sprite, &mut Blink), With<Boss>>,
) {
    use rand::RngExt;
    let base = boss_radius(BossKind::TeslaCore) * 2.0;
    for (mut tf, mut sprite, mut blink) in &mut q {
        blink.timer.tick(time.delta());
        // 예고: 순간이동 직전 축소
        let remain = blink.timer.remaining_secs();
        let scale = if remain < BLINK_TELEGRAPH_SECS { 0.6 } else { 1.0 };
        sprite.custom_size = Some(Vec2::splat(base * scale));
        if blink.timer.is_finished() {
            tf.translation.x = rng_x();
            tf.translation.y = rng_y();
            sprite.custom_size = Some(Vec2::splat(base)); // 원복
        }
    }
}
```

> `rng_x()`/`rng_y()`는 위 Task 2의 인라인 난수 코드를 그대로 쓰거나, 간결히 인라인 유지한다(별도 헬퍼로 빼지 않아도 됨). 핵심은 예고 축소 → 이동 → 원복.

- [ ] **Step 5: 테스트·클리피 통과 확인**

Run: `cargo test 2>&1 | tail -6 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: 전부 통과(신규 `emp_triggers` 포함), 경고 0.

- [ ] **Step 6: 실기 확인(선택, 사람 검증)**

Run: `cargo run` — EM Storm 스테이지 진입 시 안개·시야창·폭풍 피크, 테슬라 코어 블링크·체인·폭풍 동기 EMP 확인 후 종료.

- [ ] **Step 7: 커밋**

```bash
git add src/core/config.rs src/entities/boss.rs
git commit -m "feat: 테슬라 코어 폭풍-동기 EMP + 블링크 예고

$(printf 'Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Self-Review (작성자 체크)

- **스펙 커버리지**: 안개 시야 제한(T1) / 진행·풀5·테마 파라미터(T2) / 테슬라 블링크·EMP·체인(T2,T3) / 폭풍-동기 EMP(T3) / 아트(T1 fog, T2 bg·boss) / 배경(T2) — 스펙 §3 전 항목 태스크 존재. ✓
- **플레이스홀더**: 모든 스텝에 실제 코드/명령/기대값. 상수는 Global Constraints에 확정. ✓
- **dead_code 회피**: 상수는 소비 태스크에서 추가(STORM_PERIOD·FOG_*=T1, TESLA_*=T2, STORM_EMP_*=T3). T1의 `storm_vision_scale`은 같은 태스크의 `update_fog`가 소비, `storm_pulse`는 `storm_vision_scale`이 소비 → 임시 allow 불필요. 두 변형(EmStorm·TeslaCore)은 T2에서 함께 도입되어 즉시 생성처 확보. ✓
- **타입 일관성**: `FogState`는 T1 정의, T2 `sync_fog_state`가 기록. `Blink`는 T2 정의, T3에서 예고에 사용. `storm_emp_triggers`는 T3 정의·사용. `storm_pulse`는 T1 정의, T3 `storm_emp`가 사용. ✓
- **회귀**: `FogState` 기본 off → 비 EM-Storm 동작 불변. `sync_fog_state`가 Playing且EmStorm에서만 켜고 그 외/GameOver엔 끔. 기존 보스 4종 arm 미변경. ✓
- **주의**: T2에서 boss.rs config import에 테슬라 상수 추가 시 기존 import 목록과 병합(중복 방지). `sync_fog_state`는 `run_if(Playing)` 밖 상시 실행(GameOver에서 끄기 위함).
