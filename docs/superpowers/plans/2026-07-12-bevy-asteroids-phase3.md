# Bevy Asteroids Phase 3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Phase 2 게임에 사운드·파워업·확장형 특수무기·배경 별·화면 흔들림·하이퍼스페이스를 추가하고, `src/`를 도메인 폴더로 재구성한다.

**Architecture:** 먼저 `src/`를 `core/entities/systems/fx` 폴더로 재구성한다(단일 크레이트 유지). 이후 신규 플러그인(background/shake/audio/powerup)을 추가하고 기존 모듈을 확장한다. 순수 로직은 `core/logic.rs`(및 각 모듈의 순수 함수)에 두고 TDD. 렌더는 Gizmos, 사운드는 build.rs가 생성한 WAV + `AudioPlayer`.

**Tech Stack:** Rust 2021 · Bevy `0.19`(+`wav` feature) · rand `0.10` · bevy-persistent `0.11` · build.rs(WAV 합성)

## Global Constraints

- Bevy `0.19`, rand `0.10`. Phase 1·2 검증 API 재사용(`ButtonInput`/`Res<Time>`/`transform`/States/Gizmos/`Timer::is_finished`/`rand::rng()`+`RngExt`/`run_system_once`/`Time::<()>::default()`/상태 테스트 `StatesPlugin`).
- **오디오**: `bevy = { version = "0.19", features = ["wav"] }` (WAV 재생에 필요). 재생 = `commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN))`; 로드 = `asset_server.load::<AudioSource>("sounds/<name>.wav")`.
- **이벤트**: `#[derive(Message)] struct X;` + `app.add_message::<X>()` + `MessageWriter<X>`/`MessageReader<X>`.
- 렌더는 Gizmos만(사운드/오디오 제외). 게임 진행 중 스폰 엔티티(파워업·빔 등)는 `GameplayEntity` 부여. UFO/적총알/파티클/파워업/빔은 `Wrapping` 없음. 별은 `GameplayEntity` 아님(항상 표시).
- 커밋 메시지 한국어.
- **폴더 재구성(Task 1) 이후 모든 import 경로는 그룹 접두사**를 쓴다: `crate::core::{config,logic,components,state}`, `crate::entities::{player,bullet,asteroid,ufo,powerup}`, `crate::systems::{movement,collision}`, `crate::fx::{effects,background,shake,audio}`, `crate::ui`.

---

## File Structure (Task 1 이후)

```
src/
├── main.rs              # mod core; entities; systems; fx; ui; 플러그인 조립, 카메라, HighScore
├── core.rs              # pub mod config; logic; components; state;
│   core/{config,logic,components,state}.rs
├── entities.rs          # pub mod player; bullet; asteroid; ufo; powerup;
│   entities/{player,bullet,asteroid,ufo,powerup}.rs
├── systems.rs           # pub mod movement; collision;
│   systems/{movement,collision}.rs
├── fx.rs                # pub mod effects; background; shake; audio;
│   fx/{effects,background,shake,audio}.rs
├── ui.rs
build.rs                 # assets/sounds/*.wav 생성
```

---

## Task 1: 폴더 재구성 (기능 변화 없음)

**Files:** 이동 — `src/{config,logic,components,state}.rs` → `src/core/`; `src/{player,bullet,asteroid,ufo}.rs` → `src/entities/`; `src/{movement,collision}.rs` → `src/systems/`; `src/effects.rs` → `src/fx/`. Create — `src/core.rs`, `src/entities.rs`, `src/systems.rs`, `src/fx.rs`. Modify — `src/main.rs`(mod 선언), 모든 파일의 `use crate::...` 경로.

**Interfaces:**
- Produces: 재구성된 모듈 트리. 공개 항목 이름/시그니처는 불변, 경로만 변경.

- [ ] **Step 1: 파일 이동 (git mv)**

```bash
cd /Users/medit/_dev/00.private/02.test-game
mkdir -p src/core src/entities src/systems src/fx
git mv src/config.rs src/logic.rs src/components.rs src/state.rs src/core/
git mv src/player.rs src/bullet.rs src/asteroid.rs src/ufo.rs src/entities/
git mv src/movement.rs src/collision.rs src/systems/
git mv src/effects.rs src/fx/
```

- [ ] **Step 2: 그룹 루트 모듈 파일 생성**

`src/core.rs`:
```rust
pub mod components;
pub mod config;
pub mod logic;
pub mod state;
```
`src/entities.rs`:
```rust
pub mod asteroid;
pub mod bullet;
pub mod player;
pub mod ufo;
```
`src/systems.rs`:
```rust
pub mod collision;
pub mod movement;
```
`src/fx.rs`:
```rust
pub mod effects;
```

- [ ] **Step 3: main.rs 모듈 선언 교체**

`src/main.rs`의 기존 `mod config; mod logic; ...` 개별 선언들을 다음으로 교체(그리고 플러그인 등록/함수의 경로도 새 경로로):
```rust
mod core;
mod entities;
mod fx;
mod systems;
mod ui;
```
`main()`의 `.add_plugins(...)` 및 `setup_high_score`/`use` 경로를 새 경로로 갱신(예: `state::GameStatePlugin` → `core::state::GameStatePlugin`, `movement::MovementPlugin` → `systems::movement::MovementPlugin`, `player::PlayerPlugin` → `entities::player::PlayerPlugin`, `effects::EffectsPlugin` → `fx::effects::EffectsPlugin`, `ufo::UfoPlugin` → `entities::ufo::UfoPlugin`, `collision::CollisionPlugin` → `systems::collision::CollisionPlugin`, `ui::UiPlugin` 유지, `state::HighScore` → `core::state::HighScore`).

- [ ] **Step 4: 전 파일의 `use crate::...` 경로 갱신**

모든 `use crate::config::` → `use crate::core::config::`, `crate::logic` → `crate::core::logic`, `crate::components` → `crate::core::components`, `crate::state` → `crate::core::state`, `crate::player` → `crate::entities::player`, `crate::bullet` → `crate::entities::bullet`, `crate::asteroid` → `crate::entities::asteroid`, `crate::ufo` → `crate::entities::ufo`, `crate::movement` → `crate::systems::movement`, `crate::collision` → `crate::systems::collision`, `crate::effects` → `crate::fx::effects`. (테스트 모듈의 `use crate::...` 도 동일.)

찾기 도움: `grep -rn "crate::\(config\|logic\|components\|state\|player\|bullet\|asteroid\|ufo\|movement\|collision\|effects\)" src/`

- [ ] **Step 5: 빌드·테스트로 무결성 확인**

Run: `cargo test`
Expected: 이동 전과 동일하게 **전체 통과**(테스트 수 불변, 예: 45개). 실패 시 남은 경로 오류 수정.
Run: `cargo clippy --all-targets -- -D warnings`
Expected: 경고 0.

- [ ] **Step 6: 커밋**

```bash
git add -A
git commit -m "refactor: src를 core/entities/systems/fx 도메인 폴더로 재구성"
```

---

## Task 2: 배경 별 (fx/background.rs)

**Files:** Create `src/fx/background.rs`; Modify `src/fx.rs`(`pub mod background;`), `src/main.rs`(플러그인 등록), `src/core/config.rs`(상수).

