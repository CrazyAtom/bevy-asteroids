# 특수무기 다양화(5종 + 카트라이더 큐) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 특수무기를 1종(레이저 빔) → 5종으로 확장하고, 획득·발동을 카트라이더 방식 FIFO 큐로 전환한다(획득 순서대로 X 발동, 드롭 아이콘 종류별 구분, HUD 큐 슬롯 5개).

**Architecture:** `special_weapon.rs` 루트가 `SpecialWeaponKind`(powerup에서 이동해 옴)·`SpecialWeapon{queue}`·발동 디스패치를 소유하고, 무기별 로직은 `special_weapon/{beam,nova,missile,shockwave,shield_burst}.rs` 서브모듈에 산다(boss/ 패턴). 미사일은 `Bullet`+`Homing` 마커로 기존 총알 충돌·보스 피해 경로를 재사용. 공격형 실드는 기존 `Shield`(무적) 재사용 + 접촉 파괴 + 우주선 번쩍임(주황 틴트 토글)·크기 펄스.

**Tech Stack:** Rust 2021 / Bevy 0.19 / rand 0.10. SVG→PNG(build.rs, resvg). 아이콘 SVG는 기존 파워업 규격(viewBox 40×40), 미사일 투사체는 bullet 규격(12×28)에 준함.

## Global Constraints

- **Bevy 0.19 API**: `MessageWriter`, `query.single()`(Result), `Or<(...)>` 필터, `ImageNode`(UI). 리소스는 플러그인 `build()`.
- **동작 보존(T1)**: T1 완료 시점의 플레이 체감은 기존과 동일해야 한다(시작 [빔]×1, X→빔, 드롭 아이콘도 빔 아이콘 그대로).
- **바이너리 dead_code**: 항목은 **소비처와 같은 태스크**에서 추가. 임시 `#[allow(dead_code)]` 금지(이 플랜은 그렇게 배치됨).
- **재사용**: 노바는 `spawn_bullet` 헬퍼, 미사일은 `Bullet` 마커(기존 충돌·보스 피해), 공격실드는 `Shield`(무적).
- **테스트**: 순수 로직 + `run_system_once` 스폰 검증. 기존 95개 유지. 각 태스크 후 `cargo test`(전부 통과) + `cargo clippy --all-targets -- -D warnings`(경고 0).
- **리뷰 정책**: 매 태스크 완료 후 **적대적 리뷰어 필수**.
- **Git**: 커밋 한국어 + `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`. 브랜치 `feat/special-weapons`.
- **상수 확정값**: `HUD_QUEUE_SLOTS=5`, `NOVA_BULLETS=16`, `MISSILE_COUNT=5`, `MISSILE_SPEED=420.0`, `MISSILE_TURN_RATE=6.0`, `MISSILE_LIFETIME_SECS=2.5`, `MISSILE_COLLIDER_RADIUS=4.0`, `MISSILE_SPREAD=0.5`, `SHOCKWAVE_RADIUS=220.0`, `SHOCKWAVE_IMPULSE=380.0`, `SHOCKWAVE_RING_SECS=0.35`, `SHIELD_BURST_SECS=4.0`, `BURST_FLASH_HZ=12.0`, `BURST_PULSE=0.05`.

---

## File Structure

- `entities/special_weapon.rs` — 루트: `SpecialWeaponKind`(5종으로 성장)·`SpecialWeapon{queue}`·`weapon_icon`·`pick_weapon_kind`·발동 디스패치·플러그인·재수출(`pub use beam::SpecialBeam`).
- `entities/special_weapon/{beam,nova,missile,shockwave,shield_burst}.rs` — 무기별 fire/시스템/순수함수/테스트.
- `entities/powerup.rs` — Kind 정의 제거(→import), 드롭 스프라이트 kind별, collect→`queue.push_back`.
- `entities/bullet.rs` — `spawn_bullet` 헬퍼 추출(fire_bullet이 사용).
- `entities/player.rs` — 시작 큐, F1 5종 push, import 경로.
- `ui/hud.rs` — SpChargeText → `WeaponSlotIcon` 큐 슬롯.
- `core/config/combat.rs` — 위 상수(소비 태스크에서 추가).
- `fx/sprites.rs` — `weapon_nova/missile/shockwave/burst`, `missile` 핸들(T2/T3).
- `assets/sprites/src/` — `weapon_nova/missile/shockwave/burst.svg`(40×40), `missile.svg`(12×28).

---

## Task 1: 카트라이더 큐 전환 + 모듈 골격 (동작 동일)

Kind는 아직 `LaserBeam` 1종. 큐 인프라·beam 서브모듈·Kind 소유권 이동·HUD 큐 슬롯·`spawn_bullet` 헬퍼를 완성한다. 완료 시 플레이 체감은 기존과 동일.

**Files:**
- Create: `src/entities/special_weapon/beam.rs`
- Modify: `src/entities/special_weapon.rs`(전면 개편), `src/entities/powerup.rs`, `src/entities/player.rs`, `src/entities/bullet.rs`, `src/ui/hud.rs`, `src/ui.rs`, `src/core/config/combat.rs`

**Interfaces:**
- Produces: `SpecialWeaponKind`(special_weapon 소유), `SpecialWeapon{queue: VecDeque<SpecialWeaponKind>}`, `weapon_icon(kind,&SpriteAssets)->Handle<Image>`, `beam::fire`, `pub use beam::SpecialBeam`(외부 경로 불변), `bullet::spawn_bullet(commands,assets,pos: Vec2,dir: Vec2)`, `hud::update_weapon_slots`.

- [ ] **Step 1: 상수 추가** — `src/core/config/combat.rs` 끝에:

```rust
// 특수무기 — 카트라이더 큐
pub const HUD_QUEUE_SLOTS: usize = 5; // HUD에 표시할 큐 슬롯 수(내부 큐는 무제한)
```

- [ ] **Step 2: `beam.rs` 생성** — 기존 special_weapon.rs의 `SpecialBeam`·빔 스폰(activate_special의 LaserBeam arm 본문)·`tick_beam`을 **그대로 이동**:

```rust
//! 레이저 빔(기존 특수무기). 판정은 systems::collision::beam_vs_targets,
//! 보스 피해는 entities::boss::boss_combat이 담당. 여기서는 발사와 수명만.

use bevy::prelude::*;

use crate::core::config::{BEAM_LENGTH, BEAM_LIFETIME_SECS, BEAM_WIDTH, Z_BEAM};
use crate::core::state::GameplayEntity;
use crate::fx::sprites::SpriteAssets;

/// 발사된 레이저 빔. `origin`에서 `dir` 방향으로 뻗는 선분으로 판정한다.
#[derive(Component)]
pub struct SpecialBeam {
    pub life: Timer,
    pub origin: Vec2,
    pub dir: Vec2,
    /// 보스에게는 프레임당이 아니라 빔 1회당 한 번만 피해를 준다.
    pub damaged_boss: bool,
}

/// 우주선 정면으로 빔을 스폰한다. 디스패치(activate_special)에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, transform: &Transform) {
    let origin = transform.translation.truncate();
    let dir = (transform.rotation * Vec3::Y).truncate();
    // 빔 스프라이트는 세로(+Y)로 그려짐 → dir 방향으로 회전, 원점~사거리 중앙에 배치.
    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    let center = origin + dir.normalize_or_zero() * (BEAM_LENGTH * 0.5);
    commands.spawn((
        SpecialBeam {
            life: Timer::from_seconds(BEAM_LIFETIME_SECS, TimerMode::Once),
            origin,
            dir,
            damaged_boss: false,
        },
        Sprite {
            image: assets.beam.clone(),
            custom_size: Some(Vec2::new(BEAM_WIDTH, BEAM_LENGTH)),
            ..default()
        },
        Transform {
            translation: center.extend(Z_BEAM),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        GameplayEntity,
    ));
}

/// 빔 수명 관리(그리기는 스프라이트가 담당). 판정은 collision::beam_vs_targets.
pub(super) fn tick_beam(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut SpecialBeam)>) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

- [ ] **Step 3: `special_weapon.rs` 루트 전면 개편**:

```rust
//! 특수무기(X 키) — 카트라이더 방식 FIFO 큐. 획득한 순서대로 쌓이고 X를 누르면
//! 맨 앞 무기부터 발동한다. 무기별 로직은 서브모듈에 있고 여기의 match 디스패치가
//! 위임한다(새 무기 = 변형 추가 → 컴파일러가 모든 지점 강제).

pub mod beam;

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::core::config::SHAKE_SPECIAL;
use crate::core::state::GameState;
use crate::entities::player::Player;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

pub use beam::SpecialBeam; // collision·boss의 기존 경로 유지

/// 특수무기 종류. 파워업 드롭·큐·HUD 아이콘이 모두 이 enum을 공유한다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpecialWeaponKind {
    LaserBeam,
}

/// 획득 순서(FIFO) 큐. 앞 = 다음 발동. 내부는 무제한, HUD는 앞 HUD_QUEUE_SLOTS개 표시.
#[derive(Component)]
pub struct SpecialWeapon {
    pub queue: VecDeque<SpecialWeaponKind>,
}

/// kind → 드롭/HUD 아이콘 핸들.
pub fn weapon_icon(kind: SpecialWeaponKind, assets: &SpriteAssets) -> Handle<Image> {
    match kind {
        SpecialWeaponKind::LaserBeam => assets.powerup[4].clone(),
    }
}

/// 특수무기 드롭 종류 추첨(roll 0..1 균등 버킷). 파워업 드롭에서 사용.
pub fn pick_weapon_kind(_roll: f32) -> SpecialWeaponKind {
    SpecialWeaponKind::LaserBeam
}

pub struct SpecialWeaponPlugin;

impl Plugin for SpecialWeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (activate_special, beam::tick_beam).run_if(in_state(GameState::Playing)),
        );
    }
}

/// X = 큐 맨 앞 무기 발동(pop). 무기별 fire는 서브모듈로 위임.
fn activate_special(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut shake: MessageWriter<ShakeEvent>,
    mut query: Query<(&Transform, &mut SpecialWeapon), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let Ok((transform, mut weapon)) = query.single_mut() else { return };
    let Some(kind) = weapon.queue.pop_front() else { return };
    match kind {
        SpecialWeaponKind::LaserBeam => beam::fire(&mut commands, &assets, transform),
    }
    sfx.write(SfxEvent(Sfx::Special));
    shake.write(ShakeEvent(SHAKE_SPECIAL));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn app_with_input_and_assets(pressed: bool) -> App {
        let mut app = App::new();
        app.add_message::<SfxEvent>();
        app.add_message::<ShakeEvent>();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        let mut keys = ButtonInput::<KeyCode>::default();
        if pressed {
            keys.press(KeyCode::KeyX);
        }
        app.insert_resource(keys);
        app.insert_resource(Time::<()>::default());
        app
    }

    #[test]
    fn x_fires_front_and_consumes_in_order() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((
            Player,
            Transform::default(),
            SpecialWeapon {
                queue: VecDeque::from(vec![SpecialWeaponKind::LaserBeam, SpecialWeaponKind::LaserBeam]),
            },
        ));
        app.world_mut().run_system_once(activate_special).unwrap();
        // 맨 앞 1개 소모 + 빔 1개 스폰
        let mut beams = app.world_mut().query::<&SpecialBeam>();
        assert_eq!(beams.iter(app.world()).count(), 1);
        let mut wq = app.world_mut().query::<&SpecialWeapon>();
        assert_eq!(wq.single(app.world()).unwrap().queue.len(), 1);
    }

    #[test]
    fn empty_queue_fires_nothing() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((Player, Transform::default(), SpecialWeapon { queue: VecDeque::new() }));
        app.world_mut().run_system_once(activate_special).unwrap();
        let mut beams = app.world_mut().query::<&SpecialBeam>();
        assert_eq!(beams.iter(app.world()).count(), 0);
    }
}
```

> `pick_weapon_kind`는 T1에서 파워업 드롭이 즉시 소비(1종이라 상수 반환). T2/T3에서 버킷이 늘어난다.

- [ ] **Step 4: powerup.rs 갱신** — ① `SpecialWeaponKind` enum 정의(15–18행) 삭제, ② import를 `use crate::entities::special_weapon::{pick_weapon_kind, weapon_icon, SpecialWeapon, SpecialWeaponKind};`로 교체, ③ `pick_powerup_kind`의 마지막 버킷을 서브 roll 재스케일로:

```rust
        r if r < 0.88 => PowerupKind::ExtraLife,
        r => PowerupKind::SpecialWeapon(pick_weapon_kind((r - 0.88) / 0.12)),
