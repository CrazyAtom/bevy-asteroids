# 타이틀 화면 + 일시정지 메뉴 설계

> 미뤄뒀던 기능 재제안 중 사용자가 선택한 첫 항목. 범위: **타이틀 + 일시정지만**(난이도 선택·설정 화면·BGM 볼륨은 각각 별도 스펙).

**목표:** 게임에 타이틀 화면과 일시정지 메뉴를 추가한다. `GameState`를 화면 흐름(Title/Playing/GameOver)과 진행/정지(RunPhase) 두 축으로 분리해, 일시정지가 게임을 리셋하지 않고 순수하게 시뮬레이션만 멈추게 한다.

**핵심 원칙:** 상태 축 분리 → "일시정지 ≠ 게임 종료"를 타입 수준에서 보장. 재개 시 실수 리셋(대표적 함정) 원천 차단.

---

## 1. 배경 — 현재 구조

- `GameState { Playing(default), GameOver }`. 게임이 곧바로 `Playing`으로 부팅된다.
- 모든 판-내 콘텐츠(플레이어·소행성·UFO·총알·파워업·보스·파티클·HUD·보스바·배너·특수무기)는 `GameplayEntity` 마커를 단다.
- 생명주기가 이미 깔끔하게 분리돼 있다:
  - `OnEnter(GameState::Playing)`: `reset_game`(점수·목숨·진행도·UFO 타이머 리셋) + `spawn_player` + 스테이지 초기 소행성 스폰 + `spawn_hud`·`spawn_boss_bar`·`reset_last_stage`.
  - `OnExit(GameState::Playing)`: `despawn_gameplay_entities`(모든 `GameplayEntity` 일괄 삭제).
- 배경·안개·블랙홀 스프라이트·카메라·스프라이트 핸들은 `Startup`에 1회 스폰되고 `GameplayEntity`가 아니라 판이 바뀌어도 유지된다.
- 게임플레이 틱 시스템 다수가 `run_if(in_state(GameState::Playing))`으로 게이팅돼 있다.
- 카메라는 `Hdr` + `Bloom` + `Tonemapping::None`(특수무기 발광용). 색값 > 1.0 스프라이트/텍스트가 발광한다.

**검증된 Bevy 0.19 사실:**
- `SubStates` 파생: `#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]` + `#[source(GameState = GameState::Playing)]`. 등록은 `app.add_sub_state::<RunPhase>()`. 소스 상태가 조건을 만족할 때만 존재하고, 벗어나면 자동 제거된다. 진입 시 `#[default]` 값으로 생성된다.
- **항등 전이는 무시된다**(`bevy_state-0.19.0/src/state/transitions.rs:250`): `if transition.entered == transition.exited && !allow_same_state_transitions { return }`. 즉 `NextState`를 현재와 **같은 값**으로 설정하면 `OnEnter`/`OnExit`가 발화하지 않는다. → 일시정지 중(=Playing) 재시작은 반드시 `Playing`을 벗어났다 재진입해야 한다.

---

## 2. 상태 모델

```rust
// core/state.rs
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Title,        // 부팅 시 시작점 (기존 default=Playing에서 변경)
    Playing,
    Restarting,   // 1프레임 바운스 상태. OnEnter에서 즉시 Playing으로. UI·부수효과 없음
    GameOver,
}

#[derive(SubStates, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum RunPhase {
    #[default]
    Running,      // Playing 진입 시 자동으로 이 값
    Paused,
}
```

등록: `app.init_state::<GameState>().add_sub_state::<RunPhase>()`.

**생명주기 변경 없음:** `OnEnter/OnExit(GameState::Playing)`의 리셋·스폰·삭제 로직은 그대로 둔다. 일시정지는 `GameState`를 `Playing`으로 유지하므로 이들이 발화하지 않는다.

**바운스:** `OnEnter(GameState::Restarting)` → `NextState<GameState>::set(Playing)` 한 줄. 시각적 UI·리소스 변경 없음.

---

## 3. 내비게이션 그래프

