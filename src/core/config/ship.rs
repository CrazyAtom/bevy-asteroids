//! 우주선: 조종(회전·추진·감속), 목숨, 하이퍼스페이스.

pub const SHIP_ROTATION_SPEED: f32 = 4.0; // rad/s (회전 각속도 상한)
pub const SHIP_TURN_MIN: f32 = 1.2;       // 회전 시작 각속도(미세 조준용, rad/s)
pub const SHIP_TURN_ACCEL: f32 = 8.0;     // 누르는 동안 각속도 가속(rad/s^2)
pub const SHIP_THRUST: f32 = 320.0;       // units/s^2
pub const SHIP_COLLIDER_RADIUS: f32 = 12.0;
// 우주선 감속(마찰): FixedUpdate 틱당 속도에 곱하는 계수. 1.0=마찰없음, 낮을수록 빨리 멈춤.
pub const SHIP_DAMPING: f32 = 0.985;
// 우주선 최고 속도 상한(units/s).
pub const SHIP_MAX_SPEED: f32 = 450.0;
pub const SHIP_BRAKE_RATE: f32 = 3.0; // 브레이크 감쇠율(1/s)

pub const STARTING_LIVES: u32 = 3;
pub const SPAWN_INVINCIBILITY_SECS: f32 = 3.0; // (재)스폰 직후 일시 무적(즉사 연쇄 방지)

pub const HYPERSPACE_COOLDOWN_SECS: f32 = 2.0;