```

④ `spawn_powerup`의 이미지 선택을 kind별로:

```rust
    let image = match kind {
        PowerupKind::SpecialWeapon(w) => weapon_icon(w, assets),
        _ => assets.powerup[powerup_sprite_index(kind)].clone(),
    };
```
(스폰 번들의 `image:` 필드는 `image,`로 교체.)

⑤ collect의 특수무기 arm:

```rust
                PowerupKind::SpecialWeapon(kind) => {
                    if let Ok(mut weapon) = special_q.get_mut(player_entity) {
                        weapon.queue.push_back(kind);
                    }
                }
```

- [ ] **Step 5: player.rs 갱신** — ① import: `use crate::entities::powerup::SpecialWeaponKind;` 삭제, `use crate::entities::special_weapon::{SpecialWeapon, SpecialWeaponKind};`로 병합. ② 스폰 번들의 `SpecialWeapon { kind: ..., charges: ... }`를:

```rust
            SpecialWeapon {
                queue: std::collections::VecDeque::from(vec![
                    SpecialWeaponKind::LaserBeam;
                    STARTING_SPECIAL_CHARGES as usize
                ]),
            },
```

③ `debug_fill_hud`의 `weapon.charges += 1;` →

```rust
        weapon.queue.push_back(SpecialWeaponKind::LaserBeam);
```

④ 테스트 `player_spawns_with_special_charge`의 단정 교체:

```rust
        assert_eq!(weapon.queue.len(), STARTING_SPECIAL_CHARGES as usize);
        assert_eq!(weapon.queue.front(), Some(&SpecialWeaponKind::LaserBeam));
        assert!(!weapon.queue.is_empty(), "게임 시작 시 특수무기를 최소 1개 보유해야 한다");
```

- [ ] **Step 6: bullet.rs — `spawn_bullet` 헬퍼 추출.** `fire_bullet`의 루프 내부 스폰 블록을 헬퍼로 빼고 호출로 교체:

```rust
/// 아군 총알 1발 스폰(방향으로 회전). 확산탄·노바 등 모든 아군 탄이 공유한다.
pub fn spawn_bullet(commands: &mut Commands, assets: &SpriteAssets, position: Vec2, dir: Vec2) {
    commands.spawn((
        Bullet { life: Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once) },
        Sprite {
            image: assets.bullet.clone(),
            custom_size: Some(Vec2::new(6.0, 14.0)),
            ..default()
        },
        Transform {
            translation: position.extend(Z_ENTITY),
            rotation: Quat::from_rotation_z(dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2),
            ..default()
        },
        Velocity(dir * BULLET_SPEED),
        Collider { radius: BULLET_COLLIDER_RADIUS },
        GameplayEntity,
    ));
}
```
`fire_bullet` 루프 내부는 `spawn_bullet(&mut commands, &assets, nose.truncate(), dir);` 한 줄로. (nose의 z는 우주선 z=Z_ENTITY라 동등.) 테스트 추가:

```rust
    #[test]
    fn spawn_bullet_moves_along_dir() {
        let mut app = App::new();
        let assets = crate::fx::sprites::dummy_sprite_assets();
        app.world_mut()
            .run_system_once(move |mut c: Commands| {
                spawn_bullet(&mut c, &assets, Vec2::ZERO, Vec2::X);
            })
            .unwrap();
        let mut q = app.world_mut().query_filtered::<&Velocity, With<Bullet>>();
        let v = q.single(app.world()).unwrap();
        assert!((v.0.x - BULLET_SPEED).abs() < 1e-3 && v.0.y.abs() < 1e-3);
    }
```

- [ ] **Step 7: HUD 큐 슬롯** — `ui/hud.rs`: ① `SpChargeText` 컴포넌트·스폰 블록·update_hud의 sp 쿼리/블록 삭제. ② 추가:

```rust
/// 특수무기 큐 슬롯(앞에서 i번째). 첫 슬롯 = 다음 발동(불투명), 나머지 반투명.
#[derive(Component)]
pub(super) struct WeaponSlotIcon(usize);
```

스폰(기존 특수무기 row 자리에, 아이콘만 HUD_QUEUE_SLOTS개):

```rust
            // 특수무기 큐(획득 순서, 맨 앞 = 다음 발동)
            root.spawn(row()).with_children(|slots| {
                for i in 0..HUD_QUEUE_SLOTS {
                    slots.spawn((
                        WeaponSlotIcon(i),
                        ImageNode::new(assets.powerup[4].clone()),
                        icon(),
                        Visibility::Hidden,
                    ));
                }
            });
