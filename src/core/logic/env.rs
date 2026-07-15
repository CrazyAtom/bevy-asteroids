//! 테마 환경 효과 커브: 전자기폭풍(시야 펄스), 블랙홀(중력·흡인 강화).

use bevy::math::Vec2;

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

/// 블랙홀을 향한 거리² 반비례 가속. 중심 근처는 min_dist로 클램프해 발산을 막고,
/// body와 hole이 같은 지점이면 0을 반환한다.
pub fn gravity_accel(body: Vec2, hole: Vec2, strength: f32, min_dist: f32) -> Vec2 {
    let to = hole - body;
    let dir = to.normalize_or_zero();
    if dir == Vec2::ZERO {
        return Vec2::ZERO;
    }
    let d2 = to.length_squared().max(min_dist * min_dist);
    dir * (strength / d2)
}

/// 흡인 강화 배율: 평소 1.0, 주기(GRAVITY_PULSE_PERIOD)마다 GRAVITY_INTENSIFY까지 치솟았다 회복. [1.0, INTENSIFY].
pub fn gravity_boost(t: f32) -> f32 {
    use std::f32::consts::TAU;
    let pulse = (TAU * t / crate::core::config::GRAVITY_PULSE_PERIOD).sin().max(0.0).powi(4);
    1.0 + (crate::core::config::GRAVITY_INTENSIFY - 1.0) * pulse
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn storm_pulse_exact_peak_and_troughs() {
        // STORM_PERIOD=7 → t=1.75는 위상 1/4(sin=1) → 피크 1.0
        assert!((storm_pulse(1.75) - 1.0).abs() < 1e-4);
        // t=0은 sin=0 → 0.0
        assert!(storm_pulse(0.0).abs() < 1e-6);
        // t=5.25는 위상 3/4(sin=-1) → max(0)로 0.0.
        // (max(0.0)을 abs()로 바꾸면 1.0이 되어 이 단정이 실패 → 변형 검출)
        assert!(storm_pulse(5.25).abs() < 1e-6);
    }

    #[test]
    fn storm_vision_scale_hits_min_at_peak_and_one_at_trough() {
        // 피크(t=1.75) → FOG_VISION_MIN, 평소(t=0) → 1.0
        assert!((storm_vision_scale(1.75) - crate::core::config::FOG_VISION_MIN).abs() < 1e-4);
        assert!((storm_vision_scale(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gravity_pulls_toward_hole_and_weakens_with_distance() {
        let hole = Vec2::new(0.0, 100.0);
        let near = gravity_accel(Vec2::new(0.0, 0.0), hole, 1_000_000.0, 40.0);
        let far = gravity_accel(Vec2::new(0.0, -200.0), hole, 1_000_000.0, 40.0);
        // 방향: 둘 다 +y(중심 쪽)
        assert!(near.y > 0.0 && far.y > 0.0);
        assert!(near.x.abs() < 1e-3 && far.x.abs() < 1e-3);
        // 가까울수록 강함
        assert!(near.length() > far.length());
    }

    #[test]
    fn gravity_is_clamped_near_center() {
        let hole = Vec2::ZERO;
        // 중심과 거의 같은 지점이어도 min_dist로 클램프되어 유한
        let a = gravity_accel(Vec2::new(0.1, 0.0), hole, 1_000_000.0, 40.0);
        let max = 1_000_000.0 / (40.0 * 40.0);
        assert!(a.length() <= max + 1e-3);
    }

    #[test]
    fn gravity_zero_at_same_point() {
        let a = gravity_accel(Vec2::ZERO, Vec2::ZERO, 1_000_000.0, 40.0);
        assert_eq!(a, Vec2::ZERO);
    }

    #[test]
    fn gravity_boost_between_one_and_intensify_with_peak() {
        use crate::core::config::GRAVITY_INTENSIFY;
        let mut saw_peak = false;
        for i in 0..300 {
            let t = i as f32 * 0.05;
            let b = gravity_boost(t);
            assert!((1.0 - 1e-6..=GRAVITY_INTENSIFY + 1e-6).contains(&b));
            if b > (1.0 + GRAVITY_INTENSIFY) / 2.0 {
                saw_peak = true;
            }
        }
        assert!(saw_peak, "주기 안에 흡인 강화 피크가 존재해야 한다");
    }

    #[test]
    fn gravity_boost_exact_peak_and_troughs() {
        use crate::core::config::GRAVITY_INTENSIFY;
        // PERIOD=6 → t=1.5는 위상 1/4(sin=1) → 피크 = INTENSIFY
        assert!((gravity_boost(1.5) - GRAVITY_INTENSIFY).abs() < 1e-4);
        // t=0은 sin=0 → 1.0
        assert!((gravity_boost(0.0) - 1.0).abs() < 1e-6);
        // t=4.5는 위상 3/4(sin=-1) → max(0)로 1.0.
        // (max(0.0)을 abs()로 바꾸면 INTENSIFY가 되어 이 단정이 실패 → 변형 검출)
        assert!((gravity_boost(4.5) - 1.0).abs() < 1e-6);
    }
}
