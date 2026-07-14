# Bevy Asteroids — Phase 7 설계 문서: 전자기폭풍 테마(EM Storm) + 테슬라 코어

- **작성일**: 2026-07-14
- **목표**: 테마 풀에 **전자기폭풍(EM Storm)** 테마와 보스 **테슬라 코어**를 추가한다. 이 테마의 트위스트는 **시야 제한** — 화면을 어둠으로 덮고 우주선 주변만 밝은 원형 시야창을 두며, 주기적 폭풍 피크에 시야창이 좁아졌다 회복한다. 풀이 4→5로 늘어 사이클 조합이 "5개 중 랜덤 3개 = 10가지"가 된다.
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10`, macOS arm64. 렌더링은 카툰 스프라이트(자체 SVG→PNG).
- **선행**: Phase 1~6 (main 병합 완료). 진행 프레임워크(`Progression`)·보스 인프라(`BossKind`)·테마 4종(소행성대·외계함대·화염·얼음) 존재.

---

## 1. 배경 — 현재 진행 구조

`Progression`이 `THEME_POOL`(현재 4종)에서 `CYCLE_LEN`(=3)개를 무작위로 뽑아 사이클을 구성한다(3-of-4 = 4가지 조합). 테마 효과는 `theme_params`(단순 배율) + 얼음의 물리 트위스트(`StageModifiers` 벽 반사·미끄럼)로 표현된다. 배경은 `fx::background`가 `Progression.current_theme()`을 읽어 테마별 스프라이트로 교체(상태 비의존).

---

## 2. 결정 사항 (브레인스토밍 합의)

1. **시야 제한 = 안개(지속 시야창) + 간헐 폭풍 피크**. 평소 중간 크기 원형 시야창, 주기적으로 확 좁아졌다 회복. (상시 암전만/주기 정전만 아님 → 지속 긴장 + 리듬.)
2. **보스 = 테슬라 코어**: 블링크(순간이동)로 어둠 속을 옮겨다니며, EMP 방사 링(폭풍 피크와 동기) + 플레이어 조준 체인 전격.
3. **범위** = EM Storm만. 마지막 블랙홀은 이후 별도 PR.
4. **안개는 순수 시각 효과**: 물리(`StageModifiers`)는 손대지 않고, `fx::background`처럼 `Progression.current_theme()`을 읽어 on/off. HUD는 UI라 안개 위에 렌더되어 가려지지 않음. 별도 레이더/보조 없음(순수 가시성 도전).

---

## 3. 상세 설계

### 3.1 시야 제한 안개 (fx 모듈, 신규 `fx/fog.rs`)

- **오버레이 엔티티**(Startup 1회 스폰): `FogOverlay` 마커 + 방사형 그라데이션 스프라이트(중심 투명 → 가장자리 불투명 검정) + 높은 Z(`Z_FOG`, 모든 월드 스프라이트 위). 화면보다 충분히 크게(어느 위치에서든 화면 전체를 덮도록).
- **매 프레임 갱신 시스템 `update_fog`**:
  - `Progression.current_theme()`이 `EmStorm`이면 `Visibility::Visible`, 아니면 `Hidden`(그 외 테마는 안개 없음).
  - 보이는 동안: 오버레이 `translation`을 우주선 위치에 맞춤(시야창이 우주선을 따라감). `Transform.scale`(또는 `custom_size`)을 `storm_vision_scale(t)`로 설정 → 시야창 크기 진동.
  - **커버리지 보장**: 최소 시야 배율에서도 우주선이 화면 구석에 있을 때 반대편까지 덮이도록 기본 크기를 화면 대각선 이상으로 크게 잡는다(플랜에서 정확한 수치 확정).
- **폭풍 피크(순수 함수, 테스트 대상)**:

```rust
/// 폭풍 강도 펄스: 평소 0, 주기(STORM_PERIOD)마다 부드럽게 1로 치솟았다 가라앉음. [0,1].
pub fn storm_pulse(t: f32) -> f32;