**Interfaces:**
- Produces: `background::BackgroundPlugin`. `Star { phase: f32, base_brightness: f32 }`(Component). Startup에 별 스폰, `draw_stars`가 반짝임 렌더.
- Consumes: `config::{STAR_COUNT, TWINKLE_SPEED, HALF_WIDTH, HALF_HEIGHT}`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
// Phase 3 — 배경
pub const STAR_COUNT: usize = 120;
pub const TWINKLE_SPEED: f32 = 2.0;
```

- [ ] **Step 2: 실패 테스트 — background.rs 하단**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn spawns_configured_star_count() {
        let mut app = App::new();
        app.world_mut().run_system_once(spawn_stars).unwrap();
        let mut q = app.world_mut().query::<&Star>();
        assert_eq!(q.iter(app.world()).count(), STAR_COUNT);
    }
}
```

- [ ] **Step 3: fx.rs에 `pub mod background;` 추가 후 실패 확인** — Run: `cargo test spawns_configured_star_count` → FAIL(`Star`/`spawn_stars` 미정의).

- [ ] **Step 4: background.rs 구현 (테스트 모듈 위)**

```rust
use bevy::prelude::*;
use rand::RngExt;

use crate::core::config::{HALF_HEIGHT, HALF_WIDTH, STAR_COUNT, TWINKLE_SPEED};

#[derive(Component)]
pub struct Star {
    pub phase: f32,
    pub base_brightness: f32,
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_stars)
            .add_systems(Update, draw_stars);
    }
}

fn spawn_stars(mut commands: Commands) {
    let mut rng = rand::rng();
    for _ in 0..STAR_COUNT {
        let x = rng.random_range(-HALF_WIDTH..HALF_WIDTH);
        let y = rng.random_range(-HALF_HEIGHT..HALF_HEIGHT);
        commands.spawn((
            Star {
                phase: rng.random_range(0.0..std::f32::consts::TAU),
                base_brightness: rng.random_range(0.3..1.0),
            },
            Transform::from_xyz(x, y, 0.0),
        ));
    }
}

fn draw_stars(mut gizmos: Gizmos, time: Res<Time>, query: Query<(&Transform, &Star)>) {
    let t = time.elapsed_secs();
    for (transform, star) in &query {
        let b = star.base_brightness * (0.5 + 0.5 * (t * TWINKLE_SPEED + star.phase).sin());
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            1.0,
            Color::srgb(b, b, b),
        );
    }
}
```

- [ ] **Step 5: main.rs에 플러그인 등록** — `.add_plugins(fx::background::BackgroundPlugin)`.

- [ ] **Step 6: 통과 확인 + 빌드** — Run: `cargo test` → PASS. `cargo build` → 성공. (수동: 별밭 반짝임은 `cargo run` 확인)

- [ ] **Step 7: 커밋**

```bash
git add src/fx/background.rs src/fx.rs src/main.rs src/core/config.rs
git commit -m "feat: 반짝이는 별 배경"
```

---

## Task 3: 화면 흔들림 (fx/shake.rs)

**Files:** Create `src/fx/shake.rs`; Modify `src/fx.rs`, `src/main.rs`, `src/core/config.rs`, `src/core/logic.rs`(순수 함수), `src/systems/collision.rs`(트리거).

**Interfaces:**
- Produces: `shake::ShakePlugin`, `#[derive(Resource, Default)] ScreenShake { trauma: f32 }`, `#[derive(Message)] ShakeEvent(pub f32)`. `apply_screen_shake`가 카메라를 흔든다.
- Produces(logic): `logic::decay_trauma(trauma: f32, dt: f32) -> f32`.
- Consumes: `config::{MAX_SHAKE_OFFSET, SHAKE_DECAY, SHAKE_HIT, SHAKE_EXPLOSION}`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
// Phase 3 — 화면 흔들림
pub const MAX_SHAKE_OFFSET: f32 = 18.0;
pub const SHAKE_DECAY: f32 = 1.5;
pub const SHAKE_HIT: f32 = 0.6;
pub const SHAKE_EXPLOSION: f32 = 0.25;
pub const SHAKE_SPECIAL: f32 = 0.5;
```

- [ ] **Step 2: 실패 테스트 — logic.rs tests에 추가**

```rust
    #[test]
    fn trauma_decays_to_zero_not_below() {
        let t = decay_trauma(0.5, 0.1);
        assert!(t < 0.5 && t >= 0.0);
        assert_eq!(decay_trauma(0.05, 100.0), 0.0); // 큰 dt여도 음수 아님
    }
```

- [ ] **Step 3: 실패 확인** — Run: `cargo test trauma_decays` → FAIL.

- [ ] **Step 4: logic.rs에 순수 함수 추가**

```rust
/// trauma를 dt만큼 감쇠(0 미만으로 내려가지 않음).
pub fn decay_trauma(trauma: f32, dt: f32) -> f32 {
    (trauma - crate::core::config::SHAKE_DECAY * dt).max(0.0)
}
```

- [ ] **Step 5: shake.rs 구현**

```rust
use bevy::prelude::*;
use rand::RngExt;

use crate::core::config::MAX_SHAKE_OFFSET;
use crate::core::logic::decay_trauma;

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

#[derive(Message)]
pub struct ShakeEvent(pub f32);

pub struct ShakePlugin;

impl Plugin for ShakePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .add_message::<ShakeEvent>()
            .add_systems(Update, apply_screen_shake);
    }
}

fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut events: MessageReader<ShakeEvent>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    for ShakeEvent(amount) in events.read() {
        shake.trauma = (shake.trauma + amount).clamp(0.0, 1.0);
    }
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let intensity = shake.trauma * shake.trauma;
    if intensity > 0.0 {
        let mut rng = rand::rng();
        transform.translation.x = rng.random_range(-1.0..1.0) * MAX_SHAKE_OFFSET * intensity;
        transform.translation.y = rng.random_range(-1.0..1.0) * MAX_SHAKE_OFFSET * intensity;
    } else {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
    shake.trauma = decay_trauma(shake.trauma, time.delta_secs());
}
```

- [ ] **Step 6: fx.rs에 `pub mod shake;`, main.rs에 `.add_plugins(fx::shake::ShakePlugin)`**

- [ ] **Step 7: collision.rs에서 흔들림 트리거**

collision.rs 상단에 `use crate::fx::shake::ShakeEvent;`, `use crate::core::config::{SHAKE_EXPLOSION, SHAKE_HIT};` 추가. 두 시스템에 `mut shake: MessageWriter<ShakeEvent>` 파라미터를 추가하고:
- `bullet_vs_asteroid`/`bullet_vs_ufo`의 폭발 지점에서 `shake.write(ShakeEvent(SHAKE_EXPLOSION));`
- `player_damage`의 피격 처리에서 `shake.write(ShakeEvent(SHAKE_HIT));`

(MessageWriter 메서드는 0.19에서 `write`. 만약 컴파일 오류면 설치 소스 확인 후 올바른 메서드(`send`)로 대체하고 보고.)

- [ ] **Step 8: 통과 확인 + 빌드** — Run: `cargo test` → PASS. `cargo build` → 성공.

- [ ] **Step 9: 커밋**

```bash
git add src/fx/shake.rs src/fx.rs src/main.rs src/core/config.rs src/core/logic.rs src/systems/collision.rs
git commit -m "feat: 피격·폭발 시 화면 흔들림"
```

---

## Task 4: 사운드 인프라 (build.rs + fx/audio.rs)

**Files:** Create `build.rs`, `src/fx/audio.rs`; Modify `Cargo.toml`(`features=["wav"]`), `.gitignore`, `src/fx.rs`, `src/main.rs`, 그리고 소리 트리거를 붙이는 `entities/bullet.rs`·`entities/ufo.rs`·`systems/collision.rs`·`core/state.rs`.

**Interfaces:**
- Produces: `#[derive(Message)] SfxEvent(pub Sfx)`, `enum Sfx { Fire, Explosion, Thrust(사용 안 함 가능), UfoFire, Pickup, Special, Hyperspace, GameOver }`, `audio::AudioPlugin`, `SfxAssets` 리소스, `play_sfx` 시스템.
- 소리를 내는 시스템은 `MessageWriter<SfxEvent>`로 이벤트만 발행.

