use bevy::prelude::*;

use crate::core::components::{Collider, EdgeReflect, Velocity};
use crate::core::config::{
    BEAM_BOSS_DAMAGE, BEAM_LENGTH, BEAM_WIDTH, BOSS_BASE_HEALTH, BOSS_HEALTH_PER_CYCLE,
    BOSS_SCORE_BONUS, BULLET_BOSS_DAMAGE, EXPLOSION_PARTICLES, GOLEM_ATTACK_INTERVAL,
    GOLEM_DRIFT_SPEED, ICE_GOLEM_HEALTH_MUL, SHAKE_EXPLOSION, UFO_BULLET_SPEED, Z_ENTITY,
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
    // TODO(Task 7): 다단계 페이즈(golem_phase_transition)에서 읽음. 그때까지 임시 억제.
    #[allow(dead_code)]
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
        BossKind::MotherRock => 55.0,
        BossKind::Mothership => 70.0,
        BossKind::BlazingCore => 45.0,
        BossKind::IceGolem => 50.0,
    }
}

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

/// 보스 종류별 공격 주기(초).
fn attack_interval(kind: BossKind) -> f32 {
    match kind {
        BossKind::MotherRock => 2.8,
        BossKind::Mothership => 2.1,
        BossKind::BlazingCore => 2.4,
        BossKind::IceGolem => GOLEM_ATTACK_INTERVAL,
    }
}

#[derive(Component)]
pub struct BossAttack {
    pub timer: Timer,
}

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (boss_movement, boss_attack, boss_combat).run_if(in_state(GameState::Playing)),
        );
    }
}

/// 보스 종류별 이동 패턴.
fn boss_movement(time: Res<Time>, mut q: Query<(&Boss, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (boss, mut tf) in &mut q {
        match boss.kind {
            // 모암: 느린 배회
            BossKind::MotherRock => {
                tf.translation.x = (t * 0.3).sin() * 250.0;
                tf.translation.y = 110.0 + (t * 0.4).sin() * 60.0;
            }
            // 모함: 상단 좌우 스윕
            BossKind::Mothership => {
                tf.translation.x = (t * 0.8).sin() * 300.0;
                tf.translation.y = 150.0;
            }
            // 화염 코어: 상단 고정 + 약한 부유
            BossKind::BlazingCore => {
                tf.translation.x = (t * 1.2).sin() * 30.0;
                tf.translation.y = 150.0 + (t * 2.0).sin() * 15.0;
            }
            // 얼음 골렘: Velocity + reflect_or_wrap로 드리프트/반사(여기선 조작 없음).
            BossKind::IceGolem => {}
        }
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
            // 얼음 골렘(기본): 방사형 얼음 탄(Task 6에서 팔 휘두르기/내려찍기로 확장).
            BossKind::IceGolem => {
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
}
