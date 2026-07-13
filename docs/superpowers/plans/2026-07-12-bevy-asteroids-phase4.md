# Bevy Asteroids Phase 4 — 카툰 스프라이트 렌더링 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 현재 Gizmos 벡터 라인(즉시 모드) 렌더링을, 자체 제작 SVG를 build.rs가 PNG로 굽는 카툰 스프라이트 렌더링으로 전면 전환한다(게임플레이 로직 불변).

**Architecture:** 각 `draw_*`(즉시 모드) 시스템을 제거하고, 엔티티 스폰 시 `Sprite` 컴포넌트를 부착해 Bevy 렌더러가 `Transform`으로 자동 렌더링하게 한다. 아트는 `assets/sprites/src/*.svg`(커밋)를 build.rs가 `resvg`로 `assets/sprites/*.png`(gitignore)로 래스터화한다(사운드 WAV와 동일 패턴). 폭발은 개별 프레임 이미지를 교체하는 애니메이션으로 구현한다.

**Tech Stack:** Rust / Bevy `0.19` (`bevy_sprite`) / `resvg`(build-dependency, SVG→PNG) / rand `0.10`.

## Global Constraints

- **Bevy `0.19`** — 모든 API는 설치된 0.19 소스에 맞춰 검증(불확실하면 컴파일러/크레이트 소스로 확인).
- **에셋 자체 완결** — 외부 다운로드·라이선스 의존 0. SVG 소스만 커밋, PNG는 `.gitignore`(`/assets/sprites/*.png`), build.rs가 매 빌드 시 생성(`cargo:rerun-if-changed=assets/sprites/src`로 증분).
- **게임플레이 불변** — 이동·발사·충돌·분열·웨이브·점수·상태 전이 변경 금지. 기존 테스트 **56개 전부 통과 유지**.
- **전환 완료 후 렌더링에 Gizmos 호출 0** — 모든 `draw_*`가 스프라이트로 대체되거나 제거됨(화염 포함).
- **테스트는 순수 로직만** — 렌더 자체는 비테스트. 순수 함수(`sprite_size_for`, `next_frame_index` 등)와 스폰 후 컴포넌트 존재만 검증.
- **커밋 메시지·PR은 한국어**, 작성자 이메일은 CrazyAtom noreply.
- **좌표 규약** — 우주선 정면 = +Y(회전 0에서 위쪽). 스프라이트는 이미지 위쪽이 +Y로 렌더되므로, 방향성 있는 스프라이트(우주선=코 위, 화염=아래, 빔=위)는 그에 맞춰 그린다.

## 아트 스타일 가이드 (모든 SVG 공통)

목업(브레인스토밍에서 승인)과 동일한 톤. SVG는 `assets/sprites/src/<name>.svg`.

- **외곽선**: 어두운 색 `stroke`, 두께는 viewBox 대비 굵게(대략 캔버스 짧은 변의 4~5%), `stroke-linejoin="round"`.
- **채색**: 플랫 컬러(또는 위→아래 살짝 어두워지는 선형 그라데이션 1개), 하이라이트 1톤(밝은 반투명), 섀도/디테일 1톤.
- **팔레트**:
  - 우주선 청록 `#8fe6ff`→`#2f9fd6`, 외곽선 `#0b1a2b`
  - 화염 `#ffe36b`→`#ff6a2c`
  - 소행성 갈색 `#c79a6d`→`#7b5636`, 외곽선 `#2a1c0f`
  - UFO 대형 초록 `#c9d3dd`/돔 `#2bd39a`, 소형 노랑 계열; 외곽선 `#101a24`
  - 총알 청록 `#38e1ff`(아군)/빨강 `#ff4d5e`(적)
  - 파워업: 실드 파랑, 연사 노랑, 확산 초록, 추가목숨 분홍, 특수무기 자홍 — 배지 형태 + 아이콘
  - 실드 반투명 하늘색 링, 빔 청록/자홍 세로 빔, 스파크 흰/노랑
  - 배경 성운 남색 `#0a1230`~`#05070d` + 별
- **viewBox = 목표 픽셀 기준 크기**(아래 표). build.rs가 이 크기의 `SPRITE_SCALE=4`배 해상도 PNG로 렌더(선명도), 게임에선 `custom_size`로 표시 크기 지정.
- **투명 배경**: 배경(`background.svg`) 외에는 캔버스에 배경 사각형을 두지 않는다(알파 투명).

### 스프라이트 규격 표 (viewBox = W×H, px)

| 파일 | viewBox | 방향/비고 |
|---|---|---|
| `ship.svg` | 40×48 | 코 위(+Y) |
| `flame.svg` | 24×28 | 위가 우주선쪽, 아래로 분사 |
| `meteor_large.svg` | 100×100 | 원형 돌덩이 |
| `meteor_medium.svg` | 60×60 | 〃 |
| `meteor_small.svg` | 36×36 | 〃 |
| `ufo_large.svg` | 60×40 | 접시 |
| `ufo_small.svg` | 40×28 | 접시 |
| `bullet.svg` | 12×28 | 세로 볼트 |
| `enemy_bullet.svg` | 12×28 | 세로 볼트(빨강) |
| `powerup_shield.svg` | 40×40 | 배지 |
| `powerup_rapid.svg` | 40×40 | 배지 |
| `powerup_spread.svg` | 40×40 | 배지 |
| `powerup_life.svg` | 40×40 | 배지 |
| `powerup_special.svg` | 40×40 | 배지 |
| `beam.svg` | 24×200 | 세로 빔(위쪽 발사) |
| `spark.svg` | 12×12 | 작은 파편 |
| `shield.svg` | 48×48 | 반투명 링 |
| `background.svg` | 1280×720 | 성운+별, 불투명 |
| `explosion_0.svg` … `explosion_5.svg` | 80×80 | 확장 버스트 6프레임 |

> **아트 저작 주체**: SVG는 스타일 일관성을 위해 **컨트롤러가 저작**한다(구현 시 Task 1에서 위 규격·팔레트대로 작성). 아래 Task 1은 파이프라인과 예시 SVG를, 각 엔티티 Task는 코드 변환을 명세한다.

---

## 파일 구조