- [ ] **Step 1: Cargo.toml — wav feature + build 스크립트 인지**

`Cargo.toml`의 `bevy = "0.19"` 를 다음으로:
```toml
bevy = { version = "0.19", features = ["wav"] }
```
build.rs는 크레이트 루트에 두면 cargo가 자동 인식(별도 설정 불필요).

- [ ] **Step 2: .gitignore에 생성물 추가**

`.gitignore`에 추가:
```
/assets/sounds/*.wav
```

- [ ] **Step 3: build.rs 작성 (자립형 WAV 합성)**

`build.rs` (크레이트 루트):
```rust
use std::f32::consts::TAU;
use std::fs;
use std::io::Write;
use std::path::Path;

const SR: u32 = 22_050;

fn main() {
    let dir = Path::new("assets/sounds");
    fs::create_dir_all(dir).unwrap();
    // (이름, 지속시간초, 생성기)
    write_wav(dir, "fire.wav", synth(0.15, |t| (600.0 - 400.0 * t / 0.15) , 0.4, false));
    write_wav(dir, "explosion.wav", synth(0.4, |_| 0.0, 0.6, true));
    write_wav(dir, "ufo_fire.wav", synth(0.2, |t| 300.0 + 200.0 * (t * 30.0).sin(), 0.3, false));
    write_wav(dir, "pickup.wav", synth(0.2, |t| 500.0 + 600.0 * t / 0.2, 0.3, false));
    write_wav(dir, "special.wav", synth(0.5, |t| 200.0 + 100.0 * (t * 8.0).sin(), 0.5, true));
    write_wav(dir, "hyperspace.wav", synth(0.3, |t| 800.0 - 700.0 * t / 0.3, 0.3, false));
    write_wav(dir, "game_over.wav", synth(0.6, |t| 300.0 - 200.0 * t / 0.6, 0.4, false));
    println!("cargo:rerun-if-changed=build.rs");
}

/// dur초 동안 freq(t)Hz 톤(+옵션 노이즈)에 선형 감쇠 엔벨로프를 씌운 PCM(i16) 샘플 생성.
fn synth(dur: f32, freq: impl Fn(f32) -> f32, amp: f32, noise: bool) -> Vec<i16> {
    let n = (SR as f32 * dur) as usize;
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    let mut seed = 0x1234_5678u32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = (1.0 - t / dur).max(0.0);
        let f = freq(t);
        phase += TAU * f / SR as f32;
        let tone = phase.sin();
        let sample = if noise {
            // xorshift 노이즈
            seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
            let nz = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
            0.5 * tone + 0.5 * nz
        } else {
            tone
        };
        out.push((sample * env * amp * i16::MAX as f32) as i16);
    }
    out
}

fn write_wav(dir: &Path, name: &str, samples: Vec<i16>) {
    let path = dir.join(name);
    let data_len = (samples.len() * 2) as u32;
    let mut f = fs::File::create(&path).unwrap();
    // RIFF/WAVE 헤더 (PCM, mono, 16-bit)
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(36 + data_len).to_le_bytes()).unwrap();
    f.write_all(b"WAVEfmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();       // fmt chunk size
    f.write_all(&1u16.to_le_bytes()).unwrap();        // PCM
    f.write_all(&1u16.to_le_bytes()).unwrap();        // mono
    f.write_all(&SR.to_le_bytes()).unwrap();          // sample rate
    f.write_all(&(SR * 2).to_le_bytes()).unwrap();    // byte rate
    f.write_all(&2u16.to_le_bytes()).unwrap();        // block align
    f.write_all(&16u16.to_le_bytes()).unwrap();       // bits per sample
    f.write_all(b"data").unwrap();
    f.write_all(&data_len.to_le_bytes()).unwrap();
    for s in samples {
        f.write_all(&s.to_le_bytes()).unwrap();
    }
}
```

- [ ] **Step 4: audio.rs 구현 (테스트 없음 — 이벤트/핸들 배선, 재생은 수동 확인)**

```rust
use bevy::prelude::*;

use crate::core::state::GameState;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sfx {
    Fire,
    Explosion,
    UfoFire,
    Pickup,
    Special,
    Hyperspace,
    GameOver,
}

impl Sfx {
    fn file(self) -> &'static str {
        match self {
            Sfx::Fire => "sounds/fire.wav",
            Sfx::Explosion => "sounds/explosion.wav",
            Sfx::UfoFire => "sounds/ufo_fire.wav",
            Sfx::Pickup => "sounds/pickup.wav",
            Sfx::Special => "sounds/special.wav",
            Sfx::Hyperspace => "sounds/hyperspace.wav",
            Sfx::GameOver => "sounds/game_over.wav",
        }
    }
}

#[derive(Message)]
pub struct SfxEvent(pub Sfx);

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SfxEvent>().add_systems(Update, play_sfx);
    }
}

fn play_sfx(mut commands: Commands, asset_server: Res<AssetServer>, mut events: MessageReader<SfxEvent>) {
    for SfxEvent(sfx) in events.read() {
        commands.spawn((
            AudioPlayer::<AudioSource>::new(asset_server.load(sfx.file())),
            PlaybackSettings::DESPAWN,
        ));
    }
}

// GameState는 향후 확장(상태별 음소거 등)을 위해 import 유지
#[allow(unused_imports)]
use GameState as _GameStateUsed;
```
(주: `#[allow(unused_imports)]` 대신 실제로 `GameState`를 쓰지 않으면 import 자체를 제거하라 — 컴파일 경고 0 유지. 위 마지막 두 줄은 삭제하고 `use crate::core::state::GameState;`도 지우는 것을 기본으로 한다.)

- [ ] **Step 5: fx.rs에 `pub mod audio;`, main.rs에 `.add_plugins(fx::audio::AudioPlugin)`**

- [ ] **Step 6: 소리 트리거 배선 (이벤트 발행)**

- `entities/bullet.rs` `fire_bullet`: 발사 성공 시 `sfx.write(SfxEvent(Sfx::Fire))` (시그니처에 `mut sfx: MessageWriter<SfxEvent>` 추가, `use crate::fx::audio::{Sfx, SfxEvent};`).
- `entities/ufo.rs` `ufo_fire`: 발사 시 `SfxEvent(Sfx::UfoFire)`.
- `systems/collision.rs`: 폭발 지점(소행성/UFO 파괴)에서 `SfxEvent(Sfx::Explosion)`.
- `core/state.rs` `save_high_score`(OnEnter GameOver) 근처 또는 별도 OnEnter(GameOver) 시스템에서 `SfxEvent(Sfx::GameOver)` 1회.

