//! 특이점 코어(Black Hole 보스) 전용: 나선 탄, 주기적 흡인 강화(중력 파동),
//! 러쉬(홈 부유 ↔ 플레이어 돌진). 데이터·스폰·디스패치는 부모 `boss` 모듈이 담당한다.

use bevy::prelude::*;

use crate::core::config::{
    GRAVITY_INTENSIFY, GRAVITY_STRENGTH, SHAKE_EXPLOSION, SPIRAL_ARMS, SPIRAL_STEP,
    UFO_BULLET_SPEED,
};
use crate::core::logic::gravity_boost;
use crate::entities::black_hole::BlackHoleActive;
use crate::entities::bullet::spawn_enemy_bullet;
use crate::entities::player::Player;
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

use super::{Boss, BossKind};

/// 특이점 코어 러쉬 타이머·목표. 대부분 홈(중앙 블랙홀 위)에서 부유하다 주기 끝에 플레이어로 돌진.
#[derive(Component)]
pub(super) struct Lunge {
    pub(super) timer: Timer,
    pub(super) target: Vec2,
    pub(super) charging: bool,
}

/// 특이점 코어: 회전하는 방사 = 나선 탄. boss_attack 디스패치에서 호출.
pub(super) fn attack(commands: &mut Commands, assets: &SpriteAssets, pos: Vec2, shots: u32) {
    let base = shots as f32 * SPIRAL_STEP;
    for i in 0..SPIRAL_ARMS {
        let ang = base + i as f32 / SPIRAL_ARMS as f32 * std::f32::consts::TAU;
        let d = Vec2::new(ang.cos(), ang.sin());
        spawn_enemy_bullet(commands, assets, pos, d * UFO_BULLET_SPEED);
    }
}

/// 특이점 코어가 존재하면 블랙홀 흡인력을 주기적으로 강화한다(gravity_boost). 없으면 기본 세기.
pub(super) fn singularity_gravity_pulse(
    time: Res<Time>,
    mut bh: ResMut<BlackHoleActive>,
    mut shake: MessageWriter<ShakeEvent>,
    mut surging: Local<bool>,
    bosses: Query<&Boss>,
) {
    let has_singularity = bosses.iter().any(|b| b.kind == BossKind::SingularityCore);
    if !has_singularity {
        bh.strength = GRAVITY_STRENGTH;
        *surging = false;
        return;
    }
    let boost = gravity_boost(time.elapsed_secs());
    bh.strength = GRAVITY_STRENGTH * boost;
    // 흡인 강화가 피크로 치솟는 순간(상승 엣지) 화면을 흔들어 '중력 파동'을 체감시킨다(임팩트).
    let near_peak = boost > 1.0 + (GRAVITY_INTENSIFY - 1.0) * 0.6;
    if near_peak && !*surging {
        shake.write(ShakeEvent(SHAKE_EXPLOSION * 2.0));
        *surging = true;
    } else if !near_peak {
        *surging = false;
    }
}

/// 특이점 코어가 가만히 있지 않도록: 대부분 홈에서 부유하다 주기적으로 플레이어를 향해
/// 블랙홀 밖으로 돌진했다 복귀한다(덮치는 위협). 사이엔 다시 중앙에 앉아 흡인을 지속.
pub(super) fn singularity_lunge(
    time: Res<Time>,
    players: Query<&Transform, (With<Player>, Without<Boss>)>,
    mut q: Query<(&mut Transform, &mut Lunge), With<Boss>>,
) {
    let t = time.elapsed_secs();
    let player_pos = players.single().ok().map(|p| p.translation.truncate());
    let home = Vec2::new(0.0, crate::core::config::BLACK_HOLE_POS_Y);
    for (mut tf, mut lunge) in &mut q {
        lunge.timer.tick(time.delta());
        let dur = lunge.timer.duration().as_secs_f32();
        let f = if dur > 0.0 { lunge.timer.elapsed_secs() / dur } else { 0.0 };
        let pos = if f < 0.65 {
            // 홈에서 약한 부유
            lunge.charging = false;
            home + Vec2::new((t * 0.7).sin() * 16.0, (t * 1.1).sin() * 10.0)
        } else {
            // 돌진 창(0.65..1.0): 진입 시 플레이어 방향 목표 캡처, sin으로 나갔다 복귀
            if !lunge.charging {
                let dir = player_pos
                    .map(|pp| (pp - home).normalize_or_zero())
                    .unwrap_or(Vec2::NEG_Y);
                // 목적지를 화면 안(보스 반경 여유 70)으로 클램프해 상단 등으로 돌출하지 않게 한다.
                let hw = crate::core::config::HALF_WIDTH - 70.0;
                let hh = crate::core::config::HALF_HEIGHT - 70.0;
                let dest = (home + dir * crate::core::config::LUNGE_DIST)
                    .clamp(Vec2::new(-hw, -hh), Vec2::new(hw, hh));
                lunge.target = dest - home;
                lunge.charging = true;
            }
            let p = (f - 0.65) / 0.35;
            home + lunge.target * (p * std::f32::consts::PI).sin()
        };
        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
    }
}