- `Cargo.toml` — `[build-dependencies] resvg` 추가
- `.gitignore` — `/assets/sprites/*.png` 추가
- `build.rs` — 기존 WAV 생성 유지 + SVG→PNG 래스터화 추가
- `assets/sprites/src/*.svg` — 신규(커밋)
- `src/fx/sprites.rs` — 신규: `SpriteAssets` 리소스 + 로더 + `sprite_size_for` + Z 레이어 상수
- `src/fx/animation.rs` — 신규: `FrameAnimation` + `advance_animation` + `next_frame_index`
- `src/fx.rs` — `pub mod sprites; pub mod animation;` 추가
- `src/main.rs` — `SpriteAssets` 로더/`AnimationPlugin`/`SpritesPlugin` 등록
- `src/core/config.rs` — 스프라이트 크기 계수·Z 상수·폭발 타이밍
- `src/fx/background.rs`, `src/fx/effects.rs`, `src/entities/{asteroid,ufo,bullet,powerup,player}.rs` — 그리기 제거 + 스프라이트 스폰

---

## Task 1: 래스터화 파이프라인 + 예시 SVG

**Files:**
- Modify: `Cargo.toml`
- Modify: `.gitignore`
- Modify: `build.rs`
- Create: `assets/sprites/src/ship.svg` (및 스타일 가이드의 나머지 SVG 전부 — 컨트롤러 저작)

**Interfaces:**
- Produces: 빌드 시 `assets/sprites/*.png` 생성. 다른 Task는 이 PNG를 `asset_server.load("sprites/<name>.png")`로 로드.

- [ ] **Step 1: `Cargo.toml`에 build-dependency 추가**

```toml
[build-dependencies]
resvg = "0.44"
```

(설치 후 실제 버전의 `resvg::usvg` / `resvg::tiny_skia` 재수출 API를 확인한다. 아래 코드는 resvg 0.44 기준.)

- [ ] **Step 2: `.gitignore`에 PNG 제외 추가**

기존 `/assets/sounds/*.wav` 아래에 추가:

```
/assets/sprites/*.png
```

- [ ] **Step 3: `build.rs`에 SVG→PNG 래스터화 추가**

`build.rs`의 `fn main()` 마지막(`println!("cargo:rerun-if-changed=build.rs");` 위)에 아래 호출을 추가하고, 함수들을 파일 하단에 정의한다.

```rust
    // ── 스프라이트: assets/sprites/src/*.svg → assets/sprites/*.png ──
    rasterize_sprites();
    println!("cargo:rerun-if-changed=assets/sprites/src");
```

파일 하단에 추가:

```rust
const SPRITE_SCALE: f32 = 4.0; // SVG viewBox 대비 렌더 배율(선명도)

fn rasterize_sprites() {
    use resvg::{tiny_skia, usvg};
    let src = Path::new("assets/sprites/src");
    let out = Path::new("assets/sprites");
    if !src.exists() {
        return; // 소스 없으면 조용히 통과
    }
    fs::create_dir_all(out).unwrap();
    let opt = usvg::Options::default();
    for entry in fs::read_dir(src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("svg") {
            continue;
        }
        let data = fs::read(&path).unwrap();
        let tree = usvg::Tree::from_data(&data, &opt).unwrap();
        let size = tree.size();
        let w = (size.width() * SPRITE_SCALE).ceil() as u32;
        let h = (size.height() * SPRITE_SCALE).ceil() as u32;
        let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();
        let ts = tiny_skia::Transform::from_scale(SPRITE_SCALE, SPRITE_SCALE);
        resvg::render(&tree, ts, &mut pixmap.as_mut());
        let stem = path.file_stem().unwrap().to_str().unwrap();
        pixmap.save_png(out.join(format!("{stem}.png"))).unwrap();
    }
}
```

- [ ] **Step 4: 스타일 가이드대로 모든 SVG 저작**

`assets/sprites/src/`에 규격 표의 SVG 전부를 스타일 가이드/팔레트대로 작성(컨트롤러). `ship.svg` 예시(코가 위 +Y):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 48">
  <path d="M20,3 C27,3 30,15 30,25 C30,35 26,43 20,43 C14,43 10,35 10,25 C10,15 13,3 20,3 Z"
        fill="#5ec8ff" stroke="#0b1a2b" stroke-width="2.4" stroke-linejoin="round"/>
  <circle cx="20" cy="19" r="5.5" fill="#12324a" stroke="#0b1a2b" stroke-width="1.6"/>
  <path d="M12,34 L4,45 L13,41 Z" fill="#ff7a3c" stroke="#0b1a2b" stroke-width="2" stroke-linejoin="round"/>
  <path d="M28,34 L36,45 L27,41 Z" fill="#ff7a3c" stroke="#0b1a2b" stroke-width="2" stroke-linejoin="round"/>