각 시스템에 `MessageWriter<SfxEvent>` 파라미터를 추가한다.

- [ ] **Step 7: 빌드·실행 확인** — Run: `cargo build`(build.rs가 `assets/sounds/*.wav` 생성). `ls assets/sounds` 로 7개 wav 확인. `cargo test` → 기존 통과 유지. (수동: `cargo run` 으로 발사·폭발 소리 확인 — 사용자 몫.)

- [ ] **Step 8: 커밋**

```bash
git add build.rs Cargo.toml Cargo.lock .gitignore src/fx/audio.rs src/fx.rs src/main.rs src/entities/bullet.rs src/entities/ufo.rs src/systems/collision.rs src/core/state.rs
git commit -m "feat: 사운드 인프라(생성 WAV) 및 주요 효과음"
```

---

## Task 5: 발사 쿨다운 모델

**Files:** Modify `src/entities/bullet.rs`(또는 player), `src/entities/player.rs`, `src/core/config.rs`.

**Interfaces:**
- Produces: `player::FireCooldown(pub Timer)`(Component, 플레이어에 부여). `fire_bullet`가 `just_pressed` 대신 `pressed`+쿨다운으로 발사.
- Consumes: `config::FIRE_INTERVAL`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
// Phase 3 — 발사 쿨다운
pub const FIRE_INTERVAL: f32 = 0.25;
pub const RAPID_FIRE_INTERVAL: f32 = 0.10;
pub const SPREAD_ANGLE: f32 = 0.26; // rad
```

- [ ] **Step 2: player.rs — FireCooldown 컴포넌트 + 스폰 부여**

`entities/player.rs`에 추가:
```rust
#[derive(Component)]
pub struct FireCooldown(pub Timer);
```
`spawn_player_entity`의 스폰 튜플에 추가(즉시 발사 가능하도록 완료 상태로 시작):
```rust
        FireCooldown({
            let mut t = Timer::from_seconds(crate::core::config::FIRE_INTERVAL, TimerMode::Once);
            t.tick(t.duration()); // 시작 시 준비완료
            t
        }),
```

- [ ] **Step 3: bullet.rs — fire_bullet를 쿨다운 기반으로 교체**

`fire_bullet`를 다음으로 교체(`use crate::entities::player::{FireCooldown, Player};`, `use crate::core::config::FIRE_INTERVAL;`):
```rust
fn fire_bullet(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut query: Query<(&Transform, &mut FireCooldown), With<Player>>,
) {
    let Ok((ship, mut cooldown)) = query.single_mut() else {
        return;
    };
    cooldown.0.tick(time.delta());
    if !keys.pressed(KeyCode::Space) || !cooldown.0.is_finished() {
        return;
    }
    cooldown.0 = Timer::from_seconds(FIRE_INTERVAL, TimerMode::Once);
    let forward = (ship.rotation * Vec3::Y).truncate();
    let nose = ship.translation + (ship.rotation * Vec3::Y) * 18.0;
    commands.spawn((
        Bullet { life: Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once) },
        Transform::from_translation(nose),
        Velocity(forward * BULLET_SPEED),
        Collider { radius: BULLET_COLLIDER_RADIUS },
        Wrapping,
        GameplayEntity,
    ));
    sfx.write(SfxEvent(Sfx::Fire));
}
```
(주: `Player` transform은 이제 `FireCooldown`과 함께 쿼리하므로 기존 `Query<&Transform, With<Player>>`를 위 형태로 대체.)

- [ ] **Step 4: 실패→통과 테스트 — bullet.rs tests**

```rust
    #[test]
    fn fires_only_when_cooldown_ready() {
        use crate::entities::player::{FireCooldown, Player};
        let mut app = App::new();
        app.add_message::<SfxEvent>();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        app.insert_resource(keys);
        // 준비완료 쿨다운
        let mut cd = Timer::from_seconds(0.25, TimerMode::Once);
        cd.tick(std::time::Duration::from_secs_f32(1.0));
        app.world_mut().spawn((Player, Transform::default(), FireCooldown(cd)));
        app.world_mut().run_system_once(fire_bullet).unwrap();
        let mut q = app.world_mut().query::<&Bullet>();
        assert_eq!(q.iter(app.world()).count(), 1); // 한 발
    }
```
Run: `cargo test fires_only_when_cooldown_ready` (구현 후 PASS).

- [ ] **Step 5: 통과 + 빌드 확인** — `cargo test` → PASS. `cargo build` → 성공. (수동: Space 홀드 시 일정 간격 연사 확인)

- [ ] **Step 6: 커밋**

```bash
git add src/entities/bullet.rs src/entities/player.rs src/core/config.rs
git commit -m "feat: 발사를 쿨다운 기반으로 변경(연사/확산탄 대비)"
```

---

## Task 6: 파워업 인프라 + 추가 목숨

**Files:** Create `src/entities/powerup.rs`; Modify `src/entities.rs`, `src/main.rs`, `src/core/config.rs`, `src/systems/collision.rs`(드롭·수집).

**Interfaces:**
- Produces: `powerup::{PowerupKind, Powerup, SpecialWeaponKind}`, `powerup::pick_powerup_kind(roll: f32) -> PowerupKind`(순수), `powerup::spawn_powerup(commands, kind, pos)`, `powerup::PowerupPlugin`. `Powerup`은 `Velocity`+`Collider`+수명+`GameplayEntity`, `Wrapping` 없음.
- Consumes: `config::{POWERUP_LIFETIME_SECS, POWERUP_DRIFT_SPEED, POWERUP_RADIUS, POWERUP_DROP_CHANCE}`, `components::*`, `state::{GameState, GameplayEntity, Lives}`, `fx::audio::{Sfx, SfxEvent}`.

- [ ] **Step 1: config.rs에 상수 추가**

```rust
// Phase 3 — 파워업
pub const POWERUP_DROP_CHANCE: f32 = 0.18;
pub const POWERUP_LIFETIME_SECS: f32 = 8.0;
pub const POWERUP_DRIFT_SPEED: f32 = 30.0;
pub const POWERUP_RADIUS: f32 = 12.0;
```

- [ ] **Step 2: 실패 테스트 — powerup.rs 하단**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn pick_kind_covers_all_buckets() {
        // 경계값이 유효한 종류를 반환하는지(패닉/누락 없음)
        for i in 0..=10 {
            let _ = pick_powerup_kind(i as f32 / 10.0);
        }
        assert!(matches!(pick_powerup_kind(0.0), PowerupKind::Shield));
        // 마지막 버킷은 특수무기
        assert!(matches!(pick_powerup_kind(0.99), PowerupKind::SpecialWeapon(_)));
    }

    #[test]
    fn spawn_powerup_creates_entity() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut c: Commands| {
                spawn_powerup(&mut c, PowerupKind::ExtraLife, Vec2::ZERO);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Powerup>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }
}
```

- [ ] **Step 3: entities.rs에 `pub mod powerup;` 후 실패 확인** — Run: `cargo test -p bevy-asteroids powerup` → FAIL.

- [ ] **Step 4: powerup.rs 구현**

