use std::time::Duration;

use bevy::prelude::*;

use crate::core::components::{Collider, EdgeReflect, Velocity};
use crate::core::config::{
    BEAM_BOSS_DAMAGE, BEAM_LENGTH, BEAM_WIDTH, BOSS_BASE_HEALTH, BOSS_ENTRANCE_SHAKE,
    BOSS_HEALTH_PER_CYCLE, BOSS_SCORE_BONUS, BOSS_TRACK, BULLET_BOSS_DAMAGE, EXPLOSION_PARTICLES,
    GOLEM_ATTACK_INTERVAL, GOLEM_DRIFT_SPEED, GOLEM_STEER, ICE_GOLEM_HEALTH_MUL, SHAKE_EXPLOSION,
    UFO_BULLET_SPEED, Z_ENTITY,
};
use crate::core::logic::{aim_direction, circles_overlap, segment_circle_hit, AsteroidSize};
use crate::core::config::STARTING_LIVES;
use crate::core::state::{GameState, GameplayEntity, Lives, Score};
use crate::entities::asteroid::{random_velocity, spawn_asteroid};
use crate::entities::bullet::{spawn_enemy_bullet, Bullet};
use crate::entities::player::Player;
use crate::entities::special_weapon::SpecialBeam;
use crate::entities::ufo::{spawn_ufo, UfoSize};
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::effects::spawn_explosion;
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;
use crate::systems::stage::{advance_stage, spawn_first_wave_of_stage, Progression, ThemeId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    MotherRock,
    Mothership,
    BlazingCore,
    IceGolem,
}

pub fn boss_for_theme(t: ThemeId) -> BossKind {
    match t {
        ThemeId::AsteroidBelt => BossKind::MotherRock,
        ThemeId::AlienFleet => BossKind::Mothership,
        ThemeId::SolarFlare => BossKind::BlazingCore,
        ThemeId::FrozenField => BossKind::IceGolem,
    }
}

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
    pub phase: u8,
}

/// 보스 종류별 기준 체력(모암이 가장 높음) × 사이클 증가.
pub fn boss_max_health(kind: BossKind, cycle: u32) -> f32 {
    let base = match kind {
        BossKind::MotherRock => BOSS_BASE_HEALTH * 1.5,
        BossKind::Mothership => BOSS_BASE_HEALTH,
        BossKind::BlazingCore => BOSS_BASE_HEALTH * 1.2,
        BossKind::IceGolem => BOSS_BASE_HEALTH * ICE_GOLEM_HEALTH_MUL,
    };
    base + cycle as f32 * BOSS_HEALTH_PER_CYCLE
}

/// 체력을 깎고, 0 이하이면 true(격파).
pub fn apply_boss_damage(boss: &mut Boss, dmg: f32) -> bool {
    boss.health -= dmg;
    boss.health <= 0.0
}

pub fn boss_image(kind: BossKind, assets: &SpriteAssets) -> Handle<Image> {
    match kind {
        BossKind::MotherRock => assets.boss_mother_rock.clone(),
        BossKind::Mothership => assets.boss_mothership.clone(),
        BossKind::BlazingCore => assets.boss_blazing_core.clone(),
        BossKind::IceGolem => assets.boss_ice_golem.clone(),
    }
}

pub fn boss_radius(kind: BossKind) -> f32 {
    match kind {
        BossKind::MotherRock => 72.0,
        BossKind::Mothership => 90.0,
        BossKind::BlazingCore => 60.0,
        BossKind::IceGolem => 66.0,
    }
}

