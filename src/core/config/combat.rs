//! 아군 무기: 총알, 발사 쿨다운/연사/확산, 특수무기(빔).

pub const BULLET_SPEED: f32 = 620.0;
pub const BULLET_LIFETIME_SECS: f32 = 1.2;
pub const BULLET_COLLIDER_RADIUS: f32 = 2.0;

// 발사 쿨다운
pub const FIRE_INTERVAL: f32 = 0.25;
pub const RAPID_FIRE_INTERVAL: f32 = 0.10;
pub const SPREAD_ANGLE: f32 = 0.26; // rad
pub const RAPID_FIRE_SECS: f32 = 6.0;
pub const SPREAD_SECS: f32 = 6.0;
pub const SPREAD_MAX_LEVEL: u8 = 3; // 확산탄 최대 레벨(발사 수 = level*3 → 최대 9발)

// 특수무기
pub const STARTING_SPECIAL_CHARGES: u32 = 1; // 게임 시작 시 보유한 특수무기 충전 수
pub const BEAM_LIFETIME_SECS: f32 = 0.4;
pub const BEAM_WIDTH: f32 = 22.0;
pub const BEAM_LENGTH: f32 = 2000.0;

// 특수무기 — 카트라이더 큐
pub const HUD_QUEUE_SLOTS: usize = 5; // HUD에 표시할 큐 슬롯 수(내부 큐는 무제한)

// 특수무기 — 산탄 노바 / 유도 미사일
pub const NOVA_BULLETS: usize = 16; // 전방위 산탄 수
pub const MISSILE_COUNT: usize = 5; // 발사 수
pub const MISSILE_SPEED: f32 = 420.0;
pub const MISSILE_TURN_RATE: f32 = 6.0; // 조향 상한(rad/s)
pub const MISSILE_LIFETIME_SECS: f32 = 2.5;
pub const MISSILE_COLLIDER_RADIUS: f32 = 4.0;
pub const MISSILE_SPREAD: f32 = 0.5; // 초기 부채꼴 반각(rad)

// 특수무기 — 충격파 / 공격형 실드
pub const SHOCKWAVE_RADIUS: f32 = 220.0;
pub const SHOCKWAVE_IMPULSE: f32 = 380.0; // 밀쳐낼 속도(u/s)
pub const SHOCKWAVE_RING_SECS: f32 = 0.35; // 링 연출 시간
pub const SHIELD_BURST_SECS: f32 = 4.0;
pub const BURST_FLASH_HZ: f32 = 12.0; // 우주선 번쩍임 토글 빈도(대략 Hz 감각)
pub const BURST_PULSE: f32 = 0.12; // 크기 펄스 진폭(±12%)