```rust
use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{Collider, Velocity};
use crate::core::config::{POWERUP_DRIFT_SPEED, POWERUP_LIFETIME_SECS, POWERUP_RADIUS};
use crate::core::state::{GameState, GameplayEntity, Lives};
use crate::entities::player::Player;
use crate::fx::audio::{Sfx, SfxEvent};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpecialWeaponKind {
    LaserBeam,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerupKind {
    Shield,
    RapidFire,
    Spread,
    ExtraLife,
    SpecialWeapon(SpecialWeaponKind),
}

#[derive(Component)]
pub struct Powerup {
    pub kind: PowerupKind,
    pub life: Timer,
}

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (powerup_lifetime, draw_powerups, collect_powerup).run_if(in_state(GameState::Playing)),
        );
    }
}

/// roll(0..1)을 5개 버킷으로 나눠 종류를 고른다(특수무기는 낮은 확률).
pub fn pick_powerup_kind(roll: f32) -> PowerupKind {
    match roll {
        r if r < 0.25 => PowerupKind::Shield,
        r if r < 0.50 => PowerupKind::RapidFire,
        r if r < 0.72 => PowerupKind::Spread,
        r if r < 0.88 => PowerupKind::ExtraLife,
        _ => PowerupKind::SpecialWeapon(SpecialWeaponKind::LaserBeam),
    }
}

pub fn spawn_powerup(commands: &mut Commands, kind: PowerupKind, position: Vec2) {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let velocity = Vec2::new(angle.cos(), angle.sin()) * POWERUP_DRIFT_SPEED;
    commands.spawn((
        Powerup { kind, life: Timer::from_seconds(POWERUP_LIFETIME_SECS, TimerMode::Once) },
        Transform::from_translation(position.extend(0.0)),
        Velocity(velocity),
        Collider { radius: POWERUP_RADIUS },
        GameplayEntity,
    ));
}

fn powerup_lifetime(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Powerup)>) {
    for (entity, mut p) in &mut q {
        p.life.tick(time.delta());
        if p.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn powerup_color(kind: PowerupKind) -> Color {
    match kind {
        PowerupKind::Shield => Color::srgb(0.3, 0.6, 1.0),
        PowerupKind::RapidFire => Color::srgb(1.0, 0.8, 0.2),
        PowerupKind::Spread => Color::srgb(0.6, 1.0, 0.4),
        PowerupKind::ExtraLife => Color::srgb(1.0, 0.4, 0.6),
        PowerupKind::SpecialWeapon(_) => Color::srgb(1.0, 0.3, 1.0),
    }
}

fn draw_powerups(mut gizmos: Gizmos, q: Query<(&Transform, &Powerup)>) {
    for (transform, p) in &q {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            POWERUP_RADIUS,
            powerup_color(p.kind),
        );
    }
}

/// 우주선이 파워업에 닿으면 효과를 적용한다. (실드/연사/확산탄/특수무기는 후속 태스크에서 각자 컴포넌트를 부여)
fn collect_powerup(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut sfx: MessageWriter<SfxEvent>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    powerups: Query<(Entity, &Transform, &Collider, &Powerup)>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (powerup_entity, powerup_tf, powerup_col, powerup) in &powerups {
        if crate::core::logic::circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            powerup_tf.translation.truncate(),
            powerup_col.radius,
        ) {
            match powerup.kind {
                PowerupKind::ExtraLife => lives.0 += 1,
                // 나머지 종류는 Task 7·8·9에서 이 match에 팔을 추가해 컴포넌트 부여
                _ => {}
            }
            let _ = player_entity;
            commands.entity(powerup_entity).despawn();
            sfx.write(SfxEvent(Sfx::Pickup));
        }
    }
}
```

- [ ] **Step 5: 드롭 배선 — collision.rs**

`bullet_vs_asteroid`/`bullet_vs_ufo`의 파괴 지점에서 낮은 확률로 드롭. collision.rs 상단에 `use crate::entities::powerup::{pick_powerup_kind, spawn_powerup};`, `use crate::core::config::POWERUP_DROP_CHANCE;`, `use rand::RngExt;`(이미 있을 수 있음 — 중복 금지). 각 파괴 직후:
```rust
                {
                    let mut rng = rand::rng();
                    if rng.random_range(0.0..1.0) < POWERUP_DROP_CHANCE {
                        spawn_powerup(&mut commands, pick_powerup_kind(rng.random_range(0.0..1.0)), <파괴위치>.translation.truncate());
                    }
                }
```
(`<파괴위치>`는 소행성은 `asteroid_tf`, UFO는 `ufo_tf`.)

- [ ] **Step 6: main.rs에 `.add_plugins(entities::powerup::PowerupPlugin)`**

- [ ] **Step 7: 통과 확인 + 빌드** — `cargo test` → PASS. `cargo build` → 성공.

- [ ] **Step 8: 커밋**

```bash
git add src/entities/powerup.rs src/entities.rs src/main.rs src/core/config.rs src/systems/collision.rs
git commit -m "feat: 파워업 드롭·수집 인프라와 추가 목숨"
```

---

## Task 7: 실드 파워업

**Files:** Modify `src/entities/player.rs`(Shield 컴포넌트·렌더), `src/entities/powerup.rs`(collect match), `src/systems/collision.rs`(player_damage 조기반환), `src/core/config.rs`.

**Interfaces:**
- Produces: `player::Shield(pub Timer)`(Component). 실드 활성 시 `player_damage` 무피해, 우주선 주위 실드 원 렌더.
- Consumes: `config::SHIELD_SECS`.

- [ ] **Step 1: config.rs — `pub const SHIELD_SECS: f32 = 5.0;`**

- [ ] **Step 2: player.rs — Shield 컴포넌트 + 만료 제거 + 렌더**

```rust
#[derive(Component)]
pub struct Shield(pub Timer);
```
`PlayerPlugin`의 Update(Playing)에 `shield_tick`, `draw_shield` 추가:
```rust
fn shield_tick(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Shield)>) {
    for (entity, mut shield) in &mut q {
        shield.0.tick(time.delta());
        if shield.0.is_finished() {
            commands.entity(entity).remove::<Shield>();
        }
    }
}

fn draw_shield(mut gizmos: Gizmos, q: Query<&Transform, (With<Player>, With<Shield>)>) {
    for transform in &q {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            18.0,
            Color::srgb(0.3, 0.7, 1.0),
        );
    }
}
```

- [ ] **Step 3: powerup.rs collect_powerup의 match에 Shield 팔 추가**

`match powerup.kind` 에 추가(그리고 `use crate::entities::player::Shield;`, `use crate::core::config::SHIELD_SECS;`):
```rust
                PowerupKind::Shield => {
                    commands.entity(player_entity).insert(Shield(Timer::from_seconds(SHIELD_SECS, TimerMode::Once)));
                }
```

- [ ] **Step 4: collision.rs player_damage — 실드 시 조기 반환**

`player_damage` 쿼리에 실드 여부를 넣기 위해, players 쿼리를 `Query<(Entity, &Transform, &Collider, Option<&Shield>), With<Player>>` 로 바꾸고(상단 `use crate::entities::player::Shield;`), 히트 판정 전에:
```rust
    let Ok((player_entity, player_tf, player_col, shield)) = players.single() else { return; };
    if shield.is_some() {
        return; // 실드 중 무피해
    }
```

- [ ] **Step 5: 실패→통과 테스트 — collision.rs tests**

