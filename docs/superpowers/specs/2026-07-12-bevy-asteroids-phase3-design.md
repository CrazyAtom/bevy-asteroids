# Bevy Asteroids — Phase 3 설계 문서

- **작성일**: 2026-07-12
- **목표**: Phase 2 게임에 사운드·파워업·특수무기·배경·화면 흔들림·하이퍼스페이스를 추가하고, `src/`를 도메인별 폴더로 재구성한다.
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10` / bevy-persistent `0.11` (+ Bevy 오디오, `bevy_audio` — DefaultPlugins 기본 포함), macOS arm64
- **선행**: Phase 1·2 (main에 병합 완료).

## 결정 사항 (브레인스토밍 합의)

1. **사운드** = 코드로 합성한 WAV 에셋 (외부 다운로드 0). `assets/sounds/`에 생성, Bevy `AudioPlayer`로 재생.
2. **파워업 5종** = 실드(일시 무적) · 연사(발사속도↑) · 확산탄(3-way) · 추가 목숨(+1) · **특수무기 충전(+1)**. 적 파괴 시 낮은 확률로 드롭.
3. **특수무기(SpecialWeapon)** = 확장 가능한 enum 구조. 현재 `LaserBeam` 하나(정면 굵은 레이저빔). X 키로 발동, 충전 소모.
4. **배경** = 정적 트윙클 별밭 (카메라 고정이라 패럴랙스 불필요).
5. **화면 흔들림** = trauma 기반 카메라 오프셋 (피격/폭발/특수무기 트리거, 시간 감쇠).
6. **하이퍼스페이스** = H 키, 무작위 위치 순간이동(무적 없음, 리스크 포함), 짧은 쿨다운.
7. **프로젝트 구조** = 단일 크레이트 유지 + `src/` 도메인별 폴더 그룹핑 (Phase 3 첫 태스크).

## 프로젝트 구조 재구성 (Task 1)

단일 크레이트를 유지하되 `src/`를 도메인별 폴더로 정리한다. 모든 `use crate::...` 경로를 새 경로로 갱신하고 `cargo test`로 무결성을 확인한다.

```
src/
├── main.rs
├── core.rs            # pub mod config; logic; components; state;
│   core/
│   ├── config.rs
│   ├── logic.rs
│   ├── components.rs
│   └── state.rs
├── entities.rs        # pub mod player; bullet; asteroid; ufo; powerup;
│   entities/
│   ├── player.rs
│   ├── bullet.rs
│   ├── asteroid.rs
│   ├── ufo.rs
│   └── powerup.rs      (신규)
├── systems.rs         # pub mod movement; collision;
│   systems/
│   ├── movement.rs
│   └── collision.rs
├── fx.rs              # pub mod effects; background; shake; audio;
│   fx/
│   ├── effects.rs
│   ├── background.rs   (신규)
│   ├── shake.rs        (신규)
│   └── audio.rs        (신규)
└── ui.rs              # (단일 파일 유지)
```

- Rust 2018 스타일: 그룹 루트 파일(`core.rs` 등)이 하위 모듈을 `pub mod ...;`로 선언, 실제 파일은 동명 폴더 안에 둔다.
- import 경로는 그룹 접두사가 붙는다: `crate::core::config`, `crate::entities::player`, `crate::systems::collision`, `crate::fx::effects` 등. `main.rs`는 `mod core; mod entities; mod systems; mod fx; mod ui;` 를 선언.
- 재구성은 순수 이동/경로 갱신이며 기능 변화 없음. 각 파일의 테스트도 그대로 통과해야 한다.

## 1. 사운드 (`fx/audio.rs` + `assets/sounds/`)

- 효과음을 **코드로 PCM 합성해 WAV로 생성**한다: 발사, 소행성 폭발, UFO 폭발, 추진, UFO 사격, 파워업 획득, 특수무기 발동, 하이퍼스페이스, 게임오버 등. 톤/노이즈/엔벨로프를 조합한 간단한 합성.
- **생성 방식**: 크레이트 루트의 **`build.rs`가 빌드 시 `assets/sounds/*.wav`가 없으면 PCM을 합성해 생성**한다(자립형 WAV 인코더 포함, 외부 크레이트/다운로드 불필요). 생성된 WAV는 `.gitignore`에 추가(커밋하지 않음) → 저장소엔 코드만, 실행 시 파일은 자동 생성.
- **재생**: Bevy 이벤트 `#[derive(Event)] SfxEvent(Sfx)` (Sfx는 enum). 게임 시스템(발사·충돌 등)은 `EventWriter<SfxEvent>`로 이벤트만 쏘고, `play_sfx` 시스템이 이를 읽어 해당 `Handle<AudioSource>`로 `AudioPlayer`를 스폰(`PlaybackSettings::DESPAWN`). 핸들은 `SfxAssets` 리소스에 시작 시 로드.
- 디커플링: 소리를 내는 각 시스템은 오디오 구현을 몰라도 되고, 이벤트만 발행한다.

## 2. 파워업 (`entities/powerup.rs`)

### 컴포넌트/타입
- `#[derive(Clone, Copy)] enum PowerupKind { Shield, RapidFire, Spread, ExtraLife, SpecialWeapon(SpecialWeaponKind) }`
- `Powerup { kind: PowerupKind }` (Component): 표류하는 수집 아이템. `Velocity`(느린 표류) + `Collider` + 수명 `Timer` + gizmo 아이콘 + `GameplayEntity`, `Wrapping` 없음.

### 드롭
- 소행성/UFO 파괴 시(collision) 낮은 확률(`POWERUP_DROP_CHANCE`)로 파괴 위치에 파워업 스폰. 종류는 가중 랜덤(`pick_powerup_kind` 순수 함수, 특수무기는 낮은 가중치).

### 수집 & 효과
- `player_vs_powerup`(collision): 우주선이 아이템에 닿으면 효과 적용 + `SfxEvent(Pickup)` + 아이템 despawn.
- **실드**: 플레이어에 `Shield(Timer)` 부여. `player_damage`는 실드 활성 시 **조기 반환(피해 무시)** — 들어온 총알/소행성은 그대로 통과(별도 소모 없음). 우주선 주위 실드 원 gizmo.
- **연사**: `RapidFire(Timer)` 부여 → 발사 쿨다운 단축.
- **확산탄**: `Spread(Timer)` 부여 → 발사 시 3-way(정면 ±`SPREAD_ANGLE`).
- **추가 목숨**: 즉시 `lives.0 += 1`.
- **특수무기 충전**: 플레이어 `SpecialWeapon` 상태의 `kind`를 장착하고 `charges += 1`.

### 발사 모델 변경 (연사/확산탄 대응)
- 현재 발사는 `just_pressed(Space)` = 탭 1발. 이를 **쿨다운 기반**으로 개선: 플레이어에 `FireCooldown(Timer)`. `Space`가 `pressed`이고 쿨다운이 준비되면 발사 후 쿨다운 리셋(`FIRE_INTERVAL`). 연사 파워업은 쿨다운을 `RAPID_FIRE_INTERVAL`로 단축. 확산탄 활성 시 한 번에 3발.

## 3. 특수무기 (`entities/player.rs` 또는 별도) — 확장 가능

- `#[derive(Clone, Copy)] enum SpecialWeaponKind { LaserBeam }` — **확장 지점**(향후 변형 추가).
- 플레이어 상태 `SpecialWeapon { kind: SpecialWeaponKind, charges: u32 }`.
- **발동(X, just_pressed)**: `charges >= 1`이면 `match kind { LaserBeam => fire_laser_beam(...) }` 발사 후 `charges -= 1` + `SfxEvent(Special)`.
  - `LaserBeam`: 정면 방향으로 굵은 레이저빔을 `SpecialBeam { life: Timer(~0.4s) }` 엔티티로 스폰. 빔 경로(우주선 정면으로 뻗는 폭 `BEAM_WIDTH`의 직사각형/두꺼운 선분) 안의 소행성·UFO를 **즉시 파괴**(점수 + `spawn_explosion` + `SfxEvent`). 빔은 gizmos 굵은 선(또는 여러 평행선)으로 렌더, 수명 후 소멸. 화면 흔들림 trauma 가산.
- **빔 판정은 순수 함수**: `segment_circle_hit(beam_origin, beam_dir, beam_len, beam_half_width, target, target_radius) -> bool` (점-선분 거리 기반) — 단위 테스트.
- 확장: 새 특수무기 = enum 변형 + `fire_*` 함수 + `match` 한 팔 + 드롭 가중치 항목.

## 4. 배경 별 (`fx/background.rs`)

- 시작 시(Startup) 무작위 위치에 별 다수(`STAR_COUNT`)를 스폰. `Star { phase: f32, base_brightness: f32 }` (Component) — 게임 상태와 무관하게 항상 존재(`GameplayEntity` 아님).
- `draw_stars` 시스템: 각 별을 gizmo 점으로 그리되 밝기 = `base_brightness * (0.5 + 0.5 * sin(elapsed * TWINKLE_SPEED + phase))` 로 반짝임. 가장 뒤 레이어(다른 렌더보다 먼저 그려지도록 시스템 순서).
- 별은 순환/이동하지 않음(정적).

## 5. 화면 흔들림 (`fx/shake.rs`)

- `#[derive(Resource, Default)] ScreenShake { trauma: f32 }` (0~1로 클램프).
- 이벤트 `#[derive(Event)] ShakeEvent(f32)` 또는 직접 리소스 가산: 피격=`SHAKE_HIT`, 폭발=`SHAKE_EXPLOSION`, 특수무기=`SHAKE_SPECIAL`.
- `apply_screen_shake` 시스템(Update): 카메라(`Camera2d`) Transform에 `trauma²`에 비례한 무작위 오프셋(±`MAX_SHAKE_OFFSET`) 적용, 매 프레임 trauma를 `SHAKE_DECAY * dt` 만큼 감쇠. trauma 0이면 카메라를 원점으로 복귀.
- **감쇠·오프셋 계산은 순수 함수**: `decay_trauma(trauma, dt) -> f32` 단위 테스트.

## 6. 하이퍼스페이스 (`entities/player.rs`)

- `HyperspaceCooldown(Timer)` (플레이어 or 리소스). `H` `just_pressed` 이고 쿨다운 준비 시: 우주선을 화면 내 무작위 위치로 순간이동(`Transform` 갱신) + 속도 0 리셋(선택) + `SfxEvent(Hyperspace)` + 쿨다운 시작.
- 무적 없음 — 소행성 위에 뜨면 다음 프레임 `player_damage`로 즉사(의도된 리스크).
- **좌표는 순수 함수**: `random_hyperspace_position(half) -> Vec2` (테스트 시 결정 어려우니 범위 검증).

## HUD 추가 (`ui.rs`)

- 특수무기: 현재 장착 종류 + 충전 수(예: `Laser x2`).
- 활성 파워업(실드/연사/확산탄) 잔여 표시(간단히 아이콘/문자).

## 테스트 전략

순수 로직 우선 TDD:
- `segment_circle_hit`(빔 판정), `decay_trauma`(흔들림 감쇠), `pick_powerup_kind`(가중 선택 분포는 결정적 시드 어려우니 반환값이 유효 종류인지), `random_hyperspace_position`(범위), 발사 쿨다운 로직.
시스템 테스트: 파워업 드롭/수집/효과 적용, 특수무기 발동·빔 파괴, 실드 무피해, 이벤트 발행/소비, HUD 갱신.
사운드·별·흔들림·빔 시각은 `cargo run` 수동 확인.

## 완료 기준 (DoD)

- `src/`가 core/entities/systems/fx 폴더로 재구성되고 전체 테스트 통과.
- 주요 이벤트에 효과음이 난다.
- 적 파괴 시 파워업이 드롭되고, 5종 효과가 각각 동작한다(실드 무적/연사/확산탄/추가목숨/특수무기 충전).
- X로 특수무기(레이저빔)가 발동돼 경로상 적을 쓸어버리고 충전을 소모한다.
- 배경에 반짝이는 별밭이 보인다.
- 피격/폭발/특수무기 시 화면이 흔들린다.
- H로 하이퍼스페이스 순간이동(리스크 포함)이 된다.
- HUD에 특수무기 충전·활성 파워업이 표시된다.
- 순수 로직 단위 테스트가 통과한다.

## 범위 밖(향후 후보)

추가 특수무기 종류(폭탄·유도미사일·시간정지 등), BGM, 파워업 종류 추가, 보스, 화면 전환 연출.
