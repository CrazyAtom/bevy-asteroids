//! 얼음 골렘(Frozen Field 보스) 전용: 교대 공격(팔 휘두르기/내려찍기)과
//! 다단계 페이즈(껍질 깨짐 → 축소·가속). 데이터(체력·반경·주기)와 스폰·디스패치는
//! 부모 `boss` 모듈이 담당한다.

use std::time::Duration;

use bevy::prelude::*;

use crate::core::components::{Collider, Velocity};
use crate::core::config::{
    EXPLOSION_PARTICLES, GOLEM_DRIFT_SPEED, SHAKE_EXPLOSION, UFO_BULLET_SPEED,
};
use crate::core::logic::{aim_direction, AsteroidSize};
use crate::entities::asteroid::{random_velocity, spawn_asteroid};
use crate::entities::bullet::spawn_enemy_bullet;
use crate::fx::effects::spawn_explosion;
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

use super::{attack_interval, boss_radius, Boss, BossAttack, BossKind};

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

/// 얼음 골렘: 팔 휘두르기(부채꼴 파편) ↔ 내려찍기(방사 탄) 교대. boss_attack 디스패치에서 호출.
pub(super) fn attack(
    commands: &mut Commands,
    assets: &SpriteAssets,
    pos: Vec2,
    player_pos: Option<Vec2>,
    shots: u32,
) {
    if golem_attack_is_sweep(shots) {
        // 팔 휘두르기: 조준 방향 ±35° 부채꼴로 소형 소행성 파편 5개(벽 반사).
        let base = player_pos
            .map(|pp| aim_direction(pos, pp))
            .unwrap_or(Vec2::NEG_Y);
        for a in [-0.61f32, -0.305, 0.0, 0.305, 0.61] {
            let d = (Quat::from_rotation_z(a) * base.extend(0.0)).truncate();
            let size = AsteroidSize::Small;
            spawn_asteroid(commands, assets, size, pos, d * random_velocity(size).length());
        }
    } else {
        // 내려찍기: 방사형 얼음 탄 12발.
        let n = 12;
        for i in 0..n {
            let ang = i as f32 / n as f32 * std::f32::consts::TAU;
            let d = Vec2::new(ang.cos(), ang.sin());
            spawn_enemy_bullet(commands, assets, pos, d * UFO_BULLET_SPEED);
        }
    }
}

/// 골렘이 체력 구간을 넘으면 껍질이 깨진다: 스프라이트/콜라이더 축소, 가속, 공격 주기 단축, 파편 폭발 연출.
#[allow(clippy::type_complexity)]
pub(super) fn golem_phase_transition(
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
}
