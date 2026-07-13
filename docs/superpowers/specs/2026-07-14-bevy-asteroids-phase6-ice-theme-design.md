# Bevy Asteroids — Phase 6 설계 문서: 얼음 테마(Frozen Field) + 얼음 골렘

- **작성일**: 2026-07-14
- **목표**: 테마 풀에 **얼음(Frozen Field)** 1종과 그 보스 **얼음 골렘**을 추가한다. 얼음은 기존 테마의 단순 배율을 넘어 **물리 트위스트**(위험요소 벽 반사 + 우주선 미끄러움)를 도입한다. 이를 위해 스테이지 진입/이탈 시 켜지고 꺼지는 **활성 스테이지 수정자(StageModifiers)** 인프라를 신설한다. 풀이 3→4로 늘어 사이클 조합이 처음으로 다양해진다(3-of-4 = 4가지).
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10` / bevy-persistent `0.11`, macOS arm64. 렌더링은 Phase 4 카툰 스프라이트(자체 SVG→PNG) 기반.
- **선행**: Phase 1~5 (main 병합 완료). 진행 프레임워크(`Progression`)·보스 인프라(`BossKind`)·테마 3종 존재.

---

## 1. 배경 — 현재 진행 구조

`Progression`이 `THEME_POOL`(현재 3종: 소행성대·외계함대·화염)에서 `CYCLE_LEN`(=3)개를 무작위로 뽑아 사이클을 구성한다. 풀=3, CYCLE_LEN=3이므로 **매 사이클이 항상 같은 3테마**(순서만 셔플) → 실질적 다양성이 없다. 테마 효과는 `theme_params`의 **단순 배율**(소행성 수·속도·UFO 빈도)뿐이라 매 프레임 물리 효과는 표현할 수 없다. 화면 경계 처리는 `wrap_around`가 `With<Wrapping>` 전체(우주선·소행성·총알)를 순환시킨다.

---

## 2. 결정 사항 (브레인스토밍 합의)

1. **전체 방향** = 남은 테마 3종(얼음·전자기폭풍·블랙홀)을 추가해 풀을 6으로 완성. **테마별로 설계→구현→PR**로 나눠 진행하며, 순서는 **얼음 → 전자기폭풍 → 블랙홀**. 본 스펙은 **얼음만** 다룬다.
2. **얼음 트위스트**:
   - **벽 반사**: 소행성·파편·보스는 화면 경계에서 튕긴다(반사). **우주선과 총알은 기존대로 순환**(플레이어 탈출구 유지).
   - **미끄러움**: 우주선 마찰 감소(더 미끄럽게 미끄러짐). 순환은 유지.
3. **얼음 보스 = 얼음 골렘**: 느린 드리프트(벽 반사) + 광역 파편 휘두르기 + 내려찍기 충격파. **다단계 페이즈**(피격할수록 껍질이 벗겨져 작아지고 빨라짐). 다단계 보스 페이즈는 Phase 5에서 범위 밖이었으며 여기서 도입한다.
4. **재사용**: 기존 진행·보스 인프라·소행성 분열·적 탄을 재사용해 그 위에 얹는다. 비(非)얼음 스테이지 동작은 100% 동일하게 유지한다.

---

## 3. 상세 설계

### 3.1 트위스트 인프라 — 활성 스테이지 수정자

매 프레임 물리에 개입하는 테마 효과를 위해 리소스를 신설한다.

```rust
#[derive(Resource)]
pub struct StageModifiers {
    pub wall_bounce: bool,  // 위험요소가 경계에서 반사되는가
    pub ship_damping: f32,  // 우주선 마찰(작을수록 잘 미끄러짐)
}
// 기본값(비얼음): { wall_bounce: false, ship_damping: SHIP_DAMPING(=0.985) }

