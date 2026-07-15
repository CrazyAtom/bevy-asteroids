//! 렌더/연출: 스프라이트 배율, Z 레이어, 폭발 애니/파티클, 화면 흔들림.

pub const VISUAL_FIT: f32 = 1.2; // 콜라이더 반경 대비 스프라이트 표시 배율

// Z 레이어(배경 < 화염 < 물체 < 실드 < 빔 < 파티클 < 폭발 < 안개)
pub const Z_BACKGROUND: f32 = -100.0;
pub const Z_FLAME: f32 = -1.0;
pub const Z_ENTITY: f32 = 0.0;
pub const Z_SHIELD: f32 = 5.0;
pub const Z_BEAM: f32 = 10.0;
pub const Z_PARTICLE: f32 = 12.0;
pub const Z_EXPLOSION: f32 = 15.0;
pub const Z_FOG: f32 = 20.0; // 안개 오버레이 Z(모든 월드 스프라이트 위)

// 폭발 프레임 애니메이션
pub const EXPLOSION_FRAME_COUNT: usize = 6;
pub const EXPLOSION_FRAME_SECS: f32 = 0.06;

// 파티클
pub const EXPLOSION_PARTICLES: usize = 10;
pub const PARTICLE_LIFETIME_SECS: f32 = 0.6;
pub const PARTICLE_SPEED_MIN: f32 = 60.0;
pub const PARTICLE_SPEED_MAX: f32 = 200.0;

// 화면 흔들림
pub const MAX_SHAKE_OFFSET: f32 = 18.0;
pub const SHAKE_DECAY: f32 = 1.5;
pub const SHAKE_HIT: f32 = 0.6;
pub const SHAKE_EXPLOSION: f32 = 0.25;
pub const SHAKE_SPECIAL: f32 = 0.5;
