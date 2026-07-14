# Bevy Asteroids — Phase 8 설계 문서: 블랙홀 테마(Black Hole) + 특이점 코어

- **작성일**: 2026-07-14
- **목표**: 테마 풀에 **블랙홀(Black Hole)** 테마와 보스 **특이점 코어**를 추가한다. 트위스트는 **중력장** — 화면 중앙에 고정 블랙홀이 있어 거리² 반비례로 우주선·소행성을 끌어당기고, 사건의 지평선(중심)은 접촉 위험이다. **스테이지 다양화의 마지막 테마**로, 풀이 5→6이 되어 "6개 중 랜덤 3개 = 20가지 조합"이 완성된다.
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10`, macOS arm64. 렌더링은 카툰 스프라이트(자체 SVG→PNG).
- **선행**: Phase 1~7 (main 병합 완료). 진행 프레임워크·보스 인프라·테마 5종(소행성대·외계함대·화염·얼음·전자기폭풍) 존재. `StageModifiers`(물리)·`FogState`(안개) 패턴 확립.

---

## 1. 배경 — 현재 진행 구조

`Progression`이 `THEME_POOL`(현재 5종)에서 `CYCLE_LEN`(=3)개를 뽑아 사이클 구성(3-of-5). 테마 효과는 `theme_params`(배율) + 얼음(`StageModifiers` 벽 반사·미끄럼) + 전자기폭풍(`FogState` 시야 제한)로 표현. 환경 효과 on/off는 `Progression.current_theme()`을 읽는 **stateless sync**(배경·안개 동일 패턴).

---

## 2. 결정 사항 (브레인스토밍 합의)

1. **중력장 = 중앙 고정 블랙홀**. 거리² 반비례 흡인력으로 우주선·소행성을 끌어당김. 중심부(사건의 지평선) 접근 = 피해/즉사. (떠도는 다중 블랙홀/주기 파동 아님.)
2. **질량 무관 가속**: 우주선은 추진으로 저항, 소행성은 저항 못 해 나선으로 빨려듦(물리 정확, 그래비타형 긴장).
3. **보스 = 특이점 코어**: 중앙 블랙홀 위에 자리. 주기적 흡인 강화(잠깐 중력 세짐) + 나선 탄. 멀리서 궤도 돌며 사격, 중심 즉사존 회피.
4. **범위** = 블랙홀만(스테이지 다양화 마지막). 순수 시각/물리 패턴은 기존(StageModifiers/FogState/stateless sync)과 일관되게.

---

## 3. 상세 설계

### 3.1 중력장 (신규)

- **BlackHole 엔티티**: 화면 중앙 상단(우주선 스폰 지점 (0,0)과 겹치지 않게 y≈120, 보스 스폰 위치와 동일)에 스폰. 강착원반 스프라이트 + `Collider`(사건의 지평선 반경). `stateless sync` 시스템 `sync_black_hole`이 `current_theme()==BlackHole`(Playing)일 때만 존재를 보장(그 외 despawn). GameplayEntity로 판 종료 시 정리.
- **중력 바디 마커** `GravityBody`: 우주선·소행성 스폰에 부착(전 테마 공통, 블랙홀 없으면 무효). 총알·적탄은 미부착(중력 무관).
- **`gravity_pull` 시스템**(FixedUpdate): BlackHole이 존재하면 각 `GravityBody`에 중력 가속을 속도에 적분. 순수 함수로 분리:

```rust
/// 블랙홀을 향한 거리² 반비례 가속(중심 근처는 min_dist로 클램프해 발산 방지).
pub fn gravity_accel(body: Vec2, hole: Vec2, strength: f32, min_dist: f32) -> Vec2;
// dir = (hole - body) 정규화, d2 = max(dist², min_dist²), accel = dir * strength / d2
```

- **사건의 지평선**(중심 근접, `EVENT_HORIZON` 반경): 우주선 = 피격(기존 `player_damage` 경로 재사용 → **실드 시 무피해**, 리스폰 무적 존중). 소행성 = 소멸(빨려듦, despawn).
- **on/off**: 블랙홀 없는 테마에선 `gravity_pull`이 무동작(BlackHole 쿼리 비어 있음) → 기존 동작 불변.

### 3.2 진행 통합

- `ThemeId::BlackHole` 추가. `THEME_POOL` = 6종. `CYCLE_LEN` 3 유지 → **3-of-6 = 20가지 조합**(설계 완성).
- `theme_name(BlackHole)` = `"BLACK HOLE"`.
- `theme_params(BlackHole)`: 중력 자체가 난이도 → `count_mul 0.8, speed_mul 0.9, ufo_interval_mul 1.0`(실기 튜닝).
- `stage_modifiers`(얼음 전용)는 `BlackHole`에서 기본값(물리 트위스트 없음 — 중력은 별도 `gravity_pull`이 담당).

### 3.3 보스 — 특이점 코어 (Singularity Core)

```rust
pub enum BossKind { MotherRock, Mothership, BlazingCore, IceGolem, TeslaCore, SingularityCore }
```

- **`boss_for_theme(BlackHole)` = `SingularityCore`**. HP 중간~높음(`SINGULARITY_HEALTH_MUL ≈ 1.3`), 반경 중간.
- **위치**: 중앙 블랙홀 위(y≈120, 거의 고정 + 약한 부유). 블랙홀 스테이지의 보스 단계에선 환경 블랙홀이 계속 존재해 중력을 제공하고, 보스는 그 위에서 체력·공격을 담당(보스는 중력 면역).
- **공격**:
  - **나선 탄**: 발사 각도가 매 발사마다 일정량 회전하는 연속 방사(회전 위상 저장). `spawn_enemy_bullet` 재사용.
  - **주기적 흡인 강화**: 주기적으로 중력 세기를 잠깐 키웠다 되돌림(폭풍 피크처럼). 구현은 `BlackHole.strength`를 시간 함수/타이머로 변조(순수 함수 `gravity_boost(t)` 또는 보스 타이머로 스파이크). 시각·흔들림으로 경고.
- **피해/격파/피격**: 기존 경로 재사용(`apply_boss_damage`, `boss_combat`, `player_damage`). 중심 즉사존은 보스 콜라이더보다 작아, 원거리 사격으로 보스를 때릴 수 있다.

### 3.4 아트 (자체 SVG → PNG)

- **배경** `bg_void.svg`: 매우 어두운 공간, 별이 중심으로 늘어지는(스파게티화) 느낌, 보라/검정 톤.
- **블랙홀** `black_hole.svg`: 검은 원 + 빛나는 강착원반 링(주황/청록).
- **보스** `boss_singularity.svg`: 강착원반을 두른 특이점(블랙홀과 시각 언어 공유하되 보스답게).
- `SpriteAssets`에 `bg_void`, `black_hole`, `boss_singularity` 핸들 추가. `fx::background`에 BlackHole 분기. PNG는 gitignore.

### 3.5 UI / 연출

- 스테이지 시작 배너 "STAGE — BLACK HOLE"(기존 자동). 보스 등장 "! BOSS !" + 등장 임팩트(기존).
- 흡인 강화 시 화면 흔들림 소폭 + (선택) 블랙홀 스프라이트 확대로 경고.

### 3.6 기존 시스템과의 관계 / 변경점 요약

| 파일 | 변경 |
|---|---|
| `core/components.rs` | `GravityBody` 마커 추가 |
| `core/config.rs` | `GRAVITY_STRENGTH`, `GRAVITY_MIN_DIST`, `EVENT_HORIZON`, `BLACK_HOLE_POS_Y`, `SINGULARITY_HEALTH_MUL`, 나선/흡인강화 상수 |
| `core/logic.rs` | `gravity_accel` (필요 시 `gravity_boost`) 순수 함수 |
| `entities/black_hole.rs`(신규) 또는 `fx` | `BlackHole` 컴포넌트, 스폰, `sync_black_hole`, `gravity_pull`, 사건의 지평선 소비 |
| `systems/stage.rs` | `ThemeId::BlackHole`, `THEME_POOL`(6), `theme_params`/`theme_name` arm |
| `entities/boss.rs` | `BossKind::SingularityCore` + 각 arm, 나선 탄·흡인 강화 |
| `entities/{player,asteroid}.rs` | 스폰에 `GravityBody` 부착 |
| `fx/sprites.rs`, `fx/background.rs` | `bg_void`/`black_hole`/`boss_singularity` 핸들·분기 |
| `main.rs` | 신규 플러그인 등록 |
| `assets/sprites/src/` | `bg_void.svg`, `black_hole.svg`, `boss_singularity.svg` |

> **모듈 배치**: 중력·블랙홀은 게임 오브젝트라 `entities/black_hole.rs`(컴포넌트+스폰+플러그인+시스템)에 두는 것이 폴더 규칙에 맞다. `gravity_pull`은 물리라 `systems/`도 후보지만, BlackHole 소유와 응집을 위해 black_hole 모듈에 함께 두고 FixedUpdate에 등록한다(구현 계획에서 확정).

---

## 4. 테스트 전략

렌더/연출은 비테스트. 순수 로직 위주:

- `gravity_accel` — 방향(중심 쪽), 거리² 감쇠(가까울수록 큼), `min_dist` 클램프(중심에서 발산 없음), 같은 지점이면 0.
- 흡인 강화(`gravity_boost` 등) — 주기적, 기본 이상, 피크 존재.
- `theme_params(BlackHole)` — count·speed < 1.0.
- `boss_for_theme(BlackHole)` = SingularityCore, `boss_max_health` 배율(기존 kind 불변).
- `pick_cycle_order` — 풀 6에서 distinct 3개(기존 테스트가 풀 크기 무관하게 통과).
- 기존 테스트 유지·통과.

---

## 5. 리스크 · 오픈 이슈

- **리스폰 위치**: 우주선 스폰(0,0)이 블랙홀(y≈120)과 겹치지 않게 오프셋. 리스폰 무적(Shield) + 사건의 지평선의 `player_damage` 실드 체크로 즉사 연쇄 방지.
- **웨이브 클리어 뉘앙스**: 소행성이 빨려들어 소멸하면 `stage_control`의 `asteroids==0` 조건이 슈팅 없이 충족될 수 있음 → 슈팅에서 **생존**으로 무게 이동. 중력 세기를 적당히 튜닝(소행성은 서서히, 우주선 생존이 주 긴장). `GRAVITY_STRENGTH` 조정으로 흡수.
- **밸런스**: 중력 + 나선 탄 + 흡인 강화 동시 난이도 → `theme_params`·`GRAVITY_STRENGTH`·보스 HP 실기 조정(모두 순수 상수).
- **보스-환경 블랙홀 공존**: 보스 단계에서 환경 블랙홀 유지 + 보스가 그 위에. 보스 콜라이더 > 즉사존이라 원거리 사격 가능.

## 6. 범위 밖 (후속)

- 블랙홀은 스테이지 다양화의 **마지막**. 이후 미뤄둔 4종 제안(특수무기 다양화·타이틀/일시정지 메뉴·BGM·WASM 배포) 재개.
- 다중 블랙홀·이동 블랙홀·중력 렌즈 왜곡 등 — 범위 밖(YAGNI).
