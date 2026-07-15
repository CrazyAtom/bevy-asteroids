//! 순수 게임 로직(Bevy App 무관, 단위 테스트 집중 대상) — 도메인별 서브모듈로 분리하되
//! 전부 재수출해 기존 `crate::core::logic::함수` 경로를 그대로 유지한다.
//! geometry(기하 판정) · difficulty(난이도 공식) · env(테마 환경 커브) + 아래 공통 유틸.

mod difficulty;
mod env;
mod geometry;

pub use difficulty::*;
pub use env::*;
pub use geometry::*;

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

/// 속도를 0쪽으로 감쇠(브레이크). t는 [0,1]로 클램프해 반대로 튀지 않게 한다.
pub fn apply_brake(velocity: Vec2, rate: f32, dt: f32) -> Vec2 {
    let t = (rate * dt).clamp(0.0, 1.0);
    velocity.lerp(Vec2::ZERO, t)
}

/// from에서 to를 향하는 단위 벡터. 같은 지점이면 기본값 Vec2::Y.
pub fn aim_direction(from: Vec2, to: Vec2) -> Vec2 {
    (to - from).try_normalize().unwrap_or(Vec2::Y)
}

pub fn update_high_score(current: u32, new: u32) -> u32 {
    current.max(new)
}

/// trauma를 dt만큼 감쇠(0 미만으로 내려가지 않음).
pub fn decay_trauma(trauma: f32, dt: f32) -> f32 {
    (trauma - crate::core::config::SHAKE_DECAY * dt).max(0.0)
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
}