```
        ┌────────────────────────── Title ──────────────────────────┐
        │  [START] → Playing        [QUIT] → AppExit                 │
        └──────────────┬────────────────────────────────────────────┘
                       │ StartGame
                       ▼
      Playing (RunPhase::Running) ──ESC──► Playing (RunPhase::Paused)
        │                                    │  [RESUME]  → Running
        │ 사망(collision)                    │  [RESTART] → Restarting ─┐
        ▼                                    │  [QUIT]    → Title       │
      GameOver                               └──────────────────────────┘
        │  [RESTART] → Restarting ──────────────────────────┐
        │  [QUIT]    → Title                                │
        └───────────────────────────────────────────────────┘
                                    Restarting ──(OnEnter)──► Playing
```

**단일 `MenuAction` 디스패치:**

| 액션 | 처리 | 소스 화면 |
|---|---|---|
| `StartGame` | `NextState<GameState>::set(Playing)` | Title |
| `Resume` | `NextState<RunPhase>::set(Running)` | Paused |
| `Restart` | `NextState<GameState>::set(Restarting)` | Paused, GameOver |
| `QuitToTitle` | `NextState<GameState>::set(Title)` | Paused, GameOver |
| `QuitApp` | `AppExit` 메시지 발행 | Title |

`Resume`만 `RunPhase`를 건드리고 나머지는 `GameState`를 건드린다. 재시작은 어느 화면에서 눌러도 `Restarting`으로 통일 → `Playing` 완전 재진입(teardown+rebuild).

---

## 4. 일시정지 정지 의미론

**게이팅 교체:** 현재 `run_if(in_state(GameState::Playing))`으로 게이팅된 게임플레이 틱 시스템 전부를 `run_if(in_state(RunPhase::Running))`으로 바꾼다. 대상(플러그인별):

- `player`: `player_input, update_flame, shield_tick, update_shield_sprite, tick_fire_mods, hyperspace`(Update), `apply_ship_damping`(FixedUpdate), `debug_fill_hud`(debug).
- `movement`: 속도 적분/이동 시스템.
- `collision`: 충돌 시스템 세트.
- `stage`: 웨이브/스폰 진행 시스템.
- `boss`: 보스 AI/공격 시스템.
- `ufo`: UFO 스폰/이동/발사.
- `powerup`: 드롭/수명/수집.
- `bullet`: 총알 수명/이동.
- `special_weapon`: 빔·노바·미사일·충격파·실드버스트 틱, 유도 조향.
- `effects`: 파티클 수명/페이드.
- `animation`: 스프라이트 애니메이션 진행.
- `background`: 테마 배경 업데이트.
- `fog`: 안개 업데이트(정지 시 얼림).

**정지 중 유지되는 것:** 렌더링, 모든 엔티티(삭제 안 됨), HUD/보스바 표시 시스템은 리소스만 읽으므로 값이 얼어붙은 채 표시(계속 돌아도 무해). 카메라 흔들림(shake)은 정지 시 멈추는 게 자연스러우므로 `RunPhase::Running` 게이팅 대상에 포함.

**입력:**
- `pause_input`(게이팅 `in_state(RunPhase::Running)`): `ESC` → `NextState<RunPhase>::set(Paused)`.
- 일시정지 메뉴(게이팅 `in_state(RunPhase::Paused)`): `ESC` → `Resume`(빠른 해제), ↑↓/Enter → 메뉴 조작.

**시간:** `Res<Time>`(실시간)은 계속 흐르지만 게이팅된 시스템이 안 돌아 타이머가 틱하지 않는다. 별도 `Time` 정지 불필요.

---

## 5. 메뉴 UI (재사용 메커니즘)

기존 미니멀 룩 유지: 검은 배경, 흰 텍스트, 중앙 정렬 세로 배치. 세 화면(Title/Pause/GameOver)이 동일한 메뉴 메커니즘을 공유한다.

