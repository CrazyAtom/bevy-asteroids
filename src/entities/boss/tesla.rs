//! 테슬라 코어(EM Storm 보스) 전용: 블링크 순간이동(예고 축소 + 워프 피드백),
//! 폭풍-동기 EMP 방사 링, 조준 체인 전격. 데이터·스폰·디스패치는 부모 `boss` 모듈이 담당한다.

use bevy::prelude::*;

use crate::core::config::{
    BLINK_TELEGRAPH_SECS, SHAKE_EXPLOSION, STORM_EMP_THRESHOLD, UFO_BULLET_SPEED,
};
use crate::core::logic::{aim_direction, storm_pulse};
use crate::entities::bullet::spawn_enemy_bullet;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

use super::{boss_radius, Boss, BossKind};

/// 테슬라 코어 순간이동 타이머.
#[derive(Component)]
pub struct Blink {
    pub timer: Timer,
}

/// storm_pulse가 임계를 상향 돌파하는 순간만 true(EMP 1회 발동용 상승 엣지).
pub fn storm_emp_triggers(prev_pulse: f32, cur_pulse: f32) -> bool {
    prev_pulse < STORM_EMP_THRESHOLD && cur_pulse >= STORM_EMP_THRESHOLD
}

/// 테슬라 코어: 조준 체인 전격(EMP는 폭풍 피크에 동기화되어 storm_emp에서 발동). boss_attack 디스패치에서 호출.
pub(super) fn attack(
    commands: &mut Commands,
    assets: &SpriteAssets,
    pos: Vec2,
    player_pos: Option<Vec2>,
) {
    if let Some(pp) = player_pos {
        let dir = aim_direction(pos, pp);
        for a in [-0.25f32, 0.0, 0.25] {
            let d = (Quat::from_rotation_z(a) * dir.extend(0.0)).truncate();
            spawn_enemy_bullet(commands, assets, pos, d * UFO_BULLET_SPEED);
        }
    }
}

/// 테슬라 코어 블링크: 타이머마다 화면 내 임의 위치로 순간이동. 순간이동 직전엔 예고로 축소했다가
/// 이동 직후 원복한다.
pub(super) fn boss_blink(
    time: Res<Time>,
    mut sfx: MessageWriter<SfxEvent>,
    mut shake: MessageWriter<ShakeEvent>,
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
            let mut rng = rand::rng();
            tf.translation.x = rng.random_range(-crate::core::config::HALF_WIDTH * 0.8..crate::core::config::HALF_WIDTH * 0.8);
            tf.translation.y = rng.random_range(0.0..crate::core::config::HALF_HEIGHT * 0.7);
            sprite.custom_size = Some(Vec2::splat(base)); // 원복
            // 순간이동 피드백: 워프 사운드 + 소폭 흔들림(무음 예고 방지 — 적대적 리뷰 반영).
            sfx.write(SfxEvent(Sfx::Hyperspace));
            shake.write(ShakeEvent(SHAKE_EXPLOSION));
        }
    }
}

/// 폭풍 피크(storm_pulse 상승 엣지)에 테슬라 코어가 EMP 방사 링을 쏜다.
pub(super) fn storm_emp(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    time: Res<Time>,
    mut prev: Local<f32>,
    mut shake: MessageWriter<ShakeEvent>,
    bosses: Query<(&Boss, &Transform)>,
) {
    let cur = storm_pulse(time.elapsed_secs());
    let fire = storm_emp_triggers(*prev, cur);
    *prev = cur;
    if !fire {
        return;
    }
    let mut fired = false;
    for (boss, tf) in &bosses {
        if boss.kind != BossKind::TeslaCore {
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emp_triggers_on_rising_edge_only() {
        // 임계 아래→위로 오를 때만 true(상승 엣지), 이미 위이거나 내려갈 땐 false.
        assert!(storm_emp_triggers(0.5, 0.7)); // 상승 돌파
        assert!(!storm_emp_triggers(0.7, 0.8)); // 이미 위
        assert!(!storm_emp_triggers(0.8, 0.5)); // 하강
        assert!(!storm_emp_triggers(0.3, 0.5)); // 아래 유지
    }
}