pub fn stage_modifiers(theme: ThemeId) -> StageModifiers  // 순수 함수, 테스트 대상
// FrozenField => { wall_bounce: true, ship_damping: SHIP_DAMPING_ICE(=0.997) }
// 그 외       => 기본값
```

- **삽입/갱신**: `StagePlugin`에서 리소스를 삽입(기본값). **스테이지 시작마다** `stage_modifiers(current_theme())`로 덮어쓴다(첫 스테이지 = `start_first_stage`, 이후 = `stage_control`의 다음 웨이브/스테이지 진입 지점, 보스 격파 후 다음 스테이지). 재시작(`reset_game`) 시 기본값으로 리셋.

### 3.2 벽 반사 — 마커 + 시스템 분리

- **마커 컴포넌트** `EdgeReflect`(core::components)를 **반사 대상**(소행성·파편·얼음 골렘)에 부착한다. 파편은 소행성이므로 `spawn_asteroid`가 자동 부여.
- **경계 처리 분리**:
  - `wrap_around`: `Query<&mut Transform, (With<Wrapping>, Without<EdgeReflect>)>` — 우주선·총알만 순환(변경점: `Without<EdgeReflect>` 추가).
  - **신규** `reflect_or_wrap`: `Query<(&mut Transform, &mut Velocity), With<EdgeReflect>>` + `Res<StageModifiers>`. `wall_bounce`가 켜지면 **반사**, 아니면 기존 `wrap_position`으로 순환.
  - 소행성 스폰은 기존 `Wrapping` 대신 **`EdgeReflect`**를 부착한다(이중 처리 방지: EdgeReflect 엔티티는 `wrap_around`에서 제외됨).
- **반사 순수 함수**(테스트 대상):

```rust
/// 경계를 넘으면 경계로 클램프하고 해당 축 속도를 반전한다.
pub fn reflect_edge(pos: Vec2, vel: Vec2, half: Vec2) -> (Vec2, Vec2)
// 예: pos.x > half.x  => pos.x = half.x, vel.x = -vel.x.abs()
//     pos.x < -half.x => pos.x = -half.x, vel.x = vel.x.abs()
//     y축 동일. 두 축 독립 처리.
```

- **비얼음 스테이지**: `wall_bounce=false` → `reflect_or_wrap`이 `wrap_position` 호출 → **기존과 동일**한 순환. (소행성이 `Wrapping`→`EdgeReflect`로 바뀌어도 순환 동작은 보존된다.)
- **실행 스케줄**: `reflect_or_wrap`는 `wrap_around`와 같은 `FixedUpdate`에서 속도 적분(`apply_velocity`) 이후 실행.

### 3.3 미끄러움 — 감쇠 소스 교체

`apply_ship_damping`이 상수 `SHIP_DAMPING` 대신 `Res<StageModifiers>.ship_damping`을 읽는다. 기본 0.985 → 얼음 0.997(마찰↓, 잘 미끄러짐). 최고 속도 제한(`SHIP_MAX_SPEED`)은 유지. (브레이크율은 이번 범위에선 그대로 두고 실기 후 조정.)

### 3.4 진행 통합

- `ThemeId::FrozenField` 추가. `THEME_POOL` = 4종(소행성대·외계함대·화염·얼음). `CYCLE_LEN`은 3 유지 → **3-of-4 = 4가지 조합**.
- `theme_name(FrozenField)` = `"FROZEN FIELD"`.
- `theme_params(FrozenField)`: 벽 반사만으로 위협이 오래 지속되므로 수·속도는 낮춤 — `count_mul: 0.9, speed_mul: 0.95, ufo_interval_mul: 1.0`(실기 튜닝).
- `pick_cycle_order`/`advance_stage`/`stage_wave_number`는 풀 크기와 무관하게 동작(변경 불필요). `new_progression`의 `debug_assert!(THEME_POOL.len() >= CYCLE_LEN)`는 4≥3로 계속 성립.

### 3.5 보스 — 얼음 골렘 (Ice Golem)

```rust
pub enum BossKind { MotherRock, Mothership, BlazingCore, IceGolem }  // 변형 추가

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
    pub phase: u8,   // 신규: 다단계 페이즈(0..=2). 껍질 깨짐 전이 감지용.
}
```

- **`boss_for_theme(FrozenField)` = `IceGolem`**. `spawn_boss`에 `EdgeReflect` + `Velocity`(느린 초기 드리프트) 부여, `phase: 0`.
- **HP(탱키)**: `boss_max_health(kind, cycle)`에 **kind별 배율** 적용 — `ICE_GOLEM_HEALTH_MUL = 1.4`(느린 대신 높은 체력). 기존 보스는 배율 1.0(동작 불변).
- **다단계 페이즈**(순수 함수 `golem_phase(health, max_health) -> u8`, 테스트 대상):
  - `health > 2/3·max` → 0, `> 1/3·max` → 1, 그 이하 → 2.
  - 매 프레임 계산한 phase가 저장된 `phase`보다 크면 **껍질 깨짐 전이**: `Boss.phase` 갱신 + 스프라이트 축소 + 파편 폭발 연출(`spawn_explosion`류 파티클) + 화면 흔들림 + 방사형 파편 방출.
  - phase별 파라미터: 스프라이트 배율 `[1.0, 0.8, 0.62]`, 이동속도 배율 `[1.0, 1.35, 1.75]`, 공격 주기 배율 `[1.0, 0.8, 0.62]`(순수 함수 `golem_scale/speed_mul/attack_mul(phase)`).
- **이동 `boss_movement`(IceGolem arm)**: 느린 등속 드리프트. 경계 반사는 `EdgeReflect`가 처리(별도 순환 코드 불필요). 속도 크기 = 기본 × `golem_speed_mul(phase)`.
- **공격 `boss_attack`(IceGolem arm)**: 기존 `BossAttack { timer }` 재사용. 발사 횟수 패리티로 두 공격 교대:
  - **팔 휘두르기**(짝수 발): 소형 소행성 파편 5개를 플레이어 조준 방향 ±35° 부채꼴로 사출(`spawn_asteroid` Small, 벽 반사).
  - **내려찍기 충격파**(홀수 발): 적 탄 12발을 균등 각도 방사형으로 발사(`spawn_enemy_bullet` 재사용).
  - 주기 = 기본 간격 × `golem_attack_mul(phase)`.
- **피해/격파/피격**: 기존 경로 재사용 — 총알/빔이 체력 감소(`apply_boss_damage`), `health<=0` 격파(큰 폭발 + 점수 보너스 + `advance_stage` + 다음 웨이브 + 목숨 리필), 접촉/보스 탄 피격은 `player_damage`가 이미 보스/적탄을 처리.

### 3.6 아트 (자체 SVG → PNG)

- **배경** `bg_ice.svg`: 창백한 청/백 팔레트, 균열 얼음·성에, 차가운 톤. 기존 배경 3종과 구분되는 실루엣.
- **보스** `boss_ice_golem.svg`: 청록 얼음 덩어리 골렘, 두꺼운 어두운 외곽선 + 하이라이트 1톤(카툰 스타일 일관). 페이즈 축소는 `custom_size` 스케일로 처리(스프라이트 1장).
- `SpriteAssets`에 `bg_ice`, `boss_ice_golem` 핸들 추가. `build_sprite_assets`/`dummy_sprite_assets` 갱신. 배경 교체(`fx::background`)에 얼음 분기 추가. PNG는 gitignore.

### 3.7 UI / 연출

- 스테이지 시작 배너 "STAGE — FROZEN FIELD"(기존 `announce_stage` 자동 반영). 보스 등장 "! BOSS !"(기존).
- 껍질 깨짐 시 파편 폭발 + 흔들림으로 페이즈 전환을 시각·촉각적으로 알림.
- 보스 체력 바(기존) 그대로.

### 3.8 기존 시스템과의 관계 / 변경점 요약

| 파일 | 변경 |
|---|---|
| `core/components.rs` | `EdgeReflect` 마커 추가 |
| `core/config.rs` | `SHIP_DAMPING_ICE`, `ICE_GOLEM_HEALTH_MUL`, 골렘 기본 속도/공격간격 상수 |
| `systems/stage.rs` | `ThemeId::FrozenField`, `THEME_POOL`(4), `theme_params`/`theme_name`/`stage_modifiers`, 스테이지 시작 시 `StageModifiers` 갱신 |
| `systems/movement.rs` | `wrap_around`에 `Without<EdgeReflect>`, 신규 `reflect_or_wrap`, `apply_ship_damping`가 `StageModifiers` 참조; `StageModifiers` 리소스 정의/삽입 위치 결정 |
| `entities/asteroid.rs` | 스폰 시 `Wrapping`→`EdgeReflect` |
| `entities/boss.rs` | `BossKind::IceGolem`, `Boss.phase`, `boss_for_theme`, `boss_max_health` kind 배율, `spawn_boss`(EdgeReflect), 이동/공격 arm, 페이즈 전이 |
| `core/state.rs` | `reset_game`에서 `StageModifiers` 기본값 리셋 |
| `fx/sprites.rs`, `fx/background.rs` | 얼음 배경/보스 핸들·분기 |
| `assets/sprites/src/` | `bg_ice.svg`, `boss_ice_golem.svg` |

> `StageModifiers` 리소스의 소유 위치: 물리 시스템이 주로 읽으므로 `systems/movement.rs`에 정의하고 `MovementPlugin`에서 기본값 삽입, 갱신은 `stage.rs`가 `stage_modifiers()`로 수행. (또는 `stage.rs` 소유 — 구현 계획에서 확정.)

---

## 4. 테스트 전략

렌더/보스 연출은 비테스트. 순수 로직·상태 전이 위주:

- `stage_modifiers` — 얼음=반사 on·미끄럼(damping<기본), 그 외=기본값.
- `reflect_edge` — 경계 초과 시 클램프 + 해당 축 속도 반전, 두 축 독립, 내부 좌표는 불변.
- `golem_phase` — 2/3·1/3 임계값에서 0→1→2, 단조.
- `golem_scale/speed_mul/attack_mul` — phase 증가 시 크기↓·속도↑·주기↓.
- `boss_max_health` — IceGolem 배율(>기존), 기존 kind 불변, 사이클 스케일 유지.
- `pick_cycle_order` — 풀 4에서 distinct 3개(기존 테스트가 풀 크기 무관하게 통과).
- `theme_params(FrozenField)` — count·speed<1.0.
- 기존 68개 테스트 유지·통과.

---

## 5. 리스크 · 오픈 이슈

- **밸런스**: 벽 반사 + 미끄럼이 겹쳐 체감 난이도가 급등할 수 있음 → `theme_params`·`SHIP_DAMPING_ICE`·골렘 HP/속도는 실기 조정(모두 순수 상수).
- **반사 튜닝**: 파편이 경계에서 무한 왕복하며 화면에 오래 남을 수 있음 → 소행성 수명/개수로 조절(테마 count_mul 낮춤으로 1차 완화).
- **골렘 페이즈 연출**: 껍질 깨짐 시 방출 파편이 즉시 위협이 되지 않도록 초기 속도/개수 조절.
- **StageModifiers 소유·순서**: 물리 시스템이 읽기 전에 스테이지 시작에서 갱신되도록 스케줄 순서 확인.

## 6. 범위 밖 (후속)

- 전자기폭풍(시야 제한), 블랙홀(중력장)과 각 보스 — 각자 별도 설계→PR.
- 미끄럼의 브레이크 영향, 얼음 파편의 냉기 디버프(플레이어 감속) 등 추가 기믹 — 이번 범위 밖(YAGNI).
- 보스 4단계 이상 페이즈, 보스 고유 드롭 — 범위 밖.
