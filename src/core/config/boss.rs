//! 보스: 공통(체력·보너스·추적·등장) + 종류별(모암/모함/코어는 인라인 상수 없음,
//! 골렘·테슬라·특이점) 튜닝.

// 공통
pub const BULLET_BOSS_DAMAGE: f32 = 1.0;
pub const BEAM_BOSS_DAMAGE: f32 = 4.0;
pub const BOSS_BASE_HEALTH: f32 = 52.0;      // 기준 체력(전반 상향)
pub const BOSS_HEALTH_PER_CYCLE: f32 = 18.0; // 사이클마다 증가(후반 스케일↑)
pub const BOSS_SCORE_BONUS: u32 = 2000;      // 격파 보너스
pub const BOSS_TRACK: f32 = 0.35;          // 비-골렘 보스가 플레이어 x를 추적하는 블렌드(0=고정, 1=완전추적)
pub const BOSS_ENTRANCE_SHAKE: f32 = 0.8;  // 보스 등장 시 화면 흔들림 강도

// 얼음 골렘
pub const ICE_GOLEM_HEALTH_MUL: f32 = 1.4; // 느린 대신 높은 체력
pub const GOLEM_DRIFT_SPEED: f32 = 55.0;   // 느린 드리프트 속도(u/s)
pub const GOLEM_ATTACK_INTERVAL: f32 = 2.5;
pub const GOLEM_STEER: f32 = 1.5;          // 골렘이 플레이어 쪽으로 조향하는 초당 비율

// 테슬라 코어
pub const TESLA_HEALTH_MUL: f32 = 1.1;
pub const TESLA_ATTACK_INTERVAL: f32 = 1.8;
pub const BLINK_INTERVAL: f32 = 3.0; // 순간이동 주기(초)
pub const STORM_EMP_THRESHOLD: f32 = 0.6; // storm_pulse가 이 값을 상향 돌파하면 EMP 발동
pub const BLINK_TELEGRAPH_SECS: f32 = 0.4; // 블링크 직전 예고(축소) 시간

// 특이점 코어
pub const SINGULARITY_HEALTH_MUL: f32 = 1.3;
pub const SINGULARITY_ATTACK_INTERVAL: f32 = 0.42; // 나선 탄 발사 주기(짧게)
pub const SPIRAL_STEP: f32 = 0.4;                 // 발사마다 회전량(rad)
pub const SPIRAL_ARMS: u32 = 7;                   // 발사당 탄 수(밀도↑ = 임팩트)
pub const GRAVITY_INTENSIFY: f32 = 2.2;    // 흡인 강화 피크 배율
pub const GRAVITY_PULSE_PERIOD: f32 = 6.0; // 강화 주기(초)
pub const LUNGE_INTERVAL: f32 = 4.5;       // 특이점 코어 돌진 주기(초)
pub const LUNGE_DIST: f32 = 260.0;         // 돌진 거리(중앙 홈 기준, u)
