use bevy::math::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    pub fn radius(self) -> f32 {
        match self {
            AsteroidSize::Large => 45.0,
            AsteroidSize::Medium => 25.0,
            AsteroidSize::Small => 14.0,
        }
    }

    pub fn score(self) -> u32 {
        match self {
            AsteroidSize::Large => 20,
            AsteroidSize::Medium => 50,
            AsteroidSize::Small => 100,
        }
    }

    pub fn speed_scale(self) -> f32 {
        match self {
            AsteroidSize::Large => 1.0,
            AsteroidSize::Medium => 1.5,
            AsteroidSize::Small => 2.2,
        }
    }
}

pub fn next_asteroid_size(size: AsteroidSize) -> Option<AsteroidSize> {
    match size {
        AsteroidSize::Large => Some(AsteroidSize::Medium),
        AsteroidSize::Medium => Some(AsteroidSize::Small),
        AsteroidSize::Small => None,
    }
}

/// 화면 밖으로 나간 좌표를 반대편으로 순환시킨다. half는 반너비/반높이.
pub fn wrap_position(pos: Vec2, half: Vec2) -> Vec2 {
    let mut p = pos;
    let full = half * 2.0;
    if p.x > half.x {
        p.x -= full.x;
    } else if p.x < -half.x {
        p.x += full.x;
    }
    if p.y > half.y {
        p.y -= full.y;
    } else if p.y < -half.y {
        p.y += full.y;
    }
    p
}

/// 두 원이 겹치는지 판정 (제곱 거리 비교로 sqrt 회피).
pub fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    let r = ra + rb;
    a.distance_squared(b) <= r * r
}

/// 속도를 0쪽으로 감쇠(브레이크). t는 [0,1]로 클램프해 반대로 튀지 않게 한다.
pub fn apply_brake(velocity: Vec2, rate: f32, dt: f32) -> Vec2 {
    let t = (rate * dt).clamp(0.0, 1.0);
    velocity.lerp(Vec2::ZERO, t)
}

/// from에서 to를 향하는 단위 벡터. 같은 지점이면 기본값 Vec2::Y.
pub fn aim_direction(from: Vec2, to: Vec2) -> Vec2 {
    (to - from).try_normalize().unwrap_or(Vec2::Y)
}

/// 난이도 계산용 0-기반 지수. 게임은 웨이브1부터 시작하므로 웨이브1이
/// 기준선(지수 0)이 되도록 1을 뺀다. → 첫 웨이브 = 클래식 기본 난이도(소행성 4개).
fn difficulty_index(wave: u32) -> u32 {
    wave.saturating_sub(1)
}

pub fn asteroid_count_for_wave(wave: u32) -> usize {
    (crate::core::config::BASE_ASTEROIDS + difficulty_index(wave) as usize)
        .min(crate::core::config::MAX_ASTEROIDS)
}

pub fn asteroid_speed_scale_for_wave(wave: u32) -> f32 {
    (1.0 + difficulty_index(wave) as f32 * 0.065).min(2.0)
}

pub fn ufo_interval_for_wave(wave: u32) -> f32 {
    (crate::core::config::UFO_SPAWN_INTERVAL_BASE - difficulty_index(wave) as f32 * 0.8)
        .max(crate::core::config::UFO_SPAWN_INTERVAL_MIN)
}

pub fn small_ufo_probability_for_wave(wave: u32) -> f32 {
    (0.2 + difficulty_index(wave) as f32 * 0.05).min(0.9)
}

/// 웨이브1은 UFO 없이 소행성만 등장(초반 학습 구간). 웨이브2부터 UFO 활성화.
pub fn ufo_active_for_wave(wave: u32) -> bool {
    wave >= 2
}

pub fn update_high_score(current: u32, new: u32) -> u32 {
    current.max(new)
}

/// trauma를 dt만큼 감쇠(0 미만으로 내려가지 않음).
pub fn decay_trauma(trauma: f32, dt: f32) -> f32 {
    (trauma - crate::core::config::SHAKE_DECAY * dt).max(0.0)
}

