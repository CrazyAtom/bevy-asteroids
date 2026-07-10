pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const HALF_WIDTH: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_HEIGHT: f32 = WINDOW_HEIGHT / 2.0;

pub const SHIP_ROTATION_SPEED: f32 = 4.0; // rad/s
pub const SHIP_THRUST: f32 = 320.0;       // units/s^2
pub const SHIP_COLLIDER_RADIUS: f32 = 12.0;

pub const BULLET_SPEED: f32 = 620.0;
pub const BULLET_LIFETIME_SECS: f32 = 1.2;
pub const BULLET_COLLIDER_RADIUS: f32 = 2.0;

pub const STARTING_LIVES: u32 = 3;

pub const INITIAL_ASTEROIDS: usize = 4;
pub const ASTEROID_MIN_SPEED: f32 = 40.0;
pub const ASTEROID_MAX_SPEED: f32 = 120.0;
