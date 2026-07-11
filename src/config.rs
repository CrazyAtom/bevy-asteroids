use bevy::prelude::Color;

pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const HALF_WIDTH: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_HEIGHT: f32 = WINDOW_HEIGHT / 2.0;

pub const SHIP_ROTATION_SPEED: f32 = 4.0; // rad/s
pub const SHIP_THRUST: f32 = 320.0;       // units/s^2
pub const SHIP_COLLIDER_RADIUS: f32 = 12.0;
// 우주선 감속(마찰): FixedUpdate 틱당 속도에 곱하는 계수. 1.0=마찰없음, 낮을수록 빨리 멈춤.
pub const SHIP_DAMPING: f32 = 0.985;
// 우주선 최고 속도 상한(units/s).
pub const SHIP_MAX_SPEED: f32 = 450.0;
pub const SHIP_BRAKE_RATE: f32 = 3.0; // 브레이크 감쇠율(1/s)
pub const SHIP_COLOR: Color = Color::srgb(0.40, 0.90, 1.00); // 청록
pub const FLAME_COLOR: Color = Color::srgb(1.00, 0.55, 0.15); // 주황

pub const BULLET_SPEED: f32 = 620.0;
pub const BULLET_LIFETIME_SECS: f32 = 1.2;
pub const BULLET_COLLIDER_RADIUS: f32 = 2.0;

pub const ENEMY_BULLET_COLLIDER_RADIUS: f32 = 2.5;
pub const UFO_BULLET_LIFETIME_SECS: f32 = 2.5;
pub const UFO_BULLET_SPEED: f32 = 320.0;

pub const STARTING_LIVES: u32 = 3;

pub const INITIAL_ASTEROIDS: usize = 4;
pub const ASTEROID_MIN_SPEED: f32 = 40.0;
pub const ASTEROID_MAX_SPEED: f32 = 120.0;
pub const ASTEROID_SPIN_MAX: f32 = 1.5; // rad/s (회전 각속도 범위 ±)

pub const EXPLOSION_PARTICLES: usize = 10;
pub const PARTICLE_LIFETIME_SECS: f32 = 0.6;
pub const PARTICLE_SPEED_MIN: f32 = 60.0;
pub const PARTICLE_SPEED_MAX: f32 = 200.0;