```rust
    #[test]
    fn shielded_player_takes_no_damage() {
        use crate::entities::player::{Player, Shield};
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(Lives(3));
        app.world_mut().spawn((
            Player, Transform::from_xyz(0.0,0.0,0.0), Collider { radius: 12.0 },
            Shield(Timer::from_seconds(5.0, TimerMode::Once)),
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large }, Transform::from_xyz(0.0,0.0,0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));
        app.world_mut().run_system_once(player_damage).unwrap();
        assert_eq!(app.world().resource::<Lives>().0, 3); // 무피해
    }
```
(구현 후 PASS.)

- [ ] **Step 6: 통과 + 빌드** — `cargo test` → PASS. `cargo build` → 성공.

- [ ] **Step 7: 커밋**

```bash
git add src/entities/player.rs src/entities/powerup.rs src/systems/collision.rs src/core/config.rs
git commit -m "feat: 실드 파워업(일시 무적)"
```

---

## Task 8: 연사 + 확산탄 파워업

**Files:** Modify `src/entities/player.rs`(RapidFire/Spread 컴포넌트·틱), `src/entities/bullet.rs`(fire_bullet 반영), `src/entities/powerup.rs`(collect), `src/core/config.rs`.

**Interfaces:**
- Produces: `player::RapidFire(pub Timer)`, `player::Spread(pub Timer)`(Component). `fire_bullet`가 이들을 반영(쿨다운 단축, 3-way).
- Consumes: `config::{RAPID_FIRE_INTERVAL, SPREAD_ANGLE, RAPID_FIRE_SECS, SPREAD_SECS}`.

- [ ] **Step 1: config.rs — `pub const RAPID_FIRE_SECS: f32 = 6.0; pub const SPREAD_SECS: f32 = 6.0;`** (RAPID_FIRE_INTERVAL/SPREAD_ANGLE는 Task 5에서 추가됨)

- [ ] **Step 2: player.rs — 컴포넌트 + 만료 틱**

```rust
#[derive(Component)]
pub struct RapidFire(pub Timer);
#[derive(Component)]
pub struct Spread(pub Timer);
```
`PlayerPlugin` Update(Playing)에 만료 제거 시스템(하나로 묶어도 됨):
```rust
fn tick_fire_mods(
    mut commands: Commands,
    time: Res<Time>,
    mut rapid: Query<(Entity, &mut RapidFire)>,
    mut spread: Query<(Entity, &mut Spread)>,
) {
    for (e, mut t) in &mut rapid {
        t.0.tick(time.delta());
        if t.0.is_finished() { commands.entity(e).remove::<RapidFire>(); }
    }
    for (e, mut t) in &mut spread {
        t.0.tick(time.delta());
        if t.0.is_finished() { commands.entity(e).remove::<Spread>(); }
    }
}
```

- [ ] **Step 3: bullet.rs — fire_bullet에 연사/확산탄 반영**

`fire_bullet` 쿼리를 `Query<(&Transform, &mut FireCooldown, Option<&RapidFire>, Option<&Spread>), With<Player>>` 로 확장(`use crate::entities::player::{FireCooldown, Player, RapidFire, Spread};`, `use crate::core::config::{FIRE_INTERVAL, RAPID_FIRE_INTERVAL, SPREAD_ANGLE};`). 발사 로직 교체:
```rust
    let Ok((ship, mut cooldown, rapid, spread)) = query.single_mut() else { return; };
    cooldown.0.tick(time.delta());
    if !keys.pressed(KeyCode::Space) || !cooldown.0.is_finished() {
        return;
    }
    let interval = if rapid.is_some() { RAPID_FIRE_INTERVAL } else { FIRE_INTERVAL };
    cooldown.0 = Timer::from_seconds(interval, TimerMode::Once);
    let base = ship.rotation * Vec3::Y;
    let nose = ship.translation + base * 18.0;
    let angles: &[f32] = if spread.is_some() { &[-SPREAD_ANGLE, 0.0, SPREAD_ANGLE] } else { &[0.0] };
    for &a in angles {
        let dir = (Quat::from_rotation_z(a) * base).truncate();
        commands.spawn((
            Bullet { life: Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once) },
            Transform::from_translation(nose),
            Velocity(dir * BULLET_SPEED),
            Collider { radius: BULLET_COLLIDER_RADIUS },
            Wrapping,
            GameplayEntity,
        ));
    }
    sfx.write(SfxEvent(Sfx::Fire));
```

- [ ] **Step 4: powerup.rs collect match에 팔 추가**

`use crate::entities::player::{RapidFire, Spread};`, `use crate::core::config::{RAPID_FIRE_SECS, SPREAD_SECS};` 추가 후:
```rust
                PowerupKind::RapidFire => {
                    commands.entity(player_entity).insert(RapidFire(Timer::from_seconds(RAPID_FIRE_SECS, TimerMode::Once)));
                }
                PowerupKind::Spread => {
                    commands.entity(player_entity).insert(Spread(Timer::from_seconds(SPREAD_SECS, TimerMode::Once)));
                }
```

- [ ] **Step 5: 실패→통과 테스트 — bullet.rs tests**

```rust
    #[test]
    fn spread_fires_three_bullets() {
        use crate::entities::player::{FireCooldown, Player, Spread};
        let mut app = App::new();
        app.add_message::<SfxEvent>();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        app.insert_resource(keys);
        let mut cd = Timer::from_seconds(0.25, TimerMode::Once);
        cd.tick(std::time::Duration::from_secs_f32(1.0));
        app.world_mut().spawn((Player, Transform::default(), FireCooldown(cd), Spread(Timer::from_seconds(6.0, TimerMode::Once))));
        app.world_mut().run_system_once(fire_bullet).unwrap();
        let mut q = app.world_mut().query::<&Bullet>();
        assert_eq!(q.iter(app.world()).count(), 3);
    }
```

- [ ] **Step 6: 통과 + 빌드** — `cargo test` → PASS. `cargo build` → 성공.

- [ ] **Step 7: 커밋**

```bash
git add src/entities/player.rs src/entities/bullet.rs src/entities/powerup.rs src/core/config.rs
git commit -m "feat: 연사·확산탄 파워업"
```

---

## Task 9: 특수무기 (레이저빔) — 확장형

**Files:** Modify `src/core/logic.rs`(빔 판정 순수 함수), `src/entities/player.rs`(SpecialWeapon 상태·발동), `src/entities/powerup.rs`(collect), `src/systems/collision.rs`(빔 파괴), `src/core/config.rs`. Create 없음(SpecialBeam는 player.rs).

**Interfaces:**
- Produces: `logic::segment_circle_hit(origin, dir, length, half_width, center, radius) -> bool`(순수). `player::SpecialWeapon { kind: SpecialWeaponKind, charges: u32 }`(Component). `player::SpecialBeam { life: Timer, origin: Vec2, dir: Vec2 }`(Component). 발동 시스템 `activate_special`, 렌더 `draw_special_beam`.
- Consumes: `config::{BEAM_LIFETIME_SECS, BEAM_WIDTH, BEAM_LENGTH, SHAKE_SPECIAL}`, `powerup::SpecialWeaponKind`, `fx::{audio, shake}`.

- [ ] **Step 1: config.rs 상수**

```rust
// Phase 3 — 특수무기
pub const BEAM_LIFETIME_SECS: f32 = 0.4;
pub const BEAM_WIDTH: f32 = 22.0;
pub const BEAM_LENGTH: f32 = 2000.0;
```

- [ ] **Step 2: 실패 테스트 — logic.rs tests**

