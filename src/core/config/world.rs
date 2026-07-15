//! 월드: 화면 크기, 소행성, 적 UFO(+적탄), 파워업.

pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const HALF_WIDTH: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_HEIGHT: f32 = WINDOW_HEIGHT / 2.0;

// 소행성
pub const BASE_ASTEROIDS: usize = 3;
pub const MAX_ASTEROIDS: usize = 10;
pub const ASTEROID_MIN_SPEED: f32 = 40.0;
pub const ASTEROID_MAX_SPEED: f32 = 120.0;
pub const ASTEROID_SPIN_MAX: f32 = 1.5; // rad/s (회전 각속도 범위 ±)

// 적 UFO
pub const UFO_SPEED: f32 = 140.0;
pub const UFO_LARGE_RADIUS: f32 = 20.0;
pub const UFO_SMALL_RADIUS: f32 = 12.0;
pub const UFO_FIRE_INTERVAL_SECS: f32 = 1.4;
pub const UFO_SPAWN_INTERVAL_BASE: f32 = 12.0;
pub const UFO_SPAWN_INTERVAL_MIN: f32 = 5.0;

// 적 총알
pub const ENEMY_BULLET_COLLIDER_RADIUS: f32 = 2.5;
pub const UFO_BULLET_LIFETIME_SECS: f32 = 2.5;
pub const UFO_BULLET_SPEED: f32 = 320.0;

// 파워업
pub const POWERUP_DROP_CHANCE: f32 = 0.18;
pub const POWERUP_LIFETIME_SECS: f32 = 8.0;
pub const POWERUP_DRIFT_SPEED: f32 = 30.0;
pub const POWERUP_RADIUS: f32 = 12.0;
pub const POWERUP_MAGNET_RANGE: f32 = 100.0; // 이 거리 안이면 파워업이 플레이어로 끌려옴
pub const POWERUP_MAGNET_SPEED: f32 = 320.0; // 끌려오는 속도(u/s)
pub const SHIELD_SECS: f32 = 7.0;