```rust
// ui/menu.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction { StartGame, Resume, Restart, QuitToTitle, QuitApp }

#[derive(Component)]
pub struct MenuItem { pub index: usize, pub action: MenuAction }

#[derive(Resource, Default)]
pub struct MenuSelection { pub index: usize, pub count: usize }

/// 순수 함수(단위 테스트 대상): 순환 인덱스 이동. delta는 -1(위)/+1(아래).
pub fn wrap_index(cur: usize, count: usize, delta: i32) -> usize;
```

시스템:
- `menu_navigation`: ↑(ArrowUp)→`wrap_index(.., -1)`, ↓(ArrowDown)→`wrap_index(.., +1)`로 `MenuSelection.index` 갱신. `Enter`(또는 Space) 시, `index`와 일치하는 `MenuItem`의 `action`을 디스패치(§3 표).
- `highlight_menu`: `MenuSelection.index`와 일치하는 항목 텍스트를 **HDR 금빛**(`Color::srgb(4.0, 3.2, 1.2)` 등 색값 > 1.0)으로 발광 + `▶ ` 프리픽스, 나머지는 흐린 흰색(`Color::srgb(0.5,0.5,0.5)`), 프리픽스 없음.

**활성 조건(run_if):** `in_state(GameState::Title).or(in_state(GameState::GameOver)).or(in_state(RunPhase::Paused))`.

**디스패처 시스템 요구 파라미터:** `Res<ButtonInput<KeyCode>>`, `ResMut<MenuSelection>`, `Query<&MenuItem>`, `ResMut<NextState<GameState>>`, `ResMut<NextState<RunPhase>>`, `MessageWriter<AppExit>`.

> `NextState<RunPhase>`는 서브상태라도 `add_sub_state`가 빌드 시점에 `init_resource`로 등록하므로(검증: `bevy_state-0.19.0/src/app.rs:192`) Playing 밖(타이틀/게임오버)에서도 항상 존재한다. 따라서 디스패처가 `ResMut<NextState<RunPhase>>`를 직접 받아도 패닉하지 않는다(`Option` 불필요). Playing 밖에서 이 값을 설정하는 일은 없다(Resume는 Paused에서만).

각 화면은 진입 시 자기 메뉴 항목(Text + `MenuItem`)들을 스폰하고 `MenuSelection { index: 0, count: N }`으로 초기화, 이탈 시 자기 UI를 despawn한다. 화면별 UI는 각자의 마커 컴포넌트로 구분해 정리한다(예: `TitleUi`, `PauseUi`).

---

## 6. 화면 내용

**Title** (`ui/title.rs`, `OnEnter(GameState::Title)` 스폰 / `OnExit` 정리):
- 대형 타이틀 텍스트 `ASTEROIDS`(상단 중앙).
- `HIGH SCORE: {n}`(`Persistent<HighScore>` 읽기).
- 메뉴: `[START]`(`StartGame`), `[QUIT]`(`QuitApp`).
- 하단 조작 힌트 한 줄(예: `↑↓ SELECT   ENTER CONFIRM`).

**Paused** (`ui/pause.rs`, `OnEnter(RunPhase::Paused)` 스폰 / `OnExit` 정리):
- 전체 화면 반투명 어둠 오버레이(예: `Color::srgba(0,0,0,0.6)` 스프라이트, 게임 위·메뉴 아래).
- `PAUSED` 텍스트.
- 메뉴: `[RESUME]`(`Resume`), `[RESTART]`(`Restart`), `[QUIT TO TITLE]`(`QuitToTitle`).

**GameOver** (`ui/game_over.rs` 수정, 기존 `OnEnter/OnExit(GameState::GameOver)` 유지):
- 기존 점수/최고점수 표시 유지.
- 기존 "아무 키 → 재시작"(`restart_input`) 제거.
- 메뉴 추가: `[RESTART]`(`Restart`), `[QUIT TO TITLE]`(`QuitToTitle`).

---

## 7. 파일 구조 (`폴더=기능, 파일=관심사`)