```rust
    #[test]
    fn beam_hits_target_on_path_not_off() {
        let o = Vec2::ZERO;
        let dir = Vec2::Y;
        // 경로 위(위쪽 100)의 원
        assert!(segment_circle_hit(o, dir, 2000.0, 11.0, Vec2::new(0.0, 100.0), 20.0));
        // 경로에서 멀리 옆
        assert!(!segment_circle_hit(o, dir, 2000.0, 11.0, Vec2::new(200.0, 100.0), 20.0));
        // 뒤쪽(반대 방향)은 안 맞음
        assert!(!segment_circle_hit(o, dir, 2000.0, 11.0, Vec2::new(0.0, -100.0), 20.0));
    }
```

- [ ] **Step 3: 실패 확인 → logic.rs 구현**

```rust
/// origin에서 dir 방향으로 length만큼 뻗는 반폭 half_width 빔이 중심 center·반지름 radius 원과 겹치는지.
pub fn segment_circle_hit(origin: Vec2, dir: Vec2, length: f32, half_width: f32, center: Vec2, radius: f32) -> bool {
    let d = dir.normalize_or_zero();
    let t = (center - origin).dot(d).clamp(0.0, length);
    let closest = origin + d * t;
    closest.distance(center) <= half_width + radius
}
```

- [ ] **Step 4: player.rs — SpecialWeapon 상태 + 스폰 부여 + 발동/렌더**

```rust
use crate::entities::powerup::SpecialWeaponKind;

#[derive(Component)]
pub struct SpecialWeapon {
    pub kind: SpecialWeaponKind,
    pub charges: u32,
}

#[derive(Component)]
pub struct SpecialBeam {
    pub life: Timer,
    pub origin: Vec2,
    pub dir: Vec2,
}
```
`spawn_player_entity` 튜플에 `SpecialWeapon { kind: SpecialWeaponKind::LaserBeam, charges: 0 }` 추가.

발동(X) + 빔 수명·렌더 시스템을 `PlayerPlugin` Update(Playing)에 추가:
```rust
fn activate_special(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<crate::fx::audio::SfxEvent>,
    mut shake: MessageWriter<crate::fx::shake::ShakeEvent>,
    mut query: Query<(&Transform, &mut SpecialWeapon), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let Ok((transform, mut weapon)) = query.single_mut() else { return; };
    if weapon.charges == 0 {
        return;
    }
    weapon.charges -= 1;
    let origin = transform.translation.truncate();
    let dir = (transform.rotation * Vec3::Y).truncate();
    match weapon.kind {
        SpecialWeaponKind::LaserBeam => {
            commands.spawn((
                SpecialBeam { life: Timer::from_seconds(crate::core::config::BEAM_LIFETIME_SECS, TimerMode::Once), origin, dir },
                crate::core::state::GameplayEntity,
            ));
        }
    }
    sfx.write(crate::fx::audio::SfxEvent(crate::fx::audio::Sfx::Special));
    shake.write(crate::fx::shake::ShakeEvent(crate::core::config::SHAKE_SPECIAL));
}

fn tick_and_draw_beam(
    mut commands: Commands,
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut query: Query<(Entity, &mut SpecialBeam)>,
) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let end = beam.origin + beam.dir.normalize_or_zero() * crate::core::config::BEAM_LENGTH;
        // 굵게 보이도록 평행선 여러 개
        for off in [-8.0, -4.0, 0.0, 4.0, 8.0] {
            let perp = Vec2::new(-beam.dir.y, beam.dir.x).normalize_or_zero() * off;
            gizmos.line_2d(beam.origin + perp, end + perp, Color::srgb(1.0, 0.3, 1.0));
        }
    }
}
```
(주: `activate_special`은 빔 엔티티에 `origin/dir`를 저장해 두면, 파괴 판정은 collision에서 이 컴포넌트를 읽어 처리한다.)

- [ ] **Step 5: collision.rs — 빔이 소행성·UFO 파괴**

collision.rs에 시스템 추가(`use crate::entities::player::SpecialBeam;`, `use crate::core::logic::segment_circle_hit;`, `use crate::core::config::{BEAM_LENGTH, BEAM_WIDTH, EXPLOSION_PARTICLES};`, `use crate::entities::asteroid::Asteroid;`, `use crate::entities::ufo::Ufo;`, `use crate::fx::effects::spawn_explosion;`, `use crate::fx::audio::{Sfx, SfxEvent};`):
```rust
fn beam_vs_targets(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut sfx: MessageWriter<SfxEvent>,
    beams: Query<&SpecialBeam>,
    asteroids: Query<(Entity, &Transform, &Collider, &Asteroid)>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo)>,
) {
    for beam in &beams {
        for (e, tf, col, asteroid) in &asteroids {
            if segment_circle_hit(beam.origin, beam.dir, BEAM_LENGTH, BEAM_WIDTH * 0.5, tf.translation.truncate(), col.radius) {
                commands.entity(e).despawn();
                score.0 += asteroid.size.score();
                spawn_explosion(&mut commands, tf.translation.truncate(), EXPLOSION_PARTICLES);
                sfx.write(SfxEvent(Sfx::Explosion));
            }
        }
        for (e, tf, col, ufo) in &ufos {
            if segment_circle_hit(beam.origin, beam.dir, BEAM_LENGTH, BEAM_WIDTH * 0.5, tf.translation.truncate(), col.radius) {
                commands.entity(e).despawn();
                score.0 += ufo.size.score();
                spawn_explosion(&mut commands, tf.translation.truncate(), EXPLOSION_PARTICLES);
                sfx.write(SfxEvent(Sfx::Explosion));
            }
        }
    }
}
```
`CollisionPlugin` Update(Playing) 튜플에 `beam_vs_targets` 추가. (주: 빔은 매 프레임 판정되므로 같은 대상이 여러 프레임 맞을 수 있으나, 첫 프레임에 despawn되어 사라지므로 사실상 1회. 중복 despawn은 0.19에서 경고만.)

- [ ] **Step 6: powerup.rs collect match — 특수무기 충전**

`use crate::entities::player::SpecialWeapon;` 추가 후:
```rust
                PowerupKind::SpecialWeapon(kind) => {
                    if let Ok(mut weapon) = special_q.get_mut(player_entity) {
                        weapon.kind = kind;
                        weapon.charges += 1;
                    }
                }
```
이를 위해 `collect_powerup` 시그니처에 `mut special_q: Query<&mut SpecialWeapon>` 추가.

- [ ] **Step 7: 통과 + 빌드** — `cargo test` → PASS(빔 판정 테스트 포함). `cargo build` → 성공. (수동: X로 빔 발사·적 쓸어버림 확인)

- [ ] **Step 8: 커밋**

```bash
git add src/core/logic.rs src/entities/player.rs src/entities/powerup.rs src/systems/collision.rs src/core/config.rs
git commit -m "feat: 특수무기(레이저빔) 발동·빔 판정·충전"
```

---

## Task 10: 하이퍼스페이스

**Files:** Modify `src/entities/player.rs`, `src/core/config.rs`.

**Interfaces:**
- Produces: `player::HyperspaceCooldown(pub Timer)`(Component), `player::random_hyperspace_position(half: Vec2) -> Vec2`. 발동 시스템 `hyperspace`.
- Consumes: `config::{HYPERSPACE_COOLDOWN_SECS, HALF_WIDTH, HALF_HEIGHT}`, `fx::audio`.

