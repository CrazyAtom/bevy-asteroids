//! 기하 판정: 화면 순환/벽 반사, 원-원·선분-원 충돌.

use bevy::math::Vec2;

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

/// 경계를 넘은 좌표를 경계로 클램프하고 해당 축 속도를 안쪽으로 반전한다(벽 반사).
/// 두 축을 독립 처리하며, 경계 안의 좌표는 위치·속도 모두 불변.
pub fn reflect_edge(pos: Vec2, vel: Vec2, half: Vec2) -> (Vec2, Vec2) {
    let mut p = pos;
    let mut v = vel;
    if p.x > half.x {
        p.x = half.x;
        v.x = -v.x.abs();
    } else if p.x < -half.x {
        p.x = -half.x;
        v.x = v.x.abs();
    }
    if p.y > half.y {
        p.y = half.y;
        v.y = -v.y.abs();
    } else if p.y < -half.y {
        p.y = -half.y;
        v.y = v.y.abs();
    }
    (p, v)
}

/// 두 원이 겹치는지 판정 (제곱 거리 비교로 sqrt 회피).
pub fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    let r = ra + rb;
    a.distance_squared(b) <= r * r
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

    #[test]
    fn reflect_bounces_off_right_edge_and_flips_x() {
        let half = Vec2::new(640.0, 360.0);
        let (p, v) = reflect_edge(Vec2::new(700.0, 0.0), Vec2::new(50.0, 10.0), half);
        assert!((p.x - 640.0).abs() < 1e-4); // 경계로 클램프
        assert!(v.x < 0.0); // x 속도 반전(안쪽으로)
        assert!((v.y - 10.0).abs() < 1e-4); // y는 불변
    }

    #[test]
    fn reflect_bounces_off_bottom_edge_and_flips_y() {
        let half = Vec2::new(640.0, 360.0);
        let (p, v) = reflect_edge(Vec2::new(0.0, -400.0), Vec2::new(0.0, -20.0), half);
        assert!((p.y + 360.0).abs() < 1e-4);
        assert!(v.y > 0.0);
    }

    #[test]
    fn reflect_leaves_inside_point_unchanged() {
        let half = Vec2::new(640.0, 360.0);
        let (p, v) = reflect_edge(Vec2::new(10.0, -20.0), Vec2::new(3.0, 4.0), half);
        assert_eq!(p, Vec2::new(10.0, -20.0));
        assert_eq!(v, Vec2::new(3.0, 4.0));
    }
}
