# Bevy Asteroids — Phase 2 설계 문서

- **작성일**: 2026-07-11
- **목표**: Phase 1의 클래식 애스토로이드 게임에 조작감·비주얼·적·진행/저장 기능을 추가한다.
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10` / bevy-persistent `0.11` (0.19 호환 확인됨), macOS arm64
- **선행**: Phase 1 (main에 병합 완료). 기존 플러그인 모듈 구조 위에 최소 침습으로 확장한다.

## 결정 사항 (브레인스토밍 합의)

1. **반대추진(↓) = 브레이크(급감속)** — 후진이 아니라 현재 속도를 빠르게 0으로 줄인다.
2. **적 UFO = 클래식 2종 사격** — 큰 UFO(무작위 조준, 저점수 200) / 작은 UFO(플레이어 정확 조준, 고점수 1000).
3. **난이도 = 완만한 조합** — 웨이브마다 소행성 수·속도·UFO 압박을 함께 소폭 상승.
4. **최고점수 저장 = `bevy-persistent`** — Bevy 리소스를 파일(JSON)에 영속화.
5. **소행성 크기별 속도** — 작을수록 빠름(대<중<소) + 랜덤.

## 신규/변경 모듈

```
src/
├── ufo.rs        (신규) UFO 2종 스폰·이동·사격 AI, 적 총알(EnemyBullet)
├── effects.rs    (신규) 폭발 파티클 + 피격 이펙트 (단명 엔티티, gizmos 렌더)
├── config.rs     (+상수) 브레이크·UFO·난이도·이펙트·색상
├── player.rs     (+) ↓ 브레이크, EngineState, 추진/역추진 화염 렌더, 우주선 컬러
├── asteroid.rs   (+) 크기별 속도, 각속도(회전), 난이도 연동 스폰
├── collision.rs  (+) 폭발/피격 트리거, 총알↔UFO, 적총알↔플레이어, UFO↔플레이어
├── bullet.rs     (+) EnemyBullet 컴포넌트(적 총알)
└── state.rs      (+) Wave(웨이브·난이도) 리소스, HighScore(Persistent) 리소스
```

각 모듈은 기존과 동일하게 대응 `Plugin`을 노출하고 `main.rs`에서 조립한다. 신규 `ufo`, `effects`도 각각 `UfoPlugin`, `EffectsPlugin`으로 등록한다.

## A. 조작/물리 튜닝

### 반대추진(브레이크)
- `player_input`에서 `KeyCode::ArrowDown` 입력 시 속도를 0쪽으로 급감쇠: `velocity.0 = velocity.0.lerp(Vec2::ZERO, (SHIP_BRAKE_RATE * dt).min(1.0))`. 프레임률 독립.
- 상수 `SHIP_BRAKE_RATE`(예: 3.0/s).

### 소행성 크기별 속도
- `AsteroidSize`에 속도 배수/범위를 부여: 작을수록 빠르게. 예) 기준 속도에 `Large=1.0, Medium=1.5, Small=2.0` 배수 + 랜덤.
- `random_velocity`가 크기를 받아 속도를 스케일하도록 변경. 큰 소행성·분열 자식 모두 크기 기반으로 통일(현재 분열 자식의 고정 속도 제거).

## B. 비주얼 폴리시

### 우주선 컬러
- `draw_player`가 `Color::WHITE` 대신 `SHIP_COLOR` 상수(예: 청록 `Color::srgb(0.4, 0.9, 1.0)`)로 렌더.

### 추진/역추진 화염
- `player_input`이 매 프레임 `EngineState { thrusting: bool, braking: bool }` 컴포넌트를 갱신(↑/↓ 눌림 반영).
- `draw_player`가 `EngineState`를 읽어, 추진 중이면 우주선 뒤쪽(-Y 로컬)에 깜빡이는 화염 삼각형을, 브레이크 중이면 앞쪽에 작은 역추진 화염을 gizmos로 그림. 깜빡임은 프레임 인덱스/시간 기반 간단 변조.

### 소행성 회전
- 소행성 스폰 시 `AngularVelocity(f32)`(랜덤, 예: `-1.5..1.5` rad/s) 부여.
- FixedUpdate 시스템 `apply_spin`이 `transform.rotate_z(angular * dt)` 적용. (movement.rs 또는 asteroid.rs)

## C. 이펙트 (`effects.rs`)

### 폭발 파티클
- 공용 함수 `spawn_explosion(commands: &mut Commands, position: Vec2, count: usize)`: `Particle { life: Timer }` + `Velocity`(바깥 방향 랜덤) + `Transform` 엔티티를 `count`개 스폰.
- 시스템: `particle_lifetime`(수명 소멸), 이동은 기존 `apply_velocity` 재사용(Particle에도 Velocity). `draw_particles`가 점/짧은 선으로 gizmos 렌더(수명에 따라 밝기/길이 감소 가능).
- 파티클은 `GameplayEntity`를 붙여 판 종료 시 정리. 화면 순환은 하지 않음(짧은 수명).

### 피격 이펙트
- 우주선 피격 시 `spawn_explosion(commands, player_pos, N)` 재사용(색/개수만 다르게 가능).

## D. 적 UFO (`ufo.rs`)

### 컴포넌트/리소스
- `UfoSize { Large, Small }`; `Ufo { size: UfoSize, fire_timer: Timer }`.
- `UfoSpawnTimer(Timer)` 리소스: 만료 시 UFO 1기 스폰. 간격/소형 확률은 난이도(Wave/Score)에 연동.

### 이동/사격
- 화면 좌/우 가장자리에서 등장해 반대편으로 수평 이동(약간의 상하 사인 흔들림). 화면 밖으로 나가면 despawn(순환 없음).
- `fire_timer` 만료 시 발사: 큰 UFO = 무작위 방향, 작은 UFO = 플레이어 방향 조준(`(player_pos - ufo_pos).normalize()`). `EnemyBullet` 스폰: `Velocity` + 수명 타이머 보유, **`Wrapping` 없이 수명 만료 시 소멸**(플레이어 총알과 달리 화면 순환하지 않음).

### 점수/충돌
- 플레이어 `Bullet` ↔ `Ufo`: UFO 파괴 + 점수(`Large=200, Small=1000`) + `spawn_explosion`.
- `EnemyBullet` ↔ `Player`: 목숨↓ + 피격 이펙트(+ 기존 리스폰/게임오버 로직 재사용).
- `Ufo` ↔ `Player`: 목숨↓ + 이펙트.

### 조준 순수 함수
- `aim_direction(from: Vec2, to: Vec2) -> Vec2`(정규화, 영벡터 방어) — 단위 테스트 대상.

## E. 진행/저장 (`state.rs` 확장)

### 난이도 (Wave)
- `Wave(u32)` 리소스. `OnEnter(Playing)`에서 1로 리셋, 소행성 전멸 시(`wave_control`) +1 하고 다음 웨이브 스폰.
- 순수 함수로 스케일 공식 분리(테스트 대상):
  - `asteroid_count_for_wave(wave) -> usize` = `min(BASE + wave, MAX)`.
  - `asteroid_speed_scale(wave) -> f32` = `1.0 + wave * STEP`(상한).
  - `ufo_interval_for_wave(wave) -> f32`(웨이브↑ → 간격↓, 하한).
  - `small_ufo_probability(wave) -> f32`(웨이브↑ → 소형 확률↑, 상한).

### 최고점수 (bevy-persistent)
- `#[derive(Resource, Serialize, Deserialize, Default)] struct HighScore(u32)` 를 `Persistent<HighScore>` 로 관리.
- 앱 시작 시 플랫폼 데이터 디렉토리의 JSON 파일에서 로드(없으면 0).
- 게임오버 진입 시 `score.0 > high.0` 면 갱신 후 `persist()`.
- HUD와 게임오버 화면에 최고점수 표시.

