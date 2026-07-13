pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const HALF_WIDTH: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_HEIGHT: f32 = WINDOW_HEIGHT / 2.0;

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

pub const BULLET_SPEED: f32 = 620.0;
pub const BULLET_LIFETIME_SECS: f32 = 1.2;
pub const BULLET_COLLIDER_RADIUS: f32 = 2.0;

pub const ENEMY_BULLET_COLLIDER_RADIUS: f32 = 2.5;
pub const UFO_BULLET_LIFETIME_SECS: f32 = 2.5;
pub const UFO_BULLET_SPEED: f32 = 320.0;

pub const STARTING_LIVES: u32 = 3;
pub const SPAWN_INVINCIBILITY_SECS: f32 = 3.0; // (재)스폰 직후 일시 무적(즉사 연쇄 방지)

pub const BASE_ASTEROIDS: usize = 4;
pub const MAX_ASTEROIDS: usize = 10;
pub const ASTEROID_MIN_SPEED: f32 = 40.0;
pub const ASTEROID_MAX_SPEED: f32 = 120.0;
pub const ASTEROID_SPIN_MAX: f32 = 1.5; // rad/s (회전 각속도 범위 ±)

pub const EXPLOSION_PARTICLES: usize = 10;
pub const PARTICLE_LIFETIME_SECS: f32 = 0.6;
pub const PARTICLE_SPEED_MIN: f32 = 60.0;
pub const PARTICLE_SPEED_MAX: f32 = 200.0;

pub const UFO_SPEED: f32 = 140.0;
pub const UFO_LARGE_RADIUS: f32 = 20.0;
pub const UFO_SMALL_RADIUS: f32 = 12.0;
pub const UFO_FIRE_INTERVAL_SECS: f32 = 1.4;
pub const UFO_SPAWN_INTERVAL_BASE: f32 = 12.0;
pub const UFO_SPAWN_INTERVAL_MIN: f32 = 5.0;

// Phase 3 — 화면 흔들림
pub const MAX_SHAKE_OFFSET: f32 = 18.0;
pub const SHAKE_DECAY: f32 = 1.5;
pub const SHAKE_HIT: f32 = 0.6;
pub const SHAKE_EXPLOSION: f32 = 0.25;
pub const SHAKE_SPECIAL: f32 = 0.5;

// Phase 3 — 발사 쿨다운
pub const FIRE_INTERVAL: f32 = 0.25;
pub const RAPID_FIRE_INTERVAL: f32 = 0.10;
pub const SPREAD_ANGLE: f32 = 0.26; // rad
pub const RAPID_FIRE_SECS: f32 = 6.0;
pub const SPREAD_SECS: f32 = 6.0;
pub const SPREAD_MAX_LEVEL: u8 = 3; // 확산탄 최대 레벨(발사 수 = level*3 → 최대 9발)

// Phase 3 — 파워업
pub const POWERUP_DROP_CHANCE: f32 = 0.18;
pub const POWERUP_LIFETIME_SECS: f32 = 8.0;
pub const POWERUP_DRIFT_SPEED: f32 = 30.0;
pub const POWERUP_RADIUS: f32 = 12.0;
pub const POWERUP_MAGNET_RANGE: f32 = 100.0; // 이 거리 안이면 파워업이 플레이어로 끌려옴
pub const POWERUP_MAGNET_SPEED: f32 = 320.0; // 끌려오는 속도(u/s)
pub const SHIELD_SECS: f32 = 7.0;

// Phase 3 — 특수무기
pub const STARTING_SPECIAL_CHARGES: u32 = 1; // 게임 시작 시 보유한 특수무기 충전 수
pub const BEAM_LIFETIME_SECS: f32 = 0.4;
pub const BEAM_WIDTH: f32 = 22.0;
pub const BEAM_LENGTH: f32 = 2000.0;

// Phase 3 — 하이퍼스페이스
pub const HYPERSPACE_COOLDOWN_SECS: f32 = 2.0;

// Phase 4 — 스프라이트 렌더링
pub const VISUAL_FIT: f32 = 1.2; // 콜라이더 반경 대비 스프라이트 표시 배율
pub const Z_BACKGROUND: f32 = -100.0;
pub const Z_FLAME: f32 = -1.0;
pub const Z_ENTITY: f32 = 0.0;
pub const Z_SHIELD: f32 = 5.0;
pub const Z_BEAM: f32 = 10.0;
pub const Z_PARTICLE: f32 = 12.0;
pub const Z_EXPLOSION: f32 = 15.0;
pub const EXPLOSION_FRAME_COUNT: usize = 6;
pub const EXPLOSION_FRAME_SECS: f32 = 0.06;

// Phase 5 — 스테이지/보스
pub const CYCLE_LEN: usize = 3;     // 한 사이클의 스테이지 수(= 무작위로 뽑을 테마 수)
pub const WAVES_PER_STAGE: u32 = 3; // 스테이지당 보스 전 웨이브 수
pub const BULLET_BOSS_DAMAGE: f32 = 1.0;
pub const BEAM_BOSS_DAMAGE: f32 = 4.0;
pub const BOSS_BASE_HEALTH: f32 = 30.0;      // 기준 체력
pub const BOSS_HEALTH_PER_CYCLE: f32 = 14.0; // 사이클마다 증가
pub const BOSS_SCORE_BONUS: u32 = 2000;      // 격파 보너스