/// origin에서 dir 방향으로 length만큼 뻗는 반폭 half_width 빔이 중심 center·반지름 radius 원과 겹치는지.
pub fn segment_circle_hit(origin: Vec2, dir: Vec2, length: f32, half_width: f32, center: Vec2, radius: f32) -> bool {
    let d = dir.normalize_or_zero();
    let t = (center - origin).dot(d).clamp(0.0, length);
    let closest = origin + d * t;
    closest.distance(center) <= half_width + radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_size_shrinks_then_none() {
        assert_eq!(next_asteroid_size(AsteroidSize::Large), Some(AsteroidSize::Medium));
        assert_eq!(next_asteroid_size(AsteroidSize::Medium), Some(AsteroidSize::Small));
        assert_eq!(next_asteroid_size(AsteroidSize::Small), None);
    }

    #[test]
    fn wrap_moves_past_right_edge_to_left() {
        let half = Vec2::new(640.0, 360.0);
        let wrapped = wrap_position(Vec2::new(700.0, 0.0), half);
        assert!(wrapped.x < 0.0);
        assert!((wrapped.x - (-580.0)).abs() < 1e-4);
    }

    #[test]
    fn wrap_moves_past_bottom_edge_to_top() {
        let half = Vec2::new(640.0, 360.0);
        let wrapped = wrap_position(Vec2::new(0.0, -400.0), half);
        assert!(wrapped.y > 0.0);
    }

    #[test]
    fn wrap_leaves_inside_point_unchanged() {
        let half = Vec2::new(640.0, 360.0);
        let p = Vec2::new(10.0, -20.0);
        assert_eq!(wrap_position(p, half), p);
    }

    #[test]
    fn circles_overlap_true_when_close() {
        assert!(circles_overlap(Vec2::ZERO, 5.0, Vec2::new(8.0, 0.0), 5.0));
    }

    #[test]
    fn circles_overlap_false_when_far() {
        assert!(!circles_overlap(Vec2::ZERO, 5.0, Vec2::new(20.0, 0.0), 5.0));
    }

    #[test]
    fn smaller_asteroids_are_faster() {
        assert!(AsteroidSize::Small.speed_scale() > AsteroidSize::Medium.speed_scale());
        assert!(AsteroidSize::Medium.speed_scale() > AsteroidSize::Large.speed_scale());
        assert_eq!(AsteroidSize::Large.speed_scale(), 1.0);
    }

    #[test]
    fn brake_reduces_speed_toward_zero() {
        let v = Vec2::new(100.0, 0.0);
        let braked = apply_brake(v, 3.0, 0.1);
        assert!(braked.length() < v.length());
        assert!(braked.x > 0.0); // 방향 유지, 아직 0 아님
    }

    #[test]
    fn brake_never_overshoots_past_zero() {
        let v = Vec2::new(10.0, 0.0);
        let braked = apply_brake(v, 3.0, 100.0); // 큰 dt
        assert!(braked.length() <= v.length());
        assert!(braked.x >= 0.0); // 반대로 튀지 않음
    }

    #[test]
    fn aim_direction_points_toward_target() {
        let d = aim_direction(Vec2::ZERO, Vec2::new(0.0, 10.0));
        assert!((d - Vec2::new(0.0, 1.0)).length() < 1e-4);
    }

    #[test]
    fn aim_direction_zero_defaults_up() {
        let d = aim_direction(Vec2::ZERO, Vec2::ZERO);
        assert!((d - Vec2::Y).length() < 1e-4);
    }

    #[test]
    fn high_score_keeps_maximum() {
        assert_eq!(update_high_score(100, 250), 250);
        assert_eq!(update_high_score(300, 250), 300);
        assert_eq!(update_high_score(0, 0), 0);
    }

    #[test]
    fn trauma_decays_to_zero_not_below() {
        let t = decay_trauma(0.5, 0.1);
        assert!((0.0..0.5).contains(&t));
        assert_eq!(decay_trauma(0.05, 100.0), 0.0); // 큰 dt여도 음수 아님
    }

    #[test]
    fn wave_scaling_formulas() {
        // 웨이브1 = 기준선(지수 0): 기본4개, 상한10
        assert_eq!(asteroid_count_for_wave(1), 3);
        assert_eq!(asteroid_count_for_wave(4), 6); // 웨이브4 → 지수3 → 3+3
        assert_eq!(asteroid_count_for_wave(50), 10); // 상한
        // 속도 배수: 웨이브↑ → 증가, 웨이브1은 정확히 1.0
        assert!(asteroid_speed_scale_for_wave(6) > asteroid_speed_scale_for_wave(1));
        assert_eq!(asteroid_speed_scale_for_wave(1), 1.0);
        // UFO 간격: 웨이브↑ → 감소, 하한 존재
        assert!(ufo_interval_for_wave(6) < ufo_interval_for_wave(1));
        assert!(ufo_interval_for_wave(100) >= 5.0);
        // 소형 확률: 웨이브↑ → 증가, [0,1]
        assert!(small_ufo_probability_for_wave(11) > small_ufo_probability_for_wave(1));
        assert!(small_ufo_probability_for_wave(100) <= 1.0);

        // 캡 경계값 고정: 큰 웨이브에서 상한/하한이 정확히 걸리는지
        assert_eq!(asteroid_count_for_wave(1000), 10);            // 개수 상한
        assert_eq!(asteroid_speed_scale_for_wave(1000), 2.0);     // 속도 배수 상한
        assert_eq!(ufo_interval_for_wave(1000), 5.0);             // UFO 간격 하한
        assert_eq!(small_ufo_probability_for_wave(1000), 0.9);    // 소형 확률 상한

        // 웨이브1은 UFO 미등장, 웨이브2부터 활성화
        assert!(!ufo_active_for_wave(1));
        assert!(ufo_active_for_wave(2));
    }

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
}