## 테스트 전략

순수 로직을 먼저 TDD로 확정하고 그 위에 시스템을 얹는다.

- `brake` 감쇠(속도가 줄되 방향 유지, dt 반영)
- `AsteroidSize` 크기별 속도 스케일(대<중<소)
- 난이도 공식 4종(`asteroid_count/speed/ufo_interval/small_prob`)의 경계값(상·하한)
- `update_high_score(current, new) -> u32`(더 큰 값 유지)
- `aim_direction`(정규화, 영벡터 방어)
- 시스템 테스트: UFO 스폰/사격 타이머, 충돌 확장(총알↔UFO, 적총알↔플레이어), 파티클 수명 소멸, 웨이브 증가.
- 렌더(화염·파티클·색상·회전)는 `cargo run` 수동 확인.

## 완료 기준 (Definition of Done)

- ↓로 우주선이 급감속(브레이크)하고 역추진 화염이 보인다.
- 소행성이 작을수록 빠르게 움직이고, 각자 회전한다.
- 우주선이 지정 색으로 표시되고, 추진 시 화염이 나온다.
- 소행성/UFO 파괴 시 폭발 파티클이, 피격 시 피격 이펙트가 나온다.
- 큰/작은 UFO가 주기적으로 등장해 각각 무작위/정확 조준으로 사격하고, 파괴 시 200/1000점을 준다.
- 웨이브가 오를수록 소행성 수·속도·UFO 압박이 완만히 증가한다.
- 최고점수가 파일에 저장되어 재실행 후에도 유지되고, HUD·게임오버 화면에 표시된다.
- 순수 로직 함수 단위 테스트가 통과한다.

## 범위 밖(향후 후보)

하이퍼스페이스(순간이동), 사운드/음악, 파워업 아이템, 배경 별, 화면 흔들림(카메라 셰이크).
