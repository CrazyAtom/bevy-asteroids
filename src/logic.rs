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
}
