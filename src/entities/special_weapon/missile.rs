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