</svg>
```

- [ ] **Step 5: 빌드로 PNG 생성 확인**

Run: `cargo build 2>&1 | tail -3`
Expected: 컴파일 성공. `ls assets/sprites/*.png` 시 규격 표의 PNG가 모두 존재. (최초 빌드 시 `resvg` 크레이트 다운로드됨.)

- [ ] **Step 6: 커밋** (SVG 소스만 — PNG는 gitignore)

```bash
git add Cargo.toml Cargo.lock .gitignore build.rs assets/sprites/src
git commit -m "feat: SVG→PNG 래스터화 파이프라인(build.rs, resvg) 및 스프라이트 SVG 소스 추가"
```

---

## Task 2: SpriteAssets 리소스 · 크기 매핑 · Z 상수

**Files:**
- Create: `src/fx/sprites.rs`
- Modify: `src/fx.rs` (모듈 선언 추가)
- Modify: `src/core/config.rs` (상수 추가)
- Modify: `src/main.rs` (플러그인 등록)
- Test: `src/fx/sprites.rs` (하단 `#[cfg(test)]`)

**Interfaces:**
- Produces:
  - `Resource SpriteAssets` — 필드: `ship, flame, meteor_large, meteor_medium, meteor_small, ufo_large, ufo_small, bullet, enemy_bullet, beam, spark, shield, background: Handle<Image>`, `powerup: [Handle<Image>; 5]`, `explosion_frames: Vec<Handle<Image>>`.
  - `fn sprite_size_for(radius: f32) -> Vec2`
  - `struct SpritesPlugin` — Startup에 `load_sprite_assets` 등록.
  - Z 상수(config): `Z_BACKGROUND=-100.0, Z_FLAME=-1.0, Z_ENTITY=0.0, Z_SHIELD=5.0, Z_BEAM=10.0, Z_PARTICLE=12.0, Z_EXPLOSION=15.0`, `VISUAL_FIT=1.2`, `EXPLOSION_FRAME_COUNT=6`, `EXPLOSION_FRAME_SECS=0.06`.
- Consumes: 없음(Task 1 PNG를 경로로 로드).

- [ ] **Step 1: config 상수 추가**

`src/core/config.rs` 하단에 추가:

```rust
// Phase 4 — 스프라이트 렌더링
pub const VISUAL_FIT: f32 = 1.2; // 콜라이더 반경 대비 스프라이트 표시 배율
pub const Z_BACKGROUND: f32 = -100.0;
pub const Z_FLAME: f32 = -1.0;
pub const Z_ENTITY: f32 = 0.0;
pub const Z_SHIELD: f32 = 5.0;
pub const Z_BEAM: f32 = 10.0;
pub const Z_PARTICLE: f32 = 12.0;
pub const Z_EXPLOSION: f32 = 15.0;
pub const EXPLOSION_FRAME_COUNT: usize = 6;
pub const EXPLOSION_FRAME_SECS: f32 = 0.06;
```

- [ ] **Step 2: `src/fx.rs`에 모듈 추가**

기존 `pub mod ...` 목록에 추가:

```rust
pub mod animation;
pub mod sprites;
```

- [ ] **Step 3: 실패하는 테스트 작성** (`src/fx/sprites.rs` 신규, 테스트만 먼저)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_size_scales_with_radius() {
        let small = sprite_size_for(14.0);
        let large = sprite_size_for(45.0);
        assert!(large.x > small.x && large.y > small.y);
        // 정사각형, 반경*2*VISUAL_FIT
        assert!((small.x - 14.0 * 2.0 * crate::core::config::VISUAL_FIT).abs() < 1e-3);
        assert_eq!(small.x, small.y);
    }
}
```

- [ ] **Step 4: 테스트 실패 확인**

Run: `cargo test sprite_size_scales_with_radius 2>&1 | tail -5`
Expected: FAIL(`sprite_size_for` 미정의로 컴파일 에러).

- [ ] **Step 5: 구현 작성** (`src/fx/sprites.rs` 상단)

```rust
use bevy::prelude::*;

use crate::core::config::{EXPLOSION_FRAME_COUNT, VISUAL_FIT};

#[derive(Resource)]
pub struct SpriteAssets {
    pub ship: Handle<Image>,
    pub flame: Handle<Image>,
    pub meteor_large: Handle<Image>,
    pub meteor_medium: Handle<Image>,
    pub meteor_small: Handle<Image>,
    pub ufo_large: Handle<Image>,
    pub ufo_small: Handle<Image>,
    pub bullet: Handle<Image>,
    pub enemy_bullet: Handle<Image>,
    pub beam: Handle<Image>,
    pub spark: Handle<Image>,
    pub shield: Handle<Image>,
    pub background: Handle<Image>,
    pub powerup: [Handle<Image>; 5], // Shield, RapidFire, Spread, ExtraLife, SpecialWeapon
    pub explosion_frames: Vec<Handle<Image>>,
}

/// 콜라이더 반경에 맞춘 정사각 스프라이트 표시 크기.
pub fn sprite_size_for(radius: f32) -> Vec2 {
    Vec2::splat(radius * 2.0 * VISUAL_FIT)
}

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sprite_assets);
    }
}

fn load_sprite_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let load = |p: &str| asset_server.load(p);
    let explosion_frames = (0..EXPLOSION_FRAME_COUNT)
        .map(|i| load(&format!("sprites/explosion_{i}.png")))
        .collect();
    commands.insert_resource(SpriteAssets {
        ship: load("sprites/ship.png"),
        flame: load("sprites/flame.png"),
        meteor_large: load("sprites/meteor_large.png"),
        meteor_medium: load("sprites/meteor_medium.png"),
        meteor_small: load("sprites/meteor_small.png"),
        ufo_large: load("sprites/ufo_large.png"),
        ufo_small: load("sprites/ufo_small.png"),
        bullet: load("sprites/bullet.png"),
        enemy_bullet: load("sprites/enemy_bullet.png"),
        beam: load("sprites/beam.png"),
        spark: load("sprites/spark.png"),
        shield: load("sprites/shield.png"),
        background: load("sprites/background.png"),
        powerup: [
            load("sprites/powerup_shield.png"),
            load("sprites/powerup_rapid.png"),
            load("sprites/powerup_spread.png"),
            load("sprites/powerup_life.png"),
            load("sprites/powerup_special.png"),
        ],
        explosion_frames,
    });
}
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test sprite_size_scales_with_radius 2>&1 | tail -3`
Expected: PASS.

- [ ] **Step 7: `src/main.rs`에 `SpritesPlugin` 등록**

`.add_plugins(ui::UiPlugin)` 뒤에 추가:

```rust
        .add_plugins(fx::sprites::SpritesPlugin)
```

(`AnimationPlugin` 등록은 모듈이 생기는 Task 11에서 추가한다.)

`load_sprite_assets`는 Task 3에서 실행 순서 보장을 위해 참조하므로 `pub(crate)`로 선언한다:

```rust
pub(crate) fn load_sprite_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
```

- [ ] **Step 8: 전체 테스트 + 커밋**

Run: `cargo test 2>&1 | tail -3` → 57 passed (기존 56 + 신규 1).
```bash
git add src/fx.rs src/fx/sprites.rs src/core/config.rs src/main.rs
git commit -m "feat: SpriteAssets 리소스와 크기 매핑·Z 레이어 상수 추가"
```

---

## Task 3: 배경 스프라이트

**Files:**
- Modify: `src/fx/background.rs`
- Test: `src/fx/background.rs`

**Interfaces:**
- Consumes: `SpriteAssets.background`, `Z_BACKGROUND`.

- [ ] **Step 1: `draw_stars`·`Star`·트윙클 제거, 배경 스프라이트 스폰으로 교체**

`src/fx/background.rs` 전체를 아래로 교체. Startup에서 `SpriteAssets`가 먼저 삽입되도록 `spawn_background`를 `load_sprite_assets` 뒤에 순서 지정한다(Task 2에서 `load_sprite_assets`를 `pub(crate)`로 노출).

```rust
use bevy::prelude::*;

use crate::core::config::{WINDOW_HEIGHT, WINDOW_WIDTH, Z_BACKGROUND};
use crate::fx::sprites::{load_sprite_assets, SpriteAssets};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_background.after(load_sprite_assets));
    }
}

fn spawn_background(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        Sprite {
            image: assets.background.clone(),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
    ));
}
```

- [ ] **Step 2: `spawns_configured_star_count` 테스트 제거**

`Star`/`spawn_stars`가 사라지므로 해당 테스트 삭제. 컴파일 확인.

- [ ] **Step 3: 빌드·실행 확인**

Run: `cargo build 2>&1 | tail -2` → 성공.
실행 시 별점 대신 성운 배경이 뒤에 깔림(z=-100이라 물체 뒤).

- [ ] **Step 4: 커밋**

```bash
git add src/fx/background.rs
git commit -m "feat: 별점 대신 성운 배경 스프라이트로 교체"
```

---

## Task 4: 소행성 스프라이트

**Files:**
- Modify: `src/entities/asteroid.rs`
- Modify: `src/core/logic.rs` (필요 시 `AsteroidSize`에 sprite 선택 헬퍼 — 선택)

**Interfaces:**
- Consumes: `SpriteAssets.{meteor_large,meteor_medium,meteor_small}`, `sprite_size_for`, `AsteroidSize::radius()`.

- [ ] **Step 1: `AsteroidShape`·`draw_asteroids`·`asteroid_shape` 제거, 스폰에 Sprite 추가**

`asteroid.rs`에서:
- `AsteroidShape` 구조체 삭제, `asteroid_shape()` 삭제, `draw_asteroids()` 삭제, 플러그인 Update 튜플에서 `draw_asteroids` 제거(→ `wave_control`만 남음).
- import에 `use crate::fx::sprites::{sprite_size_for, SpriteAssets};`, `use crate::core::logic::AsteroidSize;`(이미 있음).
- `spawn_asteroid` 시그니처에 `assets: &SpriteAssets` 추가하고 Sprite 부착:

```rust
fn asteroid_image(size: AsteroidSize, assets: &SpriteAssets) -> Handle<Image> {
    match size {
        AsteroidSize::Large => assets.meteor_large.clone(),
        AsteroidSize::Medium => assets.meteor_medium.clone(),
        AsteroidSize::Small => assets.meteor_small.clone(),
    }
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    assets: &SpriteAssets,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
) {
    let mut rng = rand::rng();
    let spin = rng.random_range(-ASTEROID_SPIN_MAX..ASTEROID_SPIN_MAX);
    commands.spawn((
        Asteroid { size },
        Sprite {
            image: asteroid_image(size, assets),
            custom_size: Some(sprite_size_for(size.radius())),
            ..default()
        },
        Transform::from_translation(position.extend(crate::core::config::Z_ENTITY)),
        Velocity(velocity),
        AngularVelocity(spin),
        Collider { radius: size.radius() },
        Wrapping,
        GameplayEntity,
    ));
}
```

- [ ] **Step 2: `spawn_wave` / 호출부에 `assets` 전달**

`spawn_wave`, `spawn_initial_wave`, `wave_control`에 `assets: Res<SpriteAssets>`를 받아 `spawn_asteroid(&mut commands, &assets, ...)`로 전달:

```rust
fn spawn_wave(commands: &mut Commands, assets: &SpriteAssets, wave: u32) {
    let count = asteroid_count_for_wave(wave);
    let scale = asteroid_speed_scale_for_wave(wave);
    for _ in 0..count {
        let base = random_velocity(AsteroidSize::Large);
        spawn_asteroid(commands, assets, AsteroidSize::Large, random_spawn_position(), base * scale);
    }
}

fn spawn_initial_wave(mut commands: Commands, assets: Res<SpriteAssets>, wave: Res<Wave>) {
    spawn_wave(&mut commands, &assets, wave.0);
}

fn wave_control(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut wave: ResMut<Wave>,
    asteroids: Query<(), With<Asteroid>>,
) {
    if asteroids.iter().count() == 0 {
        wave.0 += 1;
        spawn_wave(&mut commands, &assets, wave.0);
    }
}
```

- [ ] **Step 3: 소행성 분열 호출부(collision) 갱신**

`spawn_asteroid` 호출부(분열 로직, 아마 `src/systems/collision.rs`)에 `&assets`를 전달하도록 시그니처/쿼리 수정. 해당 시스템에 `assets: Res<SpriteAssets>` 파라미터 추가.

- [ ] **Step 4: 기존 소행성 테스트 갱신**

`shape_is_closed_loop`, `shape_points_near_radius` 삭제(shape 제거됨). `spawn_asteroid_creates_entity_with_size`는 `assets` 인자가 필요 → 테스트에서 최소 `SpriteAssets`를 만들기 번거로우므로, **스폰 테스트는 삭제**하고 스폰 검증은 시각 확인으로 대체(렌더 비테스트 원칙). `Asteroid{size}` 자체는 게임플레이 컴포넌트라 분열 로직 테스트(collision)가 커버.

- [ ] **Step 5: 빌드·테스트·실행**

Run: `cargo build 2>&1 | tail -2` 성공, `cargo test 2>&1 | tail -3` 통과. 실행 시 소행성이 돌덩이 스프라이트로, 크기별로, 회전하며 표시.

- [ ] **Step 6: 커밋**

```bash
git add src/entities/asteroid.rs src/systems/collision.rs
git commit -m "feat: 소행성을 크기별 돌덩이 스프라이트로 렌더"
```

---

## Task 5: 우주선 + 화염 스프라이트

**Files:**
- Modify: `src/entities/player.rs`

**Interfaces:**
- Consumes: `SpriteAssets.{ship,flame}`, `Z_FLAME`, `SHIP_COLLIDER_RADIUS`.
- Produces: `Flame` 마커 컴포넌트.

- [ ] **Step 1: `SHIP_POINTS`·`draw_player` 제거, 우주선 Sprite + 화염 자식 스폰**

`spawn_player_entity`는 `Commands`만 받으므로 `SpriteAssets` 접근이 필요 → 시그니처에 `assets: &SpriteAssets` 추가. 우주선 스폰에 Sprite 추가하고 `.with_children`로 화염 자식 추가:

```rust
#[derive(Component)]
pub struct Flame;

pub fn spawn_player_entity(commands: &mut Commands, assets: &SpriteAssets) {
    commands
        .spawn((
            Player,
            Sprite {
                image: assets.ship.clone(),
                custom_size: Some(crate::fx::sprites::sprite_size_for(SHIP_COLLIDER_RADIUS)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, crate::core::config::Z_ENTITY),
            Velocity(Vec2::ZERO),
            Collider { radius: SHIP_COLLIDER_RADIUS },
            Wrapping,
            GameplayEntity,
            EngineState::default(),
            FireCooldown({
                let mut t = Timer::from_seconds(crate::core::config::FIRE_INTERVAL, TimerMode::Once);
                t.tick(t.duration());
                t
            }),
            SpecialWeapon { kind: SpecialWeaponKind::LaserBeam, charges: STARTING_SPECIAL_CHARGES },
            HyperspaceCooldown({
                let mut t = Timer::from_seconds(HYPERSPACE_COOLDOWN_SECS, TimerMode::Once);
                t.tick(t.duration());
                t
            }),
        ))
        .with_children(|parent| {
            parent.spawn((
                Flame,
                Sprite {
                    image: assets.flame.clone(),
                    custom_size: Some(Vec2::new(12.0, 16.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, -16.0, crate::core::config::Z_FLAME),
                Visibility::Hidden,
            ));
        });
}
```

- [ ] **Step 2: `draw_player` 삭제 → `update_flame` 시스템 추가**

`draw_player` 삭제. 플러그인 Update 튜플에서 `draw_player` → `update_flame`로 교체. 새 시스템:

```rust
fn update_flame(
    engine_q: Query<&EngineState, With<Player>>,
    mut flame_q: Query<(&mut Visibility, &mut Transform), With<Flame>>,
) {
    let Ok(engine) = engine_q.single() else { return };
    for (mut vis, mut tf) in &mut flame_q {
        if engine.thrusting {
            *vis = Visibility::Visible;
            tf.translation.y = -16.0; // 후미
            tf.rotation = Quat::IDENTITY;
        } else if engine.braking {
            *vis = Visibility::Visible;
            tf.translation.y = 16.0; // 전방
            tf.rotation = Quat::from_rotation_z(std::f32::consts::PI);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
```

(`SHIP_COLOR`·`FLAME_COLOR` import가 더 이상 안 쓰이면 제거. 남는 화염 색은 SVG에 있음.)

- [ ] **Step 3: `spawn_player` 래퍼에 assets 전달**

```rust
fn spawn_player(mut commands: Commands, assets: Res<SpriteAssets>) {
    spawn_player_entity(&mut commands, &assets);
}
```

- [ ] **Step 4: `player_starts_with_special_charge` 테스트 갱신**

이 테스트는 `run_system_once(spawn_player)`를 호출 → 이제 `Res<SpriteAssets>`가 필요. 테스트에서 `SpriteAssets`를 만들기 번거로우므로, 스폰 대신 **컴포넌트 직접 검증**으로 바꾸거나 삭제. 대체 테스트(자원 불필요):

```rust
#[test]
fn special_weapon_starts_with_configured_charge() {
    assert_eq!(STARTING_SPECIAL_CHARGES, 1);
}
```

(스폰 자체는 시각 확인. `STARTING_SPECIAL_CHARGES` 상수 검증으로 "시작 충전 1" 의도는 유지.)

- [ ] **Step 5: 빌드·테스트·실행**

`cargo build`/`cargo test` 통과. 실행 시 우주선이 스프라이트로, 추진(↑) 시 후미 화염, 브레이크(↓) 시 전방 화염.

- [ ] **Step 6: 커밋**

```bash
git add src/entities/player.rs
git commit -m "feat: 우주선과 화염을 스프라이트로 전환(화염은 자식 스프라이트 토글)"
```

---

## Task 6: UFO · 총알 스프라이트

**Files:**
- Modify: `src/entities/ufo.rs`, `src/entities/bullet.rs`

**Interfaces:**
- Consumes: `SpriteAssets.{ufo_large,ufo_small,bullet,enemy_bullet}`, `sprite_size_for`.

- [ ] **Step 1: UFO 스폰에 Sprite, `draw_ufos` 제거**

`ufo.rs`:
- `draw_ufos` 삭제, 플러그인 Update 튜플에서 제거.
- `spawn_ufo`에 `assets: &SpriteAssets` 인자 추가, Sprite 부착:

```rust
fn ufo_image(size: UfoSize, assets: &SpriteAssets) -> Handle<Image> {
    match size {
        UfoSize::Large => assets.ufo_large.clone(),
        UfoSize::Small => assets.ufo_small.clone(),
    }
}
```
`spawn_ufo` 내 `commands.spawn((Ufo{..}, Transform.., Velocity.., Collider.., GameplayEntity, ..))`에
```rust
        Sprite {
            image: ufo_image(size, assets),
            custom_size: Some(sprite_size_for(size.radius())),
            ..default()
        },
```
추가하고 `Transform` z를 `Z_ENTITY`로. `ufo_spawn_system`(호출부)에 이미 `Res` 접근 가능 → `assets: Res<SpriteAssets>` 파라미터 추가해 `spawn_ufo(&mut commands, &assets, size, from_left)`.

- [ ] **Step 2: 총알 스폰에 Sprite, `draw_bullets`/`draw_enemy_bullets` 제거**

`bullet.rs`:
- 두 `draw_*` 삭제, 플러그인 Update 튜플에서 제거.
- `fire_bullet`에 `assets: Res<SpriteAssets>` 추가, Bullet 스폰에 Sprite:
```rust
        Sprite {
            image: assets.bullet.clone(),
            custom_size: Some(sprite_size_for(BULLET_COLLIDER_RADIUS)),
            ..default()
        },
```
- `spawn_enemy_bullet`에 `assets: &SpriteAssets` 추가, EnemyBullet 스폰에 Sprite(`assets.enemy_bullet`, `sprite_size_for(ENEMY_BULLET_COLLIDER_RADIUS)`). 호출부(`ufo_fire`)에 `assets` 전달.

주의: `sprite_size_for(2.0)`은 2*2*1.2=4.8px로 너무 작다 → 총알은 가독성 위해 `custom_size: Some(Vec2::new(6.0, 14.0))`(볼트 형태)로 명시. 적 총알도 `Vec2::new(6.0, 14.0)`.

- [ ] **Step 3: 총알 스폰 테스트 갱신**

`bullet.rs` 테스트 중 `fire_bullet`/`spawn_enemy_bullet`를 호출하는 것(`fires_only_when_cooldown_ready`, `spread_fires_three_bullets`, `spawn_enemy_bullet_creates_entity`)은 `SpriteAssets`가 필요. 이들 테스트에 최소 `SpriteAssets`를 삽입하는 헬퍼를 추가하거나, **스폰 개수 검증만 남기고 `SpriteAssets`를 `app.insert_resource`로 제공**한다. 헬퍼:

```rust
#[cfg(test)]
fn dummy_sprite_assets() -> crate::fx::sprites::SpriteAssets {
    use bevy::prelude::*;
    let h = Handle::<Image>::default();
    crate::fx::sprites::SpriteAssets {
        ship: h.clone(), flame: h.clone(),
        meteor_large: h.clone(), meteor_medium: h.clone(), meteor_small: h.clone(),
        ufo_large: h.clone(), ufo_small: h.clone(), bullet: h.clone(), enemy_bullet: h.clone(),
        beam: h.clone(), spark: h.clone(), shield: h.clone(), background: h.clone(),
        powerup: [h.clone(), h.clone(), h.clone(), h.clone(), h.clone()],
        explosion_frames: vec![h.clone(); 6],
    }
}
```
`SpriteAssets` 필드는 모두 `pub`이어야 함(Task 2에서 `pub`). 각 관련 테스트에 `app.insert_resource(dummy_sprite_assets());` 추가.

- [ ] **Step 4: 빌드·테스트·실행**

통과. 실행 시 UFO·총알이 스프라이트로.

- [ ] **Step 5: 커밋**

```bash
git add src/entities/ufo.rs src/entities/bullet.rs
git commit -m "feat: UFO와 총알(아군/적)을 스프라이트로 전환"
```

---

## Task 7: 파워업 스프라이트

**Files:**
- Modify: `src/entities/powerup.rs`

**Interfaces:**
- Consumes: `SpriteAssets.powerup[5]`, `sprite_size_for`, `POWERUP_RADIUS`.
- Produces: `fn powerup_sprite_index(kind: PowerupKind) -> usize`.

- [ ] **Step 1: 실패 테스트 — 종류→인덱스 매핑**

`powerup.rs` 테스트에 추가:

```rust
#[test]
fn powerup_index_is_stable_and_distinct() {
    use PowerupKind::*;
    let idx = |k| powerup_sprite_index(k);
    assert_eq!(idx(Shield), 0);
    assert_eq!(idx(RapidFire), 1);
    assert_eq!(idx(Spread), 2);
    assert_eq!(idx(ExtraLife), 3);
    assert_eq!(idx(SpecialWeapon(SpecialWeaponKind::LaserBeam)), 4);
}
```

- [ ] **Step 2: 실패 확인**

Run: `cargo test powerup_index_is_stable_and_distinct 2>&1 | tail -5` → FAIL(미정의).

- [ ] **Step 3: 구현 — 인덱스 함수 + Sprite 스폰, `draw_powerups`/`powerup_color` 제거**

```rust
pub fn powerup_sprite_index(kind: PowerupKind) -> usize {
    match kind {
        PowerupKind::Shield => 0,
        PowerupKind::RapidFire => 1,
        PowerupKind::Spread => 2,
        PowerupKind::ExtraLife => 3,
        PowerupKind::SpecialWeapon(_) => 4,
    }
}
```
`draw_powerups`·`powerup_color` 삭제, 플러그인 Update 튜플에서 `draw_powerups` 제거. `spawn_powerup`에 `assets: &SpriteAssets` 추가:
```rust
        Sprite {
            image: assets.powerup[powerup_sprite_index(kind)].clone(),
            custom_size: Some(sprite_size_for(POWERUP_RADIUS)),
            ..default()
        },
```
`Transform` z를 `Z_ENTITY`로. 호출부(드롭 로직, collision)에 `assets` 전달.

- [ ] **Step 4: 스폰 테스트 갱신**

`spawn_powerup_creates_entity`는 `assets` 필요 → `dummy_sprite_assets`(Task 6에서 도입, 또는 `sprites.rs`에 `#[cfg(test)] pub fn` 공용화) 사용. 공용화를 위해 `dummy_sprite_assets`를 `src/fx/sprites.rs`에 `#[cfg(test)] pub fn`로 정의하고 각 테스트에서 재사용.

- [ ] **Step 5: 테스트·빌드·실행**

`cargo test`/`cargo build` 통과. 실행 시 파워업이 종류별 배지 스프라이트로.

- [ ] **Step 6: 커밋**

```bash
git add src/entities/powerup.rs src/fx/sprites.rs src/systems/collision.rs
git commit -m "feat: 파워업 5종을 종류별 배지 스프라이트로 전환"
```

---

## Task 8: 실드 스프라이트

**Files:**
- Modify: `src/entities/player.rs`

**Interfaces:**
- Consumes: `SpriteAssets.shield`, `Z_SHIELD`.
- Produces: `ShieldSprite` 마커.

- [ ] **Step 1: `draw_shield` 제거 → 실드 부여/해제 시 스프라이트 자식 토글**

실드는 `Shield` 컴포넌트가 붙는 동안만 보인다. 간단히: `draw_shield` 삭제하고, 우주선에 항상 실드 스프라이트 자식(`ShieldSprite`, 기본 Hidden)을 두고, `Shield` 유무로 가시성 토글하는 시스템 추가.

Task 5의 `spawn_player_entity` `.with_children`에 실드 자식 추가:
```rust
            parent.spawn((
                ShieldSprite,
                Sprite {
                    image: assets.shield.clone(),
                    custom_size: Some(Vec2::splat(48.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, crate::core::config::Z_SHIELD),
                Visibility::Hidden,
            ));
```
새 시스템(플러그인 Update 튜플에서 `draw_shield` → `update_shield_sprite`):
```rust
#[derive(Component)]
pub struct ShieldSprite;

fn update_shield_sprite(
    player_q: Query<Has<Shield>, With<Player>>,
    mut shield_q: Query<&mut Visibility, With<ShieldSprite>>,
) {
    let Ok(has_shield) = player_q.single() else { return };
    for mut vis in &mut shield_q {
        *vis = if has_shield { Visibility::Visible } else { Visibility::Hidden };
    }
}
```
(`Has<Shield>`는 Bevy 0.19의 존재 여부 쿼리. 미지원 시 `Option<&Shield>`로 대체하고 `.is_some()` 사용.)

- [ ] **Step 2: 빌드·실행**

실드 파워업 획득 시 우주선 주위 반투명 버블이 뜨고, 만료되면 사라짐.

- [ ] **Step 3: 커밋**

```bash
git add src/entities/player.rs
git commit -m "feat: 실드를 반투명 버블 스프라이트로 전환"
```

---

## Task 9: 스프라이트 빔

**Files:**
- Modify: `src/entities/player.rs`

**Interfaces:**
- Consumes: `SpriteAssets.beam`, `Z_BEAM`, `BEAM_WIDTH`, `BEAM_LENGTH`, `BEAM_LIFETIME_SECS`.

- [ ] **Step 1: `activate_special`에서 빔 스폰 시 Sprite 부착**

`SpecialBeam` 스폰(`activate_special` 내)에 Sprite와 Transform(빔은 원점에서 dir 방향으로 뻗음). 빔 스프라이트는 세로(+Y)로 그려졌으므로, `dir`을 향하도록 회전. 스프라이트 중심이 원점+길이/2에 오도록 배치:

```rust
        SpecialWeaponKind::LaserBeam => {
            let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2; // +Y 기준 → dir
            let center = origin + dir.normalize_or_zero() * (BEAM_LENGTH * 0.5);
            commands.spawn((
                SpecialBeam { life: Timer::from_seconds(BEAM_LIFETIME_SECS, TimerMode::Once), origin, dir },
                Sprite {
                    image: assets.beam.clone(),
                    custom_size: Some(Vec2::new(BEAM_WIDTH, BEAM_LENGTH)),
                    ..default()
                },
                Transform {
                    translation: center.extend(crate::core::config::Z_BEAM),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                GameplayEntity,
            ));
        }
```
`activate_special`에 `assets: Res<SpriteAssets>` 파라미터 추가.

- [ ] **Step 2: `tick_and_draw_beam` → `tick_beam`(Gizmos 제거)**

빔 판정/수명 로직은 유지, Gizmos 그리기만 제거:
```rust
fn tick_beam(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpecialBeam)>,
) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```
플러그인 Update 튜플에서 `tick_and_draw_beam` → `tick_beam`. (빔 대상 히트 판정이 collision 쪽에 있다면 그대로 유지; `SpecialBeam.origin/dir`는 그대로 사용.)

- [ ] **Step 3: 빌드·실행**

X키 발동 시 정면으로 굵은 빔 스프라이트가 잠깐 뻗음.

- [ ] **Step 4: 커밋**

```bash
git add src/entities/player.rs
git commit -m "feat: 특수무기 빔을 스프라이트 빔으로 전환"
```

---

## Task 10: 텍스처 파티클

**Files:**
- Modify: `src/fx/effects.rs`

**Interfaces:**
- Consumes: `SpriteAssets.spark`, `Z_PARTICLE`.
- Produces: `fn particle_alpha(fraction_remaining: f32) -> f32`.

- [ ] **Step 1: 실패 테스트 — 알파 매핑**

```rust
#[test]
fn particle_alpha_follows_remaining_life() {
    assert!((particle_alpha(1.0) - 1.0).abs() < 1e-6);
    assert!((particle_alpha(0.0)).abs() < 1e-6);
    assert!(particle_alpha(0.5) > 0.0 && particle_alpha(0.5) < 1.0);
}
```

- [ ] **Step 2: 실패 확인** → `cargo test particle_alpha_follows_remaining_life` FAIL.

- [ ] **Step 3: 구현 — spark 스프라이트 파티클 + fade, `draw_particles` 제거**

`spawn_explosion`에 `assets: &SpriteAssets` 추가, 파티클 스폰에 Sprite:
```rust
pub fn particle_alpha(fraction_remaining: f32) -> f32 {
    fraction_remaining.clamp(0.0, 1.0)
}
```
파티클 스폰:
```rust
        commands.spawn((
            Particle { life: Timer::from_seconds(PARTICLE_LIFETIME_SECS, TimerMode::Once) },
            Sprite {
                image: assets.spark.clone(),
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_translation(position.extend(crate::core::config::Z_PARTICLE)),
            Velocity(velocity),
            GameplayEntity,
        ));
```
`draw_particles` 삭제 → `fade_particles`로 교체:
```rust
fn fade_particles(mut q: Query<(&Particle, &mut Sprite)>) {
    for (particle, mut sprite) in &mut q {
        sprite.color.set_alpha(particle_alpha(particle.life.fraction_remaining()));
    }
}
```
플러그인 Update 튜플: `draw_particles` → `fade_particles`. `spawn_explosion` 호출부(collision)에 `assets` 전달.

- [ ] **Step 4: 스폰 테스트 갱신**

`spawn_explosion_creates_particles`는 `assets` 필요 → `dummy_sprite_assets` 사용하도록 수정.

- [ ] **Step 5: 테스트·빌드·실행** 통과. 폭발/피격 시 스파크가 튀며 페이드.

- [ ] **Step 6: 커밋**

```bash
git add src/fx/effects.rs src/systems/collision.rs
git commit -m "feat: 파편 파티클을 스파크 스프라이트로 전환(수명 알파 페이드)"
```

---

## Task 11: 애니메이션 폭발

**Files:**
- Create: `src/fx/animation.rs`
- Modify: `src/fx/effects.rs` (폭발 시 애니 스폰), `src/main.rs` (AnimationPlugin 등록)

**Interfaces:**
- Consumes: `SpriteAssets.explosion_frames`, `Z_EXPLOSION`, `EXPLOSION_FRAME_SECS`.
- Produces: `struct AnimationPlugin`, `Component FrameAnimation`, `fn next_frame_index(current, last) -> Option<usize>`, `fn spawn_explosion_anim(commands, assets, position)`.

- [ ] **Step 1: 실패 테스트 — 프레임 진행**

`src/fx/animation.rs`(테스트만 먼저):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_advances_then_ends() {
        assert_eq!(next_frame_index(0, 5), Some(1));
        assert_eq!(next_frame_index(4, 5), Some(5));
        assert_eq!(next_frame_index(5, 5), None); // 마지막 → despawn
    }
}
```

- [ ] **Step 2: 실패 확인** → FAIL(미정의).

- [ ] **Step 3: 구현**

```rust
use bevy::prelude::*;

use crate::core::config::{EXPLOSION_FRAME_SECS, Z_EXPLOSION};
use crate::fx::sprites::SpriteAssets;

#[derive(Component)]
pub struct FrameAnimation {
    pub frames: Vec<Handle<Image>>,
    pub timer: Timer,
    pub index: usize,
}

/// 다음 프레임 인덱스. last를 넘어서면 None(=despawn).
pub fn next_frame_index(current: usize, last: usize) -> Option<usize> {
    if current >= last {
        None
    } else {
        Some(current + 1)
    }
}

pub fn spawn_explosion_anim(commands: &mut Commands, assets: &SpriteAssets, position: Vec2) {
    let frames = assets.explosion_frames.clone();
    let first = frames[0].clone();
    commands.spawn((
        FrameAnimation {
            frames,
            timer: Timer::from_seconds(EXPLOSION_FRAME_SECS, TimerMode::Repeating),
            index: 0,
        },
        Sprite { image: first, custom_size: Some(Vec2::splat(64.0)), ..default() },
        Transform::from_translation(position.extend(Z_EXPLOSION)),
        crate::core::state::GameplayEntity,
    ));
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            advance_animation.run_if(in_state(crate::core::state::GameState::Playing)),
        );
    }
}

fn advance_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut FrameAnimation, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut q {
        anim.timer.tick(time.delta());
        if !anim.timer.is_finished() {
            continue;
        }
        let last = anim.frames.len() - 1;
        match next_frame_index(anim.index, last) {
            Some(next) => {
                anim.index = next;
                sprite.image = anim.frames[next].clone();
            }
            None => {
                commands.entity(entity).despawn();
            }
        }
    }
}
```

- [ ] **Step 4: 폭발 발생 시 애니 스폰 연결**

`fx/effects.rs`의 `spawn_explosion`(파티클) 호출부(collision, 소행성/UFO 파괴·피격)에서 `spawn_explosion_anim(&mut commands, &assets, position)`도 함께 호출. 또는 `spawn_explosion` 내부에서 `crate::fx::animation::spawn_explosion_anim(...)`을 호출(파티클 + 애니 동시). 후자 채택.

- [ ] **Step 5: `main.rs`에 AnimationPlugin 등록**

`.add_plugins(fx::sprites::SpritesPlugin)` 뒤:
```rust
        .add_plugins(fx::animation::AnimationPlugin)
```

- [ ] **Step 6: 테스트·빌드·실행** 통과. 폭발 시 프레임 애니가 재생되고 끝나면 사라짐.

- [ ] **Step 7: 커밋**

```bash
git add src/fx/animation.rs src/fx/effects.rs src/fx.rs src/main.rs src/systems/collision.rs
git commit -m "feat: 폭발 애니메이션(개별 프레임 스왑) 추가"
```

---

## Task 12: 마무리 — 잔여 Gizmos 확인·팔레트·정리

**Files:**
- Modify: 필요한 파일들, `README.md`

- [ ] **Step 1: 잔여 Gizmos 사용 0 확인**

Run: `grep -rn "gizmos\|Gizmos\|linestrip\|circle_2d\|line_2d" src --include="*.rs" | grep -v test`
Expected: 렌더링 관련 Gizmos 호출 없음(있으면 스프라이트로 정리). `Gizmos` import 잔재 제거.

- [ ] **Step 2: 미사용 import·상수 정리**

`SHIP_COLOR`, `FLAME_COLOR`, `STAR_COUNT`, `TWINKLE_SPEED` 등 더 이상 안 쓰는 config 상수/컴포넌트 제거. `cargo build`로 경고 확인.

- [ ] **Step 3: clippy 클린**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -3`
Expected: 경고 0. (필요 시 `#[allow(clippy::type_complexity)]` 유지/추가.)

- [ ] **Step 4: 전체 테스트**

Run: `cargo test 2>&1 | tail -3`
Expected: 전부 통과(게임플레이 테스트 유지 + 신규 순수함수 테스트).

- [ ] **Step 5: 실행 최종 확인**

`cargo run` — 모든 요소(우주선·화염·소행성·UFO·총알·파워업·실드·빔·파편·폭발·배경)가 카툰 스프라이트로 표시되고 게임이 정상 동작.

- [ ] **Step 6: README 한 줄**

`README.md`에 아트가 자체 제작 SVG(빌드 시 PNG 생성)임을 한 줄 명시.

- [ ] **Step 7: 커밋**

```bash
git add -A
git commit -m "chore: 잔여 Gizmos 제거·미사용 상수 정리·clippy 클린(Phase 4 마무리)"
```

---

## Self-Review 메모 (작성자 확인)

- **스펙 커버리지**: §3(아키텍처)→Task 2·각 엔티티 Task, §3.3(애니)→Task 11, §3.4(파티클)→Task 10, §3.5(빔)→Task 9, §3.6(화염)→Task 5, §4(파이프라인)→Task 1, §5(절차적 0)→Task 12 Step 1, §7(테스트)→각 Task 순수함수. 전 항목 커버.
- **타입 일관성**: `SpriteAssets`(필드명), `sprite_size_for`, `powerup_sprite_index`, `next_frame_index`, `FrameAnimation`, `Flame`/`ShieldSprite` 마커 — Task 간 명칭 일치.
- **알려진 확인 필요(구현 중 컴파일러로 검증)**: `resvg` 0.44 API(`usvg::Tree::from_data`, `resvg::render`, `tiny_skia::Pixmap`), Bevy 0.19 `Sprite`/`Visibility`/`Has<T>`/`with_children`/`single()` 시그니처. 불확실 시 설치된 크레이트 소스로 확인 후 진행.
- **Startup 순서**: `SpriteAssets` 삽입 후 배경 스폰 — `.after(load_sprite_assets)`로 보장(Task 3).