- [ ] **Step 1: config.rs — `pub const HYPERSPACE_COOLDOWN_SECS: f32 = 2.0;`**

- [ ] **Step 2: 실패 테스트 — player.rs tests**

```rust
    #[test]
    fn hyperspace_position_within_bounds() {
        let half = Vec2::new(640.0, 360.0);
        for _ in 0..20 {
            let p = random_hyperspace_position(half);
            assert!(p.x.abs() <= half.x && p.y.abs() <= half.y);
        }
    }
```

- [ ] **Step 3: 실패 확인 → player.rs 구현**

```rust
#[derive(Component)]
pub struct HyperspaceCooldown(pub Timer);

pub fn random_hyperspace_position(half: Vec2) -> Vec2 {
    use rand::RngExt;
    let mut rng = rand::rng();
    Vec2::new(
        rng.random_range(-half.x * 0.9..half.x * 0.9),
        rng.random_range(-half.y * 0.9..half.y * 0.9),
    )
}
```
`spawn_player_entity` 튜플에 `HyperspaceCooldown({ let mut t = Timer::from_seconds(HYPERSPACE_COOLDOWN_SECS, TimerMode::Once); t.tick(t.duration()); t })` 추가(`use crate::core::config::HYPERSPACE_COOLDOWN_SECS;`).

발동 시스템(`PlayerPlugin` Update Playing):
```rust
fn hyperspace(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<crate::fx::audio::SfxEvent>,
    mut query: Query<(&mut Transform, &mut crate::core::components::Velocity, &mut HyperspaceCooldown), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut cooldown)) = query.single_mut() else { return; };
    cooldown.0.tick(time.delta());
    if !keys.just_pressed(KeyCode::KeyH) || !cooldown.0.is_finished() {
        return;
    }
    let pos = random_hyperspace_position(Vec2::new(crate::core::config::HALF_WIDTH, crate::core::config::HALF_HEIGHT));
    transform.translation.x = pos.x;
    transform.translation.y = pos.y;
    velocity.0 = Vec2::ZERO;
    cooldown.0 = Timer::from_seconds(crate::core::config::HYPERSPACE_COOLDOWN_SECS, TimerMode::Once);
    sfx.write(crate::fx::audio::SfxEvent(crate::fx::audio::Sfx::Hyperspace));
}
```

- [ ] **Step 4: 통과 + 빌드** — `cargo test` → PASS. `cargo build` → 성공. (수동: H로 순간이동 확인)

- [ ] **Step 5: 커밋**

```bash
git add src/entities/player.rs src/core/config.rs
git commit -m "feat: 하이퍼스페이스(H) 순간이동"
```

---

## Task 11: HUD — 특수무기 충전·활성 파워업

**Files:** Modify `src/ui.rs`.

**Interfaces:**
- Consumes: `player::{Player, SpecialWeapon, Shield, RapidFire, Spread}`.
- 변경: `update_hud`에 특수무기 충전 수 + 활성 파워업(실드/연사/확산탄) 표시.

- [ ] **Step 1: ui.rs — update_hud 확장**

`use crate::entities::player::{Player, RapidFire, Shield, Spread, SpecialWeapon};` 추가. `update_hud` 시그니처에 플레이어 상태 쿼리 추가:
```rust
fn update_hud(
    score: Res<Score>,
    lives: Res<Lives>,
    high: Res<Persistent<HighScore>>,
    wave: Res<Wave>,
    player: Query<(Option<&SpecialWeapon>, Option<&Shield>, Option<&RapidFire>, Option<&Spread>), With<Player>>,
    mut query: Query<&mut Text, With<Hud>>,
) {
    let (charges, mods) = if let Ok((sw, sh, rf, sp)) = player.single() {
        let charges = sw.map(|w| w.charges).unwrap_or(0);
        let mut mods = String::new();
        if sh.is_some() { mods.push_str(" [실드]"); }
        if rf.is_some() { mods.push_str(" [연사]"); }
        if sp.is_some() { mods.push_str(" [확산]"); }
        (charges, mods)
    } else {
        (0, String::new())
    };
    for mut text in &mut query {
        text.0 = format!(
            "Score: {}   Lives: {}   Wave: {}   High: {}   특수: {}{}",
            score.0, lives.0, wave.0, high.0, charges, mods
        );
    }
}
```

- [ ] **Step 2: 기존 HUD 테스트 보정**

`hud_shows_score_lives_wave` 테스트는 `update_hud` 시그니처 변경으로 `player` 쿼리를 요구한다. 플레이어 엔티티가 없어도 `player.single()`이 `Err`→기본값 처리되므로 테스트는 그대로 통과해야 한다(플레이어 스폰 불필요). 다만 쿼리 파라미터가 늘었으니 `run_system_once(update_hud)`가 여전히 동작하는지 확인. 필요 시 어서션은 유지(Score/Lives/Wave/High 포함).

- [ ] **Step 3: 통과 + 빌드** — `cargo test` → PASS. `cargo clippy --all-targets -- -D warnings` → 0. (수동: HUD에 특수 충전·활성 파워업 표시 확인)

- [ ] **Step 4: 커밋 (push 하지 않음 — 최종 리뷰 후 컨트롤러가 처리)**

```bash
git add src/ui.rs
git commit -m "feat: HUD에 특수무기 충전과 활성 파워업 표시"
```

---

## Self-Review (작성자 점검 결과)

**1. Spec coverage:** 폴더 재구성(1), 배경 별(2), 화면 흔들림(3), 사운드(4), 발사 쿨다운(5), 파워업 인프라+추가목숨(6), 실드(7), 연사·확산탄(8), 특수무기(9), 하이퍼스페이스(10), HUD(11). 설계의 DoD 전 항목 대응.

**2. Placeholder scan:** 실제 코드/명령/기대값 포함. Task 6의 `<파괴위치>`는 소행성=`asteroid_tf`, UFO=`ufo_tf`로 명시. audio.rs의 임시 `#[allow(unused_imports)]` 블록은 "기본적으로 삭제"로 명시(경고 0 유지).

**3. Type consistency:** `PowerupKind`/`SpecialWeaponKind`(powerup.rs, Task 6)는 7·8·9에서 동일 사용. `SfxEvent`/`Sfx`(audio, Task 4)는 5·6·9·10에서, `ShakeEvent`(Task 3)는 9에서, `segment_circle_hit`(logic, Task 9)는 collision에서, `FireCooldown`(Task 5)은 8에서 일관. 새 경로(`crate::core::*` 등)는 Task 1 이후 전 태스크에서 사용.

**알려진 판단 포인트(구현 시 컴파일러 기준):** `MessageWriter::write` vs `send`(0.19 메서드명 — 오류 시 소스 확인 후 대체), `AudioPlayer::<AudioSource>::new` 제네릭 표기, `Query::single_mut` 반환형. Phase 1·2처럼 구현자가 소스 근거로 조정·보고.

---

## 빌드 순서 / 실행

**1(재구성) → 2·3(배경·흔들림) → 4·5(사운드·발사) → 6~9(파워업·특수무기) → 10·11(하이퍼스페이스·HUD).** 각 태스크는 `cargo test`로 검증(구현자는 `cargo run` 금지 — 시각·사운드는 사용자). 마지막 태스크만 push 보류.