```

갱신 시스템:

```rust
/// 큐 앞 HUD_QUEUE_SLOTS개를 아이콘으로 표시. 첫 슬롯만 불투명(다음 발동).
pub(super) fn update_weapon_slots(
    assets: Res<SpriteAssets>,
    player: Query<&SpecialWeapon, With<Player>>,
    mut slots: Query<(&WeaponSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    let queue = player.single().map(|w| w.queue.clone()).unwrap_or_default();
    for (slot, mut image, mut vis) in &mut slots {
        match queue.get(slot.0) {
            Some(kind) => {
                image.image = weapon_icon(*kind, &assets);
                image.color = if slot.0 == 0 { Color::WHITE } else { Color::srgba(1.0, 1.0, 1.0, 0.55) };
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}
```

import 갱신: `use crate::core::config::HUD_QUEUE_SLOTS;`, `use crate::entities::special_weapon::{weapon_icon, SpecialWeapon};`. `update_hud`에서 `player`/`sp` 파라미터·`charges` 블록 제거. ③ `ui.rs` 루트의 Update 튜플에서 시스템 목록 갱신(`hud::update_weapon_slots` 추가).

- [ ] **Step 8: 검증** — Run: `cargo test 2>&1 | tail -5 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -3`
Expected: 전부 통과(기존 95 − SpCharge 관련 0 + 신규 3 = 98), 경고 0. `cargo run`으로 기존 체감 동일(시작 빔 1, X→빔, HUD 슬롯 1개 표시) 확인 가능.

- [ ] **Step 9: 커밋**

```bash
git add -A src/
git commit -m "feat: 특수무기 카트라이더 큐 전환 + special_weapon 서브모듈 골격

$(printf 'Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>')"
```

---

## Task 2: 산탄 노바 + 유도 미사일

**Files:**
- Create: `src/entities/special_weapon/nova.rs`, `src/entities/special_weapon/missile.rs`, `assets/sprites/src/weapon_nova.svg`, `assets/sprites/src/weapon_missile.svg`, `assets/sprites/src/missile.svg`
- Modify: `src/core/config/combat.rs`, `src/entities/special_weapon.rs`, `src/entities/player.rs`(F1), `src/fx/sprites.rs`

**Interfaces:**
- Consumes: `spawn_bullet`(T1), 큐 디스패치(T1).
- Produces: `SpecialWeaponKind::{ScatterNova, HomingMissile}`, `nova::fire`, `missile::{fire, homing_steer, Homing}`.

- [ ] **Step 1: 상수** — `config/combat.rs`:

```rust
// 특수무기 — 산탄 노바 / 유도 미사일
pub const NOVA_BULLETS: usize = 16;          // 전방위 산탄 수
pub const MISSILE_COUNT: usize = 5;          // 발사 수
pub const MISSILE_SPEED: f32 = 420.0;
pub const MISSILE_TURN_RATE: f32 = 6.0;      // 조향 상한(rad/s)
pub const MISSILE_LIFETIME_SECS: f32 = 2.5;
pub const MISSILE_COLLIDER_RADIUS: f32 = 4.0;
pub const MISSILE_SPREAD: f32 = 0.5;         // 초기 부채꼴 반각(rad)
```

- [ ] **Step 2: 아트** — `weapon_nova.svg`(40×40: 중앙 점 + 8방향 스파이크, 노랑/주황), `weapon_missile.svg`(40×40: 위로 향한 미사일 실루엣, 회색/빨강 팁), `missile.svg`(12×28: bullet 규격, 몸통 회색+팁 빨강+꼬리 불꽃, +Y 방향). 카툰 스타일(두꺼운 외곽선). `fx/sprites.rs`의 구조체/`build_sprite_assets`/`dummy_sprite_assets`에 `weapon_nova`, `weapon_missile`, `missile` 핸들 추가(`powerup:` 배열 근처).

- [ ] **Step 3: `nova.rs`**:

```rust
//! 산탄 노바: 우주선 주변 전방위로 아군 탄을 방출한다(포위 탈출기).

use bevy::prelude::*;

use crate::core::config::NOVA_BULLETS;
use crate::entities::bullet::spawn_bullet;
use crate::fx::sprites::SpriteAssets;

/// n개 균등 각도의 단위 방향 벡터.
pub(super) fn nova_directions(n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let ang = i as f32 / n as f32 * std::f32::consts::TAU;
            Vec2::new(ang.cos(), ang.sin())
        })
        .collect()
}

/// 전방위 NOVA_BULLETS발. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, pos: Vec2) {
    for dir in nova_directions(NOVA_BULLETS) {
        spawn_bullet(commands, assets, pos, dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nova_directions_are_unit_and_distinct() {
        let dirs = nova_directions(16);
        assert_eq!(dirs.len(), 16);
        for d in &dirs {
            assert!((d.length() - 1.0).abs() < 1e-4);
        }
        // 첫 방향과 반대편(8번째)은 반대 방향
        assert!((dirs[0] + dirs[8]).length() < 1e-3);
    }
}
```

- [ ] **Step 4: `missile.rs`**:

```rust
//! 유도 미사일: Bullet 마커를 함께 달아 기존 총알 충돌·보스 피해 경로를 재사용하고,
//! homing_steer가 가장 가까운 표적(소행성/UFO/보스)으로 조향한다.

use bevy::prelude::*;

use crate::core::components::{Collider, Velocity};
use crate::core::config::{
    MISSILE_COLLIDER_RADIUS, MISSILE_COUNT, MISSILE_LIFETIME_SECS, MISSILE_SPEED,
    MISSILE_SPREAD, MISSILE_TURN_RATE, Z_ENTITY,
};
use crate::core::state::GameplayEntity;
use crate::entities::asteroid::Asteroid;
use crate::entities::boss::Boss;
use crate::entities::bullet::Bullet;
use crate::entities::ufo::Ufo;
use crate::fx::sprites::SpriteAssets;

/// 유도 조향 대상 마커(미사일).
#[derive(Component)]
pub struct Homing;

/// 후보 중 from에서 가장 가까운 위치의 인덱스.
pub(super) fn nearest_target(from: Vec2, targets: &[Vec2]) -> Option<usize> {
    targets
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            from.distance_squared(**a)
                .partial_cmp(&from.distance_squared(**b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// 현재 방향을 표적 방향으로 최대 max_turn·dt만큼만 회전(급선회 제한).
pub(super) fn steer_toward(cur_dir: Vec2, to_target: Vec2, max_turn: f32, dt: f32) -> Vec2 {
    let cur = cur_dir.normalize_or_zero();
    let tgt = to_target.normalize_or_zero();
    if cur == Vec2::ZERO || tgt == Vec2::ZERO {
        return cur_dir;
    }
    let cur_a = cur.y.atan2(cur.x);
    let tgt_a = tgt.y.atan2(tgt.x);
    let mut diff = tgt_a - cur_a;
    while diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    let step = diff.clamp(-max_turn * dt, max_turn * dt);
    let a = cur_a + step;
    Vec2::new(a.cos(), a.sin())
}

/// 전방 부채꼴로 미사일 MISSILE_COUNT발. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, transform: &Transform) {
    let base = (transform.rotation * Vec3::Y).truncate();
    let pos = transform.translation.truncate();
    for i in 0..MISSILE_COUNT {
        // -SPREAD..+SPREAD 균등 분포(0 포함 중앙 대칭)
        let t = if MISSILE_COUNT > 1 { i as f32 / (MISSILE_COUNT - 1) as f32 } else { 0.5 };
        let ang = -MISSILE_SPREAD + t * 2.0 * MISSILE_SPREAD;
        let dir = (Quat::from_rotation_z(ang) * base.extend(0.0)).truncate();
        commands.spawn((
            Bullet { life: Timer::from_seconds(MISSILE_LIFETIME_SECS, TimerMode::Once) },
            Homing,
            Sprite {
                image: assets.missile.clone(),
                custom_size: Some(Vec2::new(8.0, 18.0)),
                ..default()
            },
            Transform {
                translation: pos.extend(Z_ENTITY),
                rotation: Quat::from_rotation_z(dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2),
                ..default()
            },
            Velocity(dir * MISSILE_SPEED),
            Collider { radius: MISSILE_COLLIDER_RADIUS },
            GameplayEntity,
        ));
    }
}

/// 미사일을 가장 가까운 표적으로 조향(속력 유지)하고 스프라이트를 진행 방향으로 회전.
#[allow(clippy::type_complexity)]
pub(super) fn homing_steer(
    time: Res<Time>,
    targets: Query<
        &Transform,
        (Or<(With<Asteroid>, With<Ufo>, With<Boss>)>, Without<Homing>),
    >,
    mut missiles: Query<(&mut Transform, &mut Velocity), With<Homing>>,
) {
    let dt = time.delta_secs();
    let positions: Vec<Vec2> = targets.iter().map(|t| t.translation.truncate()).collect();
    if positions.is_empty() {
        return;
    }
    for (mut tf, mut vel) in &mut missiles {
        let pos = tf.translation.truncate();
        if let Some(i) = nearest_target(pos, &positions) {
            let new_dir = steer_toward(vel.0, positions[i] - pos, MISSILE_TURN_RATE, dt);
            vel.0 = new_dir * MISSILE_SPEED;
            tf.rotation = Quat::from_rotation_z(new_dir.y.atan2(new_dir.x) - std::f32::consts::FRAC_PI_2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_picks_closest_and_none_when_empty() {
        let ts = vec![Vec2::new(100.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(-50.0, 0.0)];
        assert_eq!(nearest_target(Vec2::ZERO, &ts), Some(1));
        assert_eq!(nearest_target(Vec2::ZERO, &[]), None);
    }

    #[test]
    fn steer_is_clamped_and_converges() {
        // 90° 꺾어야 하는데 max_turn*dt가 작으면 그만큼만 회전
        let d = steer_toward(Vec2::X, Vec2::Y, 1.0, 0.5); // 최대 0.5rad
        let angle = d.y.atan2(d.x);
        assert!((angle - 0.5).abs() < 1e-4);
        // dt가 충분하면 표적 방향에 도달
        let d2 = steer_toward(Vec2::X, Vec2::Y, 10.0, 1.0);
        assert!((d2 - Vec2::Y).length() < 1e-4);
    }
}
```

- [ ] **Step 5: 루트 통합** — `special_weapon.rs`: ① `pub mod missile; pub mod nova;` 추가. ② Kind에 `ScatterNova, HomingMissile` 추가. ③ `weapon_icon` arm: `ScatterNova => assets.weapon_nova.clone(), HomingMissile => assets.weapon_missile.clone(),`. ④ `pick_weapon_kind`를 3등분으로:

```rust
pub fn pick_weapon_kind(roll: f32) -> SpecialWeaponKind {
    match roll {
        r if r < 1.0 / 3.0 => SpecialWeaponKind::LaserBeam,
        r if r < 2.0 / 3.0 => SpecialWeaponKind::ScatterNova,
        _ => SpecialWeaponKind::HomingMissile,
    }
}
```

⑤ 디스패치 arm 추가:

```rust
        SpecialWeaponKind::ScatterNova => nova::fire(&mut commands, &assets, transform.translation.truncate()),
        SpecialWeaponKind::HomingMissile => missile::fire(&mut commands, &assets, transform),
```

⑥ 플러그인 Update 튜플에 `missile::homing_steer` 추가. ⑦ 테스트 추가(루트): 노바 발동 시 Bullet 16개, 미사일 발동 시 Homing 5개(activate_special run_system_once, 큐에 해당 kind 1개), `pick_weapon_kind` 3버킷 도달.

- [ ] **Step 6: F1** — player.rs `debug_fill_hud`: LaserBeam push 아래에 `ScatterNova`, `HomingMissile` push 추가.

- [ ] **Step 7: 검증 + 커밋** — `cargo build`(PNG 3개 생성 확인: weapon_nova/weapon_missile/missile) + test + clippy. 커밋: `feat: 특수무기 산탄 노바 + 유도 미사일 추가`.

---

## Task 3: 충격파 + 공격형 실드 (라인업 완성)

**Files:**
- Create: `src/entities/special_weapon/shockwave.rs`, `src/entities/special_weapon/shield_burst.rs`, `assets/sprites/src/weapon_shockwave.svg`, `assets/sprites/src/weapon_burst.svg`
- Modify: `src/core/config/combat.rs`, `src/entities/special_weapon.rs`, `src/entities/player.rs`(F1), `src/fx/sprites.rs`

**Interfaces:**
- Consumes: 큐 디스패치, `Shield`(player), try_despawn 관행.
- Produces: `SpecialWeaponKind::{Shockwave, ShieldBurst}`, `shockwave::{fire, apply_shockwave, ring_fade}`, `shield_burst::{fire, burst_tick, burst_contact_destroy, ShieldBurst}`.

- [ ] **Step 1: 상수** — `config/combat.rs`:

```rust
// 특수무기 — 충격파 / 공격형 실드
pub const SHOCKWAVE_RADIUS: f32 = 220.0;
pub const SHOCKWAVE_IMPULSE: f32 = 380.0;   // 밀쳐낼 속도(u/s)
pub const SHOCKWAVE_RING_SECS: f32 = 0.35;  // 링 연출 시간
pub const SHIELD_BURST_SECS: f32 = 4.0;
pub const BURST_FLASH_HZ: f32 = 12.0;       // 우주선 번쩍임 토글 빈도(대략 Hz 감각)
pub const BURST_PULSE: f32 = 0.05;          // 크기 펄스 진폭(±5%)
```

- [ ] **Step 2: 아트** — `weapon_shockwave.svg`(40×40: 동심원 파문, 청록), `weapon_burst.svg`(40×40: 우주선 실루엣 + 방사 광, 주황). `fx/sprites.rs`에 `weapon_shockwave`, `weapon_burst` 핸들 추가.

- [ ] **Step 3: `shockwave.rs`** — 발동은 Blast 엔티티를 남기고, 적용 시스템이 한 프레임 처리 후 제거(ECS 관행). 링 연출은 실드 스프라이트 확대·페이드:

```rust
//! 충격파: 반경 내 적탄 소멸 + 소행성/UFO 밀쳐내기 + Small 소행성 파괴(방어형 탈출기).

use bevy::prelude::*;

use crate::core::components::{Collider, Velocity};
use crate::core::config::{
    EXPLOSION_PARTICLES, SHAKE_EXPLOSION, SHOCKWAVE_IMPULSE, SHOCKWAVE_RADIUS,
    SHOCKWAVE_RING_SECS, Z_SHIELD,
};
use crate::core::logic::AsteroidSize;
use crate::core::state::{GameplayEntity, Score};
use crate::entities::asteroid::Asteroid;
use crate::entities::bullet::EnemyBullet;
use crate::entities::ufo::Ufo;
use crate::fx::effects::spawn_explosion;
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

/// 이번 프레임에 적용할 충격파(발동 지점). apply_shockwave가 소비 후 제거.
#[derive(Component)]
pub struct ShockwaveBlast {
    pub center: Vec2,
}

/// 링 연출(확대 + 페이드).
#[derive(Component)]
pub(super) struct ShockRing {
    life: Timer,
}

/// 반경 판정(순수).
pub(super) fn shockwave_hits(center: Vec2, radius: f32, pos: Vec2) -> bool {
    center.distance_squared(pos) <= radius * radius
}

/// 발동: Blast + 링 연출 스폰. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, pos: Vec2) {
    commands.spawn(ShockwaveBlast { center: pos });
    commands.spawn((
        ShockRing { life: Timer::from_seconds(SHOCKWAVE_RING_SECS, TimerMode::Once) },
        Sprite {
            image: assets.shield.clone(),
            custom_size: Some(Vec2::splat(10.0)),
            color: Color::srgba(0.5, 1.0, 1.0, 0.8),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, Z_SHIELD),
        GameplayEntity,
    ));
}

/// Blast 적용: 적탄 소멸, Small 소행성 파괴(점수+폭발), 소행성/UFO 밀쳐내기.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn apply_shockwave(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut shake: MessageWriter<ShakeEvent>,
    blasts: Query<(Entity, &ShockwaveBlast)>,
    enemy_bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
    mut asteroids: Query<(Entity, &Transform, &mut Velocity, &Asteroid)>,
    mut ufos: Query<(Entity, &Transform, &mut Velocity), (With<Ufo>, Without<Asteroid>)>,
) {
    for (blast_e, blast) in &blasts {
        let c = blast.center;
        for (e, tf) in &enemy_bullets {
            if shockwave_hits(c, SHOCKWAVE_RADIUS, tf.translation.truncate()) {
                commands.entity(e).try_despawn();
            }
        }
        for (e, tf, mut vel, asteroid) in &mut asteroids {
            let p = tf.translation.truncate();
            if !shockwave_hits(c, SHOCKWAVE_RADIUS, p) {
                continue;
            }
            if asteroid.size == AsteroidSize::Small {
                // Small은 파괴(점수+폭발, 분열 없음)
                commands.entity(e).try_despawn();
                score.0 += asteroid.size.score();
                spawn_explosion(&mut commands, &assets, p, EXPLOSION_PARTICLES);
            } else {
                // 중심 → 바깥으로 강하게 밀침
                let out = (p - c).normalize_or_zero();
                vel.0 = out * SHOCKWAVE_IMPULSE;
            }
        }
        for (_e, tf, mut vel) in &mut ufos {
            let p = tf.translation.truncate();
            if shockwave_hits(c, SHOCKWAVE_RADIUS, p) {
                let out = (p - c).normalize_or_zero();
                vel.0 = out * SHOCKWAVE_IMPULSE;
            }
        }
        shake.write(ShakeEvent(SHAKE_EXPLOSION));
        commands.entity(blast_e).despawn();
    }
}

/// 링을 SHOCKWAVE_RADIUS*2까지 키우며 페이드아웃 후 제거.
pub(super) fn ring_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut rings: Query<(Entity, &mut ShockRing, &mut Sprite)>,
) {
    for (e, mut ring, mut sprite) in &mut rings {
        ring.life.tick(time.delta());
        let f = ring.life.fraction();
        sprite.custom_size = Some(Vec2::splat(10.0 + f * (SHOCKWAVE_RADIUS * 2.0 - 10.0)));
        sprite.color.set_alpha(0.8 * (1.0 - f));
        if ring.life.is_finished() {
            commands.entity(e).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_inside_not_outside() {
        assert!(shockwave_hits(Vec2::ZERO, 220.0, Vec2::new(200.0, 0.0)));
        assert!(!shockwave_hits(Vec2::ZERO, 220.0, Vec2::new(230.0, 0.0)));
    }
}
```

> UFO가 자체 이동 시스템에서 x속도를 다시 쓰더라도 한 프레임 밀침 + 이후 복귀는 수용(연출 목적). `Timer::fraction()`은 Bevy 0.19 API — 없으면 `elapsed_secs()/duration` 사용.

- [ ] **Step 4: `shield_burst.rs`**:

```rust
//! 공격형 실드: 몇 초간 무적(기존 Shield 재사용) + 접촉한 소행성/UFO/적탄을 파괴.
//! 표현은 버블 없이 우주선 자체 번쩍임(주황 고주파 틴트) + 크기 펄스(±5%), 만료 시 원복.

use bevy::prelude::*;

use crate::core::components::Collider;
use crate::core::config::{
    BURST_FLASH_HZ, BURST_PULSE, EXPLOSION_PARTICLES, SHIELD_BURST_SECS, SHIP_COLLIDER_RADIUS,
};
use crate::core::logic::circles_overlap;
use crate::core::state::Score;
use crate::entities::asteroid::Asteroid;
use crate::entities::bullet::EnemyBullet;
use crate::entities::player::{Player, Shield};
use crate::entities::ufo::Ufo;
use crate::fx::effects::spawn_explosion;
use crate::fx::sprites::{sprite_size_for, SpriteAssets};

/// 공격형 실드 지속 타이머(플레이어에 부착).
#[derive(Component)]
pub struct ShieldBurst {
    pub timer: Timer,
}

/// 번쩍임 on/off(고주파 토글, 순수).
pub(super) fn burst_flash_on(t: f32) -> bool {
    (t * BURST_FLASH_HZ).sin() > 0.0
}

/// 크기 펄스 배율(1±BURST_PULSE, 순수).
pub(super) fn burst_pulse_scale(t: f32) -> f32 {
    1.0 + BURST_PULSE * (t * BURST_FLASH_HZ * 0.8).sin()
}

/// 발동: ShieldBurst + 기존 Shield(같은 시간, 무적 재사용) 부여. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, player: Entity) {
    commands.entity(player).insert((
        ShieldBurst { timer: Timer::from_seconds(SHIELD_BURST_SECS, TimerMode::Once) },
        Shield(Timer::from_seconds(SHIELD_BURST_SECS, TimerMode::Once)),
    ));
}

/// 지속 관리 + 우주선 번쩍임/펄스. 만료 프레임에 색·크기 원복(필수).
pub(super) fn burst_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut ShieldBurst, &mut Sprite), With<Player>>,
) {
    for (e, mut burst, mut sprite) in &mut q {
        burst.timer.tick(time.delta());
        if burst.timer.is_finished() {
            sprite.color = Color::WHITE;
            sprite.custom_size = Some(sprite_size_for(SHIP_COLLIDER_RADIUS));
            commands.entity(e).remove::<ShieldBurst>();
            continue;
        }
        let t = time.elapsed_secs();
        sprite.color = if burst_flash_on(t) {
            Color::srgb(1.0, 0.75, 0.3) // 주황 섬광
        } else {
            Color::WHITE
        };
        sprite.custom_size = Some(sprite_size_for(SHIP_COLLIDER_RADIUS) * burst_pulse_scale(t));
    }
}

/// 지속 중 접촉한 소행성/UFO/적탄 파괴(점수+폭발, 분열 없음; 보스는 무적만).
#[allow(clippy::type_complexity)]
pub(super) fn burst_contact_destroy(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    players: Query<(&Transform, &Collider), (With<Player>, With<ShieldBurst>)>,
    asteroids: Query<(Entity, &Transform, &Collider, &Asteroid), Without<Player>>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo), Without<Player>>,
    enemy_bullets: Query<(Entity, &Transform, &Collider), (With<EnemyBullet>, Without<Player>)>,
) {
    let Ok((ptf, pcol)) = players.single() else { return };
    let ppos = ptf.translation.truncate();
    for (e, tf, col, asteroid) in &asteroids {
        if circles_overlap(ppos, pcol.radius, tf.translation.truncate(), col.radius) {
            commands.entity(e).try_despawn();
            score.0 += asteroid.size.score();
            spawn_explosion(&mut commands, &assets, tf.translation.truncate(), EXPLOSION_PARTICLES);
        }
    }
    for (e, tf, col, ufo) in &ufos {
        if circles_overlap(ppos, pcol.radius, tf.translation.truncate(), col.radius) {
            commands.entity(e).try_despawn();
            score.0 += ufo.size.score();
            spawn_explosion(&mut commands, &assets, tf.translation.truncate(), EXPLOSION_PARTICLES);
        }
    }
    for (e, tf, col) in &enemy_bullets {
        if circles_overlap(ppos, pcol.radius, tf.translation.truncate(), col.radius) {
            commands.entity(e).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_stays_within_band_and_flash_toggles() {
        let mut saw_on = false;
        let mut saw_off = false;
        for i in 0..200 {
            let t = i as f32 * 0.01;
            let s = burst_pulse_scale(t);
            assert!(s >= 1.0 - BURST_PULSE - 1e-4 && s <= 1.0 + BURST_PULSE + 1e-4);
            if burst_flash_on(t) { saw_on = true } else { saw_off = true }
        }
        assert!(saw_on && saw_off, "번쩍임이 켜짐/꺼짐을 오가야 한다");
    }
}
```

- [ ] **Step 5: 루트 통합** — ① `pub mod shield_burst; pub mod shockwave;` ② Kind에 `Shockwave, ShieldBurst` 추가. ③ `weapon_icon` arm 2개(`weapon_shockwave`/`weapon_burst`). ④ `pick_weapon_kind` 5등분(0.2 버킷, 순서: LaserBeam·ScatterNova·HomingMissile·Shockwave·ShieldBurst). ⑤ 디스패치: `activate_special`의 플레이어 쿼리를 `(Entity, &Transform, &mut SpecialWeapon)`로 확장하고 arm 추가:

```rust
        SpecialWeaponKind::Shockwave => shockwave::fire(&mut commands, &assets, transform.translation.truncate()),
        SpecialWeaponKind::ShieldBurst => shield_burst::fire(&mut commands, player_entity),
```

⑥ 플러그인 Update 튜플에 `shockwave::apply_shockwave, shockwave::ring_fade, shield_burst::burst_tick, shield_burst::burst_contact_destroy` 추가. ⑦ 테스트: `pick_weapon_kind` 5버킷 전부 도달, 발동 스폰 테스트(Shockwave→Blast 1개, ShieldBurst→ShieldBurst+Shield 부여), `burst_contact_destroy`가 접촉 소행성 파괴(run_system_once: 플레이어+ShieldBurst+Collider, 겹친 소행성 → 사라짐).

- [ ] **Step 6: F1** — `debug_fill_hud`에 `Shockwave`, `ShieldBurst` push 추가(5종 완성).

- [ ] **Step 7: 검증 + 커밋** — `cargo build`(PNG 2개) + test + clippy. 실기(`cargo run`, 선택): F1 5회 → X 연타로 5종 순서 발동 확인. 커밋: `feat: 특수무기 충격파 + 공격형 실드 추가(라인업 완성)`.

---

## Self-Review (작성자 체크)

- **스펙 커버리지**: 큐 FIFO(T1) / Kind 이동·의존 교정(T1) / 서브모듈 5개(T1–T3) / 드롭 kind별 아이콘(T1 구조+T2·T3 아이콘) / HUD 슬롯(T1) / 노바16·미사일5·충격파·공격실드(T2·T3) / 번쩍임+펄스+원복(T3) / F1 5종(T1–T3 누적) — 스펙 §3 전 항목 존재. ✓
- **dead_code**: T1의 `pick_weapon_kind`·`weapon_icon`은 T1의 powerup/HUD가 즉시 소비. `spawn_bullet`은 fire_bullet이 소비. T2·T3 상수·함수·아이콘은 각 태스크 내 소비. 임시 allow 불필요. ✓
- **타입 일관성**: `SpecialWeapon.queue`(T1)를 powerup(push_back)·HUD(get(i))·player(초기화)·F1이 일관 사용. `fire` 시그니처: beam/missile=&Transform, nova/shockwave=Vec2, shield_burst=Entity — 디스패치 호출과 일치. `Homing`+`Bullet` 조합은 기존 bullet_vs_*·boss_combat과 그대로 호환(추가 마커일 뿐). ✓
- **쿼리 안전**: `homing_steer` 표적 쿼리에 `Without<Homing>`(읽기 Transform vs 미사일 쓰기 Transform 분리). `burst_contact_destroy`·`apply_shockwave`의 Without/disjoint 필터 명시. ✓
- **동작 보존(T1)**: 시작 [빔]×STARTING_SPECIAL_CHARGES, X→빔, 드롭 아이콘 동일(weapon_icon(LaserBeam)=powerup[4]), 기존 빔 판정 경로 불변(`pub use beam::SpecialBeam`). 기존 테스트 중 charges 참조 2곳(player 스폰 테스트·powerup index 테스트)만 갱신 명시. ✓
- **주의**: powerup의 `powerup_sprite_index(SpecialWeapon(_))=4` 테스트는 유지(인덱스 함수 자체는 존치 — 비특수 드롭 이미지 선택에 계속 사용). `Timer::fraction()` 미존재 시 대체식 명시. hud 테스트(hud_shows_score_lives_wave)는 update_hud 시그니처 축소 후에도 그대로 통과.