/// 시야 배율: 평소 FOG_VISION_BASE, 피크에서 FOG_VISION_MIN까지 축소. (base 이하로만 변함)
pub fn storm_vision_scale(t: f32) -> f32; // = FOG_VISION_BASE * (1 - STORM_DEPTH * storm_pulse(t))
```

- HUD는 `ui.rs`의 UI 노드(월드 카메라 위 렌더)라 안개 영향 없음. 우주선은 시야창 중심이라 항상 보임. 총알은 어둠 속으로 날아감(의도된 도전).

### 3.2 진행 통합

- `ThemeId::EmStorm` 추가. `THEME_POOL` = 5종. `CYCLE_LEN` 3 유지 → **3-of-5 = 10가지 조합**.
- `theme_name(EmStorm)` = `"EM STORM"`.
- `theme_params(EmStorm)`: 시야 제한 자체가 난이도라 낮춤 — `count_mul 0.85, speed_mul 0.9, ufo_interval_mul 1.0`(실기 튜닝).
- `pick_cycle_order`/`advance_stage` 등은 풀 크기 무관하게 동작(변경 불필요). `stage_modifiers`(얼음 전용)는 `EmStorm`에서 기본값 반환(물리 트위스트 없음).

### 3.3 보스 — 테슬라 코어 (Tesla Core)

```rust
pub enum BossKind { MotherRock, Mothership, BlazingCore, IceGolem, TeslaCore } // 변형 추가
```

- **`boss_for_theme(EmStorm)` = `TeslaCore`**. HP 배율 중간(`TESLA_HEALTH_MUL ≈ 1.1`), 반경 중간(~55).
- **이동 — 블링크**: 주기(`BLINK_INTERVAL`)마다 순간이동. 짧은 예고 섬광(스프라이트 깜빡/축소) 후 새 위치(화면 내, 때로 우주선에서 먼 = 시야 밖)로 이동. `boss_movement`의 TeslaCore arm에서 처리(블링크 타이머는 별도 컴포넌트 또는 재사용).
- **공격**:
  - **EMP 방사 링**: 전방위 전격탄 N발. **폭풍 피크와 동기** — `storm_pulse(t)`가 임계 이상으로 오르는 순간 발동(엣지 감지). 어둠이 짙어지는 순간 사방에서 탄이 퍼짐.
  - **체인 전격**: 플레이어 조준 3갈래 탄(`aim_direction` + 각도), 별도 주기(`BossAttack` 타이머).
  - 탄은 기존 `spawn_enemy_bullet` 재사용.
- **피해/격파/피격**: 기존 경로 재사용(`apply_boss_damage`, `boss_combat`, `player_damage`).

### 3.4 아트 (자체 SVG → PNG)

- **배경** `bg_storm.svg`: 어두운 뇌운, 번개 실루엣, 청보라/회색 톤.
- **보스** `boss_tesla_core.svg`: 전기 코일/구체(청백 스파크, 두꺼운 외곽선, 카툰).
- **안개** `fog.svg`: 방사형 그라데이션(중심 `rgba(0,0,0,0)` → 가장자리 `rgba(0,0,0,1)`), 시야창 반경만큼 투명 후 검정으로 램프. resvg `radialGradient` 사용.
- `SpriteAssets`에 `bg_storm`, `boss_tesla_core`, `fog` 핸들 추가. `fx::background`에 EmStorm 분기. PNG는 gitignore.

### 3.5 UI / 연출

- 스테이지 시작 배너 "STAGE — EM STORM"(기존 `announce_stage` 자동). 보스 등장 "! BOSS !" + 등장 임팩트(기존 `boss_entrance`).
- 블링크 시 짧은 섬광/사운드로 순간이동을 알림. EMP 발동 시 화면 흔들림 소폭.

### 3.6 기존 시스템과의 관계 / 변경점 요약

| 파일 | 변경 |
|---|---|
| `core/config.rs` | `Z_FOG`, `FOG_VISION_BASE`/`FOG_VISION_MIN`/`STORM_PERIOD`/`STORM_DEPTH`, `TESLA_HEALTH_MUL`, `BLINK_INTERVAL`, `TESLA_ATTACK_INTERVAL`, EMP/체인 상수 |
| `core/logic.rs` 또는 `fx/fog.rs` | `storm_pulse`/`storm_vision_scale` 순수 함수 |
| `fx/fog.rs`(신규) + `fx.rs` | `FogOverlay`, `FogPlugin`, `update_fog`(Progression 기반 on/off·추적·시야 진동) |
| `systems/stage.rs` | `ThemeId::EmStorm`, `THEME_POOL`(5), `theme_params`/`theme_name`/`stage_modifiers` arm |
| `entities/boss.rs` | `BossKind::TeslaCore` + 각 match arm, 블링크 이동, EMP(폭풍 동기)·체인 공격, HP 배율 |
| `fx/sprites.rs`, `fx/background.rs` | `bg_storm`/`boss_tesla_core`/`fog` 핸들·분기 |
| `main.rs` | `FogPlugin` 등록 |
| `assets/sprites/src/` | `bg_storm.svg`, `boss_tesla_core.svg`, `fog.svg` |

---

## 4. 테스트 전략

렌더/연출은 비테스트. 순수 로직 위주:

- `storm_pulse` — 주기적, [0,1], 피크 존재(어느 t에서 임계 이상).
- `storm_vision_scale` — [FOG_VISION_MIN, FOG_VISION_BASE] 범위, 피크에서 최소, 평소 base.
- `theme_params(EmStorm)` — count·speed < 1.0.
- `boss_for_theme(EmStorm)` = TeslaCore, `boss_max_health` 배율(기존 kind 불변).
- `pick_cycle_order` — 풀 5에서 distinct 3개(기존 테스트가 풀 크기 무관하게 통과).
- 기존 테스트 유지·통과.

---

## 5. 리스크 · 오픈 이슈

- **안개 커버리지**: 우주선이 화면 구석일 때 반대편 미커버(밝은 누출) 방지 → 기본 오버레이 크기를 넉넉히(대각선 이상). 텍스처는 저해상 그라데이션을 확대(부드러움).
- **난이도 스파이크**: 시야 제한 + 어둠 속 충돌이 과할 수 있음 → 시야창 기본 크기 넉넉히, `theme_params`·`STORM_DEPTH` 실기 튜닝(모두 순수 상수).
- **블링크 체감**: 순간이동이 갑작스러워 불쾌하지 않도록 예고 섬광/사운드 필수.
- **EMP-폭풍 동기 구현**: `storm_pulse` 임계 상승 엣지 감지(`Local<bool>` 등)로 1회 발동. 위상 계산은 공유 `elapsed_secs` 기반.
- **성능**: 큰 안개 스프라이트 1장 매 프레임 갱신(translation/scale) — 저비용.

## 6. 범위 밖 (후속)

- 블랙홀(중력장) 테마 + 보스 — 마지막 테마, 별도 설계→PR(풀 6 완성 → "6개 중 랜덤 3개" = 20가지).
- 안개가 물리/조작에 개입하는 확장(정전기 조작 방해 등) — 이번 범위 밖(YAGNI).
- 보스 다단계 페이즈 — 이번 범위 밖.
