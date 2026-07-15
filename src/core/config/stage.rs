//! 스테이지 진행 + 테마 트위스트(얼음·전자기폭풍·블랙홀 환경 효과).

pub const CYCLE_LEN: usize = 3;     // 한 사이클의 스테이지 수(= 무작위로 뽑을 테마 수)
pub const WAVES_PER_STAGE: u32 = 3; // 스테이지당 보스 전 웨이브 수

// 테마 트위스트 — 얼음(Frozen Field)
// 얼음 테마 감쇠(기본보다 마찰↓ → 더 미끄러움).
pub const SHIP_DAMPING_ICE: f32 = 0.997;

// 테마 트위스트 — 전자기폭풍(EM Storm, 안개 시야 제한)
pub const STORM_PERIOD: f32 = 7.0;     // 폭풍 피크 주기(초)
pub const FOG_VISION_MIN: f32 = 0.6;   // 피크 시 시야 배율(1.0=평소)
pub const FOG_BASE_SIZE: f32 = 5200.0; // 안개 오버레이 기본 크기(px). 최소 배율에서도 화면 구석까지 덮음

// 테마 트위스트 — 블랙홀(중력장)
pub const GRAVITY_STRENGTH: f32 = 1_800_000.0; // 흡인력 계수(accel = strength / dist²)
pub const GRAVITY_MIN_DIST: f32 = 40.0;        // 중심 근처 클램프(발산 방지)
pub const EVENT_HORIZON: f32 = 32.0;           // 사건의 지평선(치명) 반경
pub const BLACK_HOLE_POS_Y: f32 = 120.0;       // 블랙홀 위치 y(우주선 스폰(0,0)과 겹치지 않게)
pub const BLACK_HOLE_VISUAL: f32 = 190.0;      // 블랙홀 스프라이트 크기(지평선보다 크게 = 위압·경고)
pub const BLACK_HOLE_SPIN: f32 = 0.5;          // 강착원반 회전 각속도(rad/s, 화려함)