pub fn spawn_boss(commands: &mut Commands, assets: &SpriteAssets, kind: BossKind, cycle: u32) {
    let hp = boss_max_health(kind, cycle);
    let r = boss_radius(kind);
    let mut e = commands.spawn((
        Boss { kind, health: hp, max_health: hp, phase: 0 },
        BossAttack { timer: Timer::from_seconds(attack_interval(kind), TimerMode::Repeating), shots: 0 },
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

/// 보스 종류별 공격 주기(초).
fn attack_interval(kind: BossKind) -> f32 {
    match kind {
        BossKind::MotherRock => 2.8,
        BossKind::Mothership => 2.1,
        BossKind::BlazingCore => 2.4,
        BossKind::IceGolem => GOLEM_ATTACK_INTERVAL,
    }
}

/// 골렘 공격 교대: 짝수 발 = 팔 휘두르기(부채꼴 파편), 홀수 발 = 내려찍기(방사 탄).
pub fn golem_attack_is_sweep(shots: u32) -> bool {
    shots.is_multiple_of(2)
}

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

#[derive(Component)]
pub struct BossAttack {
    pub timer: Timer,
    pub shots: u32,
}

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (boss_movement, boss_attack, boss_combat, golem_phase_transition, boss_entrance)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// 보스 x를 패턴 위치에서 플레이어 x 쪽으로 factor만큼 당긴다(수평 추적). 플레이어가 없으면 패턴 유지.
pub fn track(pattern_x: f32, player_x: Option<f32>, factor: f32) -> f32 {
    match player_x {
        Some(px) => pattern_x + (px - pattern_x) * factor,
        None => pattern_x,
    }
}

/// 보스 종류별 이동 패턴 + 플레이어 수평 추적(위협감). 골렘은 속도를 플레이어 쪽으로 약간 조향한다.
fn boss_movement(
    time: Res<Time>,
    players: Query<&Transform, (With<Player>, Without<Boss>)>,
    mut q: Query<(&Boss, &mut Transform, Option<&mut Velocity>)>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    let player_pos = players.single().ok().map(|p| p.translation.truncate());
    let player_x = player_pos.map(|p| p.x);
    for (boss, mut tf, vel) in &mut q {
        match boss.kind {
            // 모암: 느린 배회 + 수평 추적
            BossKind::MotherRock => {
                tf.translation.x = track((t * 0.3).sin() * 250.0, player_x, BOSS_TRACK);
                tf.translation.y = 110.0 + (t * 0.4).sin() * 60.0;
            }
            // 모함: 상단 좌우 스윕 + 수평 추적
            BossKind::Mothership => {
                tf.translation.x = track((t * 0.8).sin() * 300.0, player_x, BOSS_TRACK);
                tf.translation.y = 150.0;
            }
            // 화염 코어: 상단 부유 + 수평 추적
            BossKind::BlazingCore => {
                tf.translation.x = track((t * 1.2).sin() * 30.0, player_x, BOSS_TRACK);
                tf.translation.y = 150.0 + (t * 2.0).sin() * 15.0;
            }
            // 얼음 골렘: 드리프트/반사 + 플레이어 쪽으로 약한 조향(유도).
            BossKind::IceGolem => {
                if let (Some(mut v), Some(pp)) = (vel, player_pos) {
                    let desired = (pp - tf.translation.truncate()).normalize_or_zero();
                    if desired != Vec2::ZERO {
                        let speed = v.0.length();
                        let cur = v.0.normalize_or_zero();
                        let steered = cur.lerp(desired, (GOLEM_STEER * dt).min(1.0)).normalize_or_zero();
                        v.0 = steered * speed;
                    }
                }
            }
        }
    }
}

/// 보스 등장 임팩트: 스폰되는 순간 강한 화면 흔들림 + 굉음으로 위압감을 준다.
fn boss_entrance(
    new_bosses: Query<(), Added<Boss>>,
    mut shake: MessageWriter<ShakeEvent>,
    mut sfx: MessageWriter<SfxEvent>,
) {
    for _ in &new_bosses {
        shake.write(ShakeEvent(BOSS_ENTRANCE_SHAKE));
        sfx.write(SfxEvent(Sfx::Explosion));
    }
}

/// 보스 종류별 주기적 공격.
fn boss_attack(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    time: Res<Time>,
    mut bosses: Query<(&Boss, &mut BossAttack, &Transform)>,
    players: Query<&Transform, With<Player>>,
) {
    let player_pos = players.single().ok().map(|t| t.translation.truncate());
    for (boss, mut atk, tf) in &mut bosses {
        atk.timer.tick(time.delta());
        if !atk.timer.is_finished() {
            continue;
        }
        let shots = atk.shots;
        atk.shots = atk.shots.wrapping_add(1);
        let pos = tf.translation.truncate();
        match boss.kind {
            // 모암: 소행성 파편 방사
            BossKind::MotherRock => {
                for _ in 0..3 {
                    let size = AsteroidSize::Medium;
                    spawn_asteroid(&mut commands, &assets, size, pos, random_velocity(size));
                }
            }
            // 모함: 플레이어 조준 3-way + 가끔 소형 UFO 소환
            BossKind::Mothership => {
                if let Some(pp) = player_pos {
                    let dir = aim_direction(pos, pp);
                    for a in [-0.2f32, 0.0, 0.2] {
                        let d = (Quat::from_rotation_z(a) * dir.extend(0.0)).truncate();
                        spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                    }
                }
                use rand::RngExt;
                if rand::rng().random_range(0.0..1.0) < 0.4 {
                    spawn_ufo(&mut commands, &assets, UfoSize::Small, pos.x < 0.0);
                }
            }
            // 화염 코어: 방사형 탄막(전방위 12발)
            BossKind::BlazingCore => {
                let n = 12;
                for i in 0..n {
                    let ang = i as f32 / n as f32 * std::f32::consts::TAU;
                    let d = Vec2::new(ang.cos(), ang.sin());
                    spawn_enemy_bullet(&mut commands, &assets, pos, d * UFO_BULLET_SPEED);
                }
            }
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
        }
    }
}

/// 총알/빔이 보스 체력을 깎고, 격파 시 폭발·보너스·다음 스테이지로.
#[allow(clippy::too_many_arguments)]
fn boss_combat(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut prog: ResMut<Progression>,
    mut shake: MessageWriter<ShakeEvent>,
    mut sfx: MessageWriter<SfxEvent>,
    mut bosses: Query<(Entity, &mut Boss, &Transform, &Collider)>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    mut beams: Query<&mut SpecialBeam>,
) {
    for (boss_e, mut boss, boss_tf, boss_col) in &mut bosses {
        let bpos = boss_tf.translation.truncate();
        let mut defeated = false;
        for (be, btf, bcol) in &bullets {
            if circles_overlap(bpos, boss_col.radius, btf.translation.truncate(), bcol.radius) {
                commands.entity(be).despawn();
                if apply_boss_damage(&mut boss, BULLET_BOSS_DAMAGE) {
                    defeated = true;
                    break;
                }
            }
        }
        if !defeated {
            for mut beam in &mut beams {
                if !beam.damaged_boss
                    && segment_circle_hit(beam.origin, beam.dir, BEAM_LENGTH, BEAM_WIDTH * 0.5, bpos, boss_col.radius)
                {
                    beam.damaged_boss = true;
                    if apply_boss_damage(&mut boss, BEAM_BOSS_DAMAGE) {
                        defeated = true;
                        break;
                    }
                }
            }
        }
        if defeated {
            commands.entity(boss_e).despawn();
            spawn_explosion(&mut commands, &assets, bpos, EXPLOSION_PARTICLES * 3);
            shake.write(ShakeEvent(SHAKE_EXPLOSION * 2.0));
            sfx.write(SfxEvent(Sfx::Explosion));
            score.0 += BOSS_SCORE_BONUS;
            advance_stage(&mut prog);
            // 다음 스테이지 진입 시 목숨을 기본치 이상으로 리필.
            lives.0 = lives.0.max(STARTING_LIVES);
            spawn_first_wave_of_stage(&mut commands, &assets, &prog);
        }
    }
}

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
        let mut b = Boss { kind: BossKind::Mothership, health: 3.0, max_health: 10.0, phase: 0 };
        assert!(!apply_boss_damage(&mut b, 1.0)); // 2 남음
        assert!((b.health - 2.0).abs() < 1e-6);
        assert!(apply_boss_damage(&mut b, 5.0)); // 0 이하 → 격파
    }

    #[test]
    fn ice_golem_is_frozen_field_boss_and_tanky() {
        assert_eq!(boss_for_theme(ThemeId::FrozenField), BossKind::IceGolem);
        // 같은 사이클에서 골렘이 모함(기준)보다 체력이 높다.
        assert!(boss_max_health(BossKind::IceGolem, 0) > boss_max_health(BossKind::Mothership, 0));
    }

    #[test]
    fn golem_alternates_sweep_and_slam() {
        assert!(golem_attack_is_sweep(0)); // 첫 발 = 팔 휘두르기
        assert!(!golem_attack_is_sweep(1)); // 다음 = 내려찍기
        assert!(golem_attack_is_sweep(2));
    }

    #[test]
    fn golem_phase_thresholds() {
        let m = 30.0;
        assert_eq!(golem_phase(30.0, m), 0); // 만피
        assert_eq!(golem_phase(20.1, m), 0); // >2/3
        assert_eq!(golem_phase(15.0, m), 1); // 1/3~2/3
        assert_eq!(golem_phase(5.0, m), 2); // <1/3
    }

    #[test]
    fn golem_phase_scales_monotonic() {
        assert!(golem_scale(2) < golem_scale(0)); // 작아짐
        assert!(golem_speed_mul(2) > golem_speed_mul(0)); // 빨라짐
        assert!(golem_attack_mul(2) < golem_attack_mul(0)); // 주기 짧아짐
    }

    #[test]
    fn track_pulls_toward_player_and_holds_without() {
        assert!((track(0.0, Some(100.0), 0.5) - 50.0).abs() < 1e-4); // 절반 당김
        assert_eq!(track(30.0, None, 0.5), 30.0); // 플레이어 없으면 패턴 유지
        assert_eq!(track(30.0, Some(100.0), 0.0), 30.0); // factor 0이면 고정
    }
}