| 파일 | 변경 | 책임 |
|---|---|---|
| `core/state.rs` | 수정 | `GameState`에 `Title`/`Restarting` 추가·default를 Title로, `RunPhase` 서브상태 정의, `add_sub_state`, `OnEnter(Restarting)` 바운스 시스템 |
| `ui/menu.rs` | 신규 | `MenuAction`·`MenuItem`·`MenuSelection`·`wrap_index`·`menu_navigation`·`highlight_menu`(공유) |
| `ui/title.rs` | 신규 | 타이틀 화면 스폰/정리 + `TitleUi` 마커 |
| `ui/pause.rs` | 신규 | `pause_input`, 어둠 오버레이·메뉴 스폰/정리 + `PauseUi` 마커 |
| `ui/game_over.rs` | 수정 | 메뉴 방식 전환(기존 `restart_input` 제거, `MenuItem` 스폰) |
| `ui.rs` | 수정 | 신규 서브모듈 등록, 메뉴 시스템 스케줄 배선 |
| 게임플레이 플러그인 13종 | 수정 | `run_if` 조건 `in_state(Playing)` → `in_state(RunPhase::Running)` 교체 |

`RunPhase`는 `core/state.rs`에 함께 둔다(상태 정의 응집). 파일이 과도하게 커지면 이후 분리.

---

## 8. 에러/엣지 케이스

- **부팅 시 상태**: default가 Title이므로 게임은 타이틀에서 시작. `Startup` 스폰물(배경 스타필드)이 타이틀 뒤에 보인다(의도된 배경).
- **재개가 리셋하지 않음**: 재개는 `RunPhase` 전환만 하므로 `GameState`는 Playing 유지 → `OnEnter/OnExit(Playing)` 미발화 → 점수·목숨·엔티티 보존.
- **재시작 통일**: Paused·GameOver 모두 `Restarting` 경유 → `OnExit(이전)` 정리 + `OnEnter(Playing)` 재구성. Paused에서 눌러도 `Playing→Restarting`으로 `OnExit(Playing)`가 발화해 보드가 정리됨.
- **일시정지 중 사망 불가**: 시뮬레이션이 멈춰 충돌이 없으므로 정지 중 GameOver 전이 없음.
- **RunPhase 잔존 방지**: `Playing`을 벗어나면 서브상태가 자동 제거되므로 "타이틀인데 Paused" 같은 모순 상태 불가.
- **AppExit(WASM)**: 웹 배포 시 `AppExit`는 무의미하나 데스크톱에선 정상 종료. WASM 배포는 별도 스펙에서 Title의 QUIT 처리 재검토.

---

## 9. 테스트 전략

**순수 단위(로직):**
- `wrap_index`: 끝에서 아래로 → 0 순환, 0에서 위로 → 끝 순환, count=1일 때 제자리.

**통합(App 기반, 기존 `RunSystemOnce`/상태 전이 패턴 재사용):**
1. **기본 상태 = Title**: 앱 초기화 후 `State<GameState>`가 `Title`.
2. **재개가 리셋하지 않음(함정 회귀 방지)**: `Score=50` 설정, Playing/Paused 진입 후 Resume(→Running) 전이 → `Score`가 여전히 50이고 `GameplayEntity`가 삭제되지 않음.
3. **재시작이 리셋함**: `Score=50` → `Restart`(Restarting→Playing 재진입) → `Score=0`.
4. **타이틀 복귀가 보드 정리**: Playing에서 `GameplayEntity` 스폰 → `QuitToTitle`(→Title) → `OnExit(Playing)`로 삭제됨.
5. **메뉴 내비게이션**: `MenuSelection` 설정 + ↓ 입력 → index 증가(순환), Enter → 해당 액션의 상태 전이 발생.

---

## 10. 범위 밖 (별도 스펙)

- 난이도 선택 화면.
- 설정 화면(키 리매핑 등).
- BGM 및 볼륨/음소거 UI(→ BGM 스펙에서 일시정지 메뉴에 부착).
- 웹(WASM) 배포 및 그에 따른 QUIT 동작 조정.
- 마우스 조작(현 설계는 키보드 전용).
