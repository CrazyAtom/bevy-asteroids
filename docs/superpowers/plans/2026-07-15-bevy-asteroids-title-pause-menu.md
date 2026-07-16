# 타이틀 화면 + 일시정지 메뉴 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 게임에 타이틀 화면과 ESC 일시정지 메뉴를 추가하되, 일시정지가 게임을 리셋하지 않게 한다.

**Architecture:** `GameState`(Title/Playing/Restarting/GameOver)와 `RunPhase`(Running/Paused, Playing의 SubState) 두 축으로 상태를 분리한다. 게임플레이 틱 시스템을 `RunPhase::Running`으로 게이팅해 일시정지 시 시뮬레이션만 멈추고 엔티티·렌더링은 유지한다. 재시작은 항등 전이가 무시되는 Bevy 특성상 1프레임 `Restarting` 바운스 상태로 `Playing`을 재진입해 처리한다. 타이틀·일시정지·게임오버 세 화면은 공유 방향키 메뉴 엔진(`ui/menu.rs`)을 쓴다.

**Tech Stack:** Rust, Bevy 0.19(SubStates, UI Text/Node), bevy-persistent.

## Global Constraints

- Bevy `0.19`. SubStates 문법: `#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]` + `#[source(GameState = GameState::Playing)]`, 등록 `app.add_sub_state::<RunPhase>()`.
- **항등 전이는 무시된다**(`bevy_state-0.19.0/src/state/transitions.rs:250`): `NextState`를 현재와 같은 값으로 설정해도 `OnEnter/OnExit` 미발화. → Playing 중 재시작은 `Restarting` 경유 필수.
- `NextState<RunPhase>`는 `add_sub_state`가 빌드 시 `init_resource`로 등록(`app.rs:192`)하므로 Playing 밖에서도 항상 존재 → 시스템이 `ResMut<NextState<RunPhase>>`를 직접 받아도 안전(`Option` 불필요).
- `in_state`는 상태 리소스 부재 시 `false` 반환(`condition.rs:104`) → 타이틀에서 `in_state(RunPhase::Paused)` 안전.
- run-condition 결합자 `.or()` 사용 가능(`condition.rs:537`).
- `AppExit`는 Message → `MessageWriter<AppExit>` + `exit.write(AppExit::Success)`(import `bevy::app::AppExit`). 코드베이스 기존 패턴은 `MessageWriter<SfxEvent>`.
- **기존 `OnEnter/OnExit(GameState::Playing)` 생명주기(리셋·스폰·삭제)는 변경 금지.** 일시정지는 Playing을 유지하므로 이들을 건드리지 않는다.
- 일시정지 키 = `ESC`(현재 미사용, 충돌 없음). 메뉴 조작 = `ArrowUp`/`ArrowDown` + `Enter`.
- UI 텍스트 패턴(기존 `ui/banner.rs`·`game_over.rs`와 동일): `Text::new(..)`, `TextFont { font_size: FontSize::Px(..), ..default() }`(import `bevy::text::FontSize`), `TextColor(..)`, `Node { position_type: PositionType::Absolute, top: Val::Percent(..), left: Val::Percent(..), ..default() }`.
- 바이너리 크레이트라 `cargo clippy -- -D warnings`에서 미사용 pub 항목/필드도 경고 → 각 항목은 이를 소비하는 태스크에 둔다.
- 커밋 메시지는 한국어, 끝에 `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.
- 검증 명령: `cargo test`(전체), `cargo clippy -- -D warnings`(린트).

---

## File Structure

| 파일 | 태스크 | 책임 |
|---|---|---|
| `src/core/state.rs` | 1, 3 | `GameState`에 `Title`/`Restarting` 추가, `RunPhase` SubState, `add_sub_state`, `OnEnter(Restarting)` 바운스. (3에서 default를 Title로) |
| 게임플레이 13파일 | 1 | `run_if(in_state(GameState::Playing))` → `run_if(in_state(RunPhase::Running))` |
| `src/ui/menu.rs` | 2 | `MenuAction`·`MenuItem`·`MenuSelection`·`wrap_index`·`menu_move`·`menu_activate`·`highlight_menu` |
| `src/ui.rs` | 1,2,3,4,5 | 서브모듈 등록·시스템 배선(게이팅 교체, 메뉴 엔진, 화면 스케줄) |
| `src/ui/title.rs` | 3 | 타이틀 화면 스폰/정리 + `TitleUi` 마커 |
| `src/ui/pause.rs` | 4 | `pause_input`(ESC)·`resume_on_esc`·오버레이·메뉴 스폰/정리 + `PauseUi` 마커 |
| `src/ui/game_over.rs` | 5 | 메뉴 방식 전환(기존 `restart_input`/R 제거) |

---

## Task 1: 상태 모델 — RunPhase SubState + 게이팅 교체 + Restarting 바운스

**Files:**
- Modify: `src/core/state.rs`
- Modify(게이팅): `src/ui.rs:34`, `src/fx/background.rs:16`, `src/fx/effects.rs:20`, `src/fx/animation.rs:49`, `src/systems/collision.rs:32`, `src/systems/stage.rs:179`, `src/entities/boss.rs:169`, `src/entities/ufo.rs:57`, `src/entities/black_hole.rs:42-43`, `src/entities/bullet.rs:30`, `src/entities/powerup.rs:37`, `src/entities/special_weapon.rs:77`, `src/entities/player.rs:69,73,77`
- Test: `src/core/state.rs`(tests 모듈)

**Interfaces:**
- Produces:
  - `pub enum GameState { #[default] Playing, Title, Restarting, GameOver }` (이 태스크에서 default는 **Playing 유지**; Task 3에서 Title로 변경)
  - `pub enum RunPhase { #[default] Running, Paused }` (SubState of `GameState::Playing`)
  - `GameStatePlugin`이 `add_sub_state::<RunPhase>()` + `OnEnter(GameState::Restarting)` → `bounce_to_playing` 등록.

- [ ] **Step 1: 실패하는 테스트 작성** — `src/core/state.rs`의 `#[cfg(test)] mod tests`에 추가:

```rust
    use crate::entities::ufo::UfoSpawnTimer;

    /// 테스트용 최소 앱: 상태 기계 + GameStatePlugin + reset_game가 요구하는 리소스.
    fn state_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(GameStatePlugin);
        app.insert_resource(UfoSpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));
        // 명시적으로 Playing 진입(기본 상태에 의존하지 않음 → Task 3의 default 변경에도 견고)
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        app
    }

    #[test]
    fn resume_does_not_reset_score() {
        let mut app = state_test_app();
        app.world_mut().resource_mut::<Score>().0 = 50;
        app.world_mut().resource_mut::<NextState<RunPhase>>().set(RunPhase::Paused);
        app.update();
        app.world_mut().resource_mut::<NextState<RunPhase>>().set(RunPhase::Running);
        app.update();
        assert_eq!(app.world().resource::<Score>().0, 50, "재개는 점수를 리셋하면 안 된다");
    }

    #[test]
    fn restart_via_bounce_resets_score() {
        let mut app = state_test_app();
        app.world_mut().resource_mut::<Score>().0 = 50;
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Restarting);
        app.update(); // Playing→Restarting: OnExit(Playing), OnEnter(Restarting)→NextState(Playing)
        app.update(); // Restarting→Playing: OnEnter(Playing) reset_game → 0
        assert_eq!(app.world().resource::<Score>().0, 0, "재시작은 점수를 0으로 리셋해야 한다");
        assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::Playing);
    }
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --lib state::tests 2>&1 | tail -20`
Expected: 컴파일 실패(`RunPhase` 미정의, `GameState::Restarting` 없음).

- [ ] **Step 3: 상태 모델 구현** — `src/core/state.rs` 편집.

`GameState` enum 교체(변형 추가, default는 Playing 유지):

```rust
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Title,
    Restarting,
    GameOver,
}

/// Playing 하위 진행/정지 축. Playing에 진입하면 자동으로 Running으로 생성된다.
#[derive(SubStates, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum RunPhase {
    #[default]
    Running,
    Paused,
}
```

`GameStatePlugin::build`에 SubState 등록 + 바운스 시스템 추가(기존 `.add_systems` 체인에 이어붙임):

```rust
        app.init_state::<GameState>()
            .add_sub_state::<RunPhase>()
            .insert_resource(Score(0))
            .insert_resource(Lives(STARTING_LIVES))
            .insert_resource(crate::systems::stage::new_progression())
            .add_systems(OnEnter(GameState::Playing), reset_game)
            .add_systems(OnExit(GameState::Playing), despawn_gameplay_entities)
            .add_systems(OnEnter(GameState::Restarting), bounce_to_playing)
            .add_systems(OnEnter(GameState::GameOver), (save_high_score, play_game_over_sfx));
```

바운스 시스템 추가(파일 하단 아무 위치, 예: `reset_game` 근처):

```rust
/// Restarting은 1프레임 바운스 상태다. 진입 즉시 Playing으로 넘겨
/// OnExit(Playing)→OnEnter(Playing)의 전면 teardown+rebuild를 유발한다.
fn bounce_to_playing(mut next: ResMut<NextState<GameState>>) {
    next.set(GameState::Playing);
}
```

- [ ] **Step 4: 게임플레이 게이팅 교체** — 아래 13개 파일에서 `in_state(GameState::Playing)` → `in_state(RunPhase::Running)`으로 바꾸고, 각 파일의 `use crate::core::state::{...}` 에 `RunPhase`를 추가한다(이미 `GameState`를 import 중이면 목록에 `RunPhase` 추가).

교체 대상(파일:라인 — 현재 문자열 → 새 문자열):
- `src/ui.rs:34` `in_state(GameState::Playing)` → `in_state(RunPhase::Running)` (HUD/보스바/배너 Update 블록)
- `src/fx/background.rs:16` 동일 교체
- `src/fx/effects.rs:20` 동일 교체
- `src/fx/animation.rs:49` `in_state(crate::core::state::GameState::Playing)` → `in_state(crate::core::state::RunPhase::Running)` (import 없이 완전경로 사용 중이면 그대로 완전경로 교체)
- `src/systems/collision.rs:32` 동일 교체
- `src/systems/stage.rs:179` 동일 교체
- `src/entities/boss.rs:169` 동일 교체
- `src/entities/ufo.rs:57` 동일 교체
- `src/entities/black_hole.rs:42`, `:43` 두 곳 동일 교체
- `src/entities/bullet.rs:30` 동일 교체
- `src/entities/powerup.rs:37` 동일 교체
- `src/entities/special_weapon.rs:77` 동일 교체
- `src/entities/player.rs:69`, `:73`, `:77` 세 곳 동일 교체

주의: `debug_fill_hud`(player.rs:77)도 `RunPhase::Running`으로 게이팅(일시정지 중 디버그 채우기 방지).

- [ ] **Step 5: 테스트·린트 통과 확인**

Run: `cargo test 2>&1 | tail -20 && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: 전체 통과(신규 2 테스트 포함), clippy 경고 0. 기존 테스트도 유지(게이팅은 run_system_once 테스트에 영향 없음, default는 Playing 유지).

- [ ] **Step 6: 수동 확인(로컬)**

Run: `cargo run` — 게임이 여전히 Playing으로 부팅되고 정상 플레이되는지 확인(아직 타이틀/메뉴 없음). ESC는 아직 미배선.

- [ ] **Step 7: 커밋**

```bash
git add src/core/state.rs src/ui.rs src/fx src/systems src/entities
git commit -m "feat: RunPhase 서브상태 + 게임플레이 게이팅 교체 + Restarting 바운스

$(printf '일시정지를 위한 상태 뼈대. GameState에 Title/Restarting 추가(기본은 아직\nPlaying), Playing 하위 RunPhase{Running,Paused} 서브상태 도입. 게임플레이\n틱 13파일을 in_state(RunPhase::Running)으로 게이팅해 정지 시 시뮬레이션만\n멈춘다. 재시작용 Restarting 1프레임 바운스 추가. 재개-무리셋/재시작-리셋\n회귀 테스트 2종.')

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 2: 공유 메뉴 엔진 — ui/menu.rs

**Files:**
- Create: `src/ui/menu.rs`
- Modify: `src/ui.rs`(모듈 등록 + 리소스 + 시스템 배선)
- Test: `src/ui/menu.rs`(tests 모듈)

**Interfaces:**
- Consumes: `GameState`, `RunPhase`(Task 1).
- Produces:
  - `pub enum MenuAction { StartGame, Resume, Restart, QuitToTitle, QuitApp }`
  - `pub struct MenuItem { pub index: usize, pub action: MenuAction, pub label: &'static str }`(Component)
  - `pub struct MenuSelection { pub index: usize, pub count: usize }`(Resource, Default)
  - `pub fn wrap_index(cur: usize, count: usize, delta: i32) -> usize`
  - 시스템: `menu_move`, `menu_activate`, `highlight_menu`(전부 `pub(crate)` 또는 `pub`), OR 조건으로 게이팅되어 UiPlugin에 등록됨.

- [ ] **Step 1: 실패하는 단위 테스트 작성** — `src/ui/menu.rs` 생성, 하단에:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_index_cycles() {
        assert_eq!(wrap_index(0, 3, -1), 2, "0에서 위로 → 마지막");
        assert_eq!(wrap_index(2, 3, 1), 0, "마지막에서 아래로 → 0");
        assert_eq!(wrap_index(1, 3, 1), 2);
        assert_eq!(wrap_index(1, 3, -1), 0);
        assert_eq!(wrap_index(0, 1, 1), 0, "항목 1개는 제자리");
        assert_eq!(wrap_index(0, 0, 1), 0, "빈 메뉴는 0");
    }

    #[test]
    fn menu_move_updates_selection() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = App::new();
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::ArrowDown);
        app.insert_resource(input);
        app.insert_resource(MenuSelection { index: 0, count: 3 });
        app.world_mut().run_system_once(menu_move).unwrap();
        assert_eq!(app.world().resource::<MenuSelection>().index, 1);
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --lib ui::menu 2>&1 | tail -20`
Expected: 컴파일 실패(`wrap_index`/`menu_move`/`MenuSelection` 미정의).

- [ ] **Step 3: 메뉴 엔진 구현** — `src/ui/menu.rs` 상단:

```rust
//! 타이틀·일시정지·게임오버가 공유하는 방향키 메뉴 엔진.
//! 화면은 MenuItem(Text) 엔티티들을 스폰하고 MenuSelection을 초기화한다.
//! 이 모듈은 이동(menu_move)·확정(menu_activate)·강조(highlight_menu)를 담당한다.

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::core::state::{GameState, RunPhase};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    StartGame,
    Resume,
    Restart,
    QuitToTitle,
    QuitApp,
}

/// 메뉴 한 항목(Text 엔티티에 부착). 화면 마커와 함께 스폰된다.
#[derive(Component)]
pub struct MenuItem {
    pub index: usize,
    pub action: MenuAction,
    pub label: &'static str,
}

/// 현재 강조 중인 항목과 총 개수. 화면 진입 시 초기화된다(단일 전역 리소스).
#[derive(Resource, Default)]
pub struct MenuSelection {
    pub index: usize,
    pub count: usize,
}

/// 순환 인덱스 이동. delta는 -1(위)/+1(아래). count가 0이면 0.
pub fn wrap_index(cur: usize, count: usize, delta: i32) -> usize {
    if count == 0 {
        return 0;
    }
    let n = count as i32;
    (((cur as i32 + delta) % n + n) % n) as usize
}

/// ↑↓로 선택 인덱스 이동(순환).
pub fn menu_move(keys: Res<ButtonInput<KeyCode>>, mut selection: ResMut<MenuSelection>) {
    if selection.count == 0 {
        return;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        selection.index = wrap_index(selection.index, selection.count, -1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        selection.index = wrap_index(selection.index, selection.count, 1);
    }
}

/// Enter로 선택 항목의 액션을 디스패치.
pub fn menu_activate(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<MenuSelection>,
    items: Query<&MenuItem>,
    mut next_game: ResMut<NextState<GameState>>,
    mut next_phase: ResMut<NextState<RunPhase>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !keys.just_pressed(KeyCode::Enter) {
        return;
    }
    let Some(action) = items
        .iter()
        .find(|i| i.index == selection.index)
        .map(|i| i.action)
    else {
        return;
    };
    match action {
        MenuAction::StartGame => next_game.set(GameState::Playing),
        MenuAction::Resume => next_phase.set(RunPhase::Running),
        MenuAction::Restart => next_game.set(GameState::Restarting),
        MenuAction::QuitToTitle => next_game.set(GameState::Title),
        MenuAction::QuitApp => {
            exit.write(AppExit::Success);
        }
    }
}

/// 선택 항목은 밝은 금빛 + "▶ " 프리픽스, 나머지는 흐리게.
/// 색값 > 1.0은 HDR+Bloom(카메라에 설정됨) 지원 시 발광한다.
pub fn highlight_menu(
    selection: Res<MenuSelection>,
    mut items: Query<(&MenuItem, &mut Text, &mut TextColor)>,
) {
    for (item, mut text, mut color) in &mut items {
        if item.index == selection.index {
            *text = Text::new(format!("\u{25B6} {}", item.label));
            color.0 = Color::srgb(1.5, 1.2, 0.3);
        } else {
            *text = Text::new(item.label.to_string());
            color.0 = Color::srgb(0.55, 0.55, 0.55);
        }
    }
}
```

- [ ] **Step 4: UiPlugin에 배선** — `src/ui.rs` 편집.

모듈 선언 추가(다른 `mod` 옆):

```rust
mod menu;
```

`use` 추가:

```rust
use crate::core::state::{GameState, RunPhase};
```
(기존 `use crate::core::state::GameState;`가 있으면 `RunPhase` 추가로 합침.)

`UiPlugin::build`에 리소스 + 메뉴 시스템 등록(기존 체인에 이어붙임):

```rust
            .init_resource::<menu::MenuSelection>()
            .add_systems(
                Update,
                (menu::menu_move, menu::menu_activate, menu::highlight_menu).run_if(
                    in_state(GameState::Title)
                        .or(in_state(GameState::GameOver))
                        .or(in_state(RunPhase::Paused)),
                ),
            )
```

- [ ] **Step 5: 테스트·린트 통과 확인**

Run: `cargo test 2>&1 | tail -15 && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: 통과. 메뉴 시스템은 등록됐지만 아직 MenuItem 엔티티가 없어 무동작(미사용 경고 없음 — 전부 등록/참조됨).

- [ ] **Step 6: 커밋**

```bash
git add src/ui/menu.rs src/ui.rs
git commit -m "feat: 공유 방향키 메뉴 엔진(ui/menu) 추가

MenuAction·MenuItem·MenuSelection·wrap_index와 이동/확정/강조 시스템.
세 화면(타이틀·일시정지·게임오버)이 공유하며 OR 조건으로 게이팅해 UiPlugin에
등록. 선택 항목은 HDR 금빛(Bloom) + ▶ 프리픽스. wrap_index 순환·menu_move
회귀 테스트 포함. 아직 화면은 없음(다음 태스크).

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 3: 타이틀 화면 + 기본 상태 전환

**Files:**
- Create: `src/ui/title.rs`
- Modify: `src/ui.rs`(모듈 등록 + OnEnter/OnExit(Title) 스케줄)
- Modify: `src/core/state.rs`(default를 `Title`로)
- Test: `src/ui/title.rs`(tests 모듈)

**Interfaces:**
- Consumes: `MenuItem`, `MenuAction`, `MenuSelection`(Task 2); `GameState`(Task 1); `Persistent<HighScore>`.
- Produces: `TitleUi` 마커, `spawn_title`, `despawn_title`.

- [ ] **Step 1: 실패하는 테스트 작성** — `src/ui/title.rs` 생성, 하단:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::GameStatePlugin;

    #[test]
    fn boots_to_title() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(GameStatePlugin);
        app.update();
        assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::Title);
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --lib ui::title 2>&1 | tail -15`
Expected: 실패(`boots_to_title` — 현재 default가 Playing) + 컴파일 실패(`title` 모듈/함수 미정의).

- [ ] **Step 3: 타이틀 화면 구현** — `src/ui/title.rs`:

```rust
//! 타이틀 화면: 게임명·최고점수·메뉴(START/QUIT).

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;

use crate::core::state::HighScore;
use crate::ui::menu::{MenuAction, MenuItem, MenuSelection};

#[derive(Component)]
pub(super) struct TitleUi;

pub(super) fn spawn_title(
    mut commands: Commands,
    high: Res<Persistent<HighScore>>,
    mut selection: ResMut<MenuSelection>,
) {
    // 타이틀
    commands.spawn((
        TitleUi,
        Text::new("ASTEROIDS"),
        TextFont { font_size: FontSize::Px(72.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(22.0),
            left: Val::Percent(32.0),
            ..default()
        },
    ));
    // 최고점수
    commands.spawn((
        TitleUi,
        Text::new(format!("HIGH SCORE: {}", high.0)),
        TextFont { font_size: FontSize::Px(28.0), ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(38.0),
            ..default()
        },
    ));
    // 메뉴 항목 2개
    let items = [(MenuAction::StartGame, "START"), (MenuAction::QuitApp, "QUIT")];
    for (i, (action, label)) in items.iter().enumerate() {
        commands.spawn((
            TitleUi,
            MenuItem { index: i, action: *action, label },
            Text::new(label.to_string()),
            TextFont { font_size: FontSize::Px(36.0), ..default() },
            TextColor(Color::srgb(0.55, 0.55, 0.55)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(52.0 + i as f32 * 8.0),
                left: Val::Percent(45.0),
                ..default()
            },
        ));
    }
    // 조작 힌트
    commands.spawn((
        TitleUi,
        Text::new("\u{2191}\u{2193} SELECT   ENTER CONFIRM"),
        TextFont { font_size: FontSize::Px(20.0), ..default() },
        TextColor(Color::srgb(0.45, 0.45, 0.45)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(80.0),
            left: Val::Percent(36.0),
            ..default()
        },
    ));
    *selection = MenuSelection { index: 0, count: 2 };
}

pub(super) fn despawn_title(mut commands: Commands, query: Query<Entity, With<TitleUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

- [ ] **Step 4: 기본 상태를 Title로** — `src/core/state.rs`의 `GameState`에서 `#[default]`를 `Playing`에서 `Title`로 이동:

```rust
pub enum GameState {
    #[default]
    Title,
    Playing,
    Restarting,
    GameOver,
}
```

- [ ] **Step 5: UiPlugin에 배선** — `src/ui.rs`:

`mod title;` 추가. `menu`를 화면 모듈에서 접근하도록 `pub(crate) mod menu;`로 변경(title/pause/game_over가 `crate::ui::menu` 사용). 스케줄 추가:

```rust
            .add_systems(OnEnter(GameState::Title), title::spawn_title)
            .add_systems(OnExit(GameState::Title), title::despawn_title)
```

- [ ] **Step 6: 테스트·린트·수동 확인**

Run: `cargo test 2>&1 | tail -15 && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: 통과(`boots_to_title` 포함). 기존 `state::tests`는 명시적 Playing 진입이라 영향 없음.

Run: `cargo run` — 타이틀에서 부팅, ↑↓로 START/QUIT 이동(선택 항목 금빛+▶), Enter로 START → 게임 시작, QUIT → 종료.

- [ ] **Step 7: 커밋**

```bash
git add src/ui/title.rs src/ui.rs src/core/state.rs
git commit -m "feat: 타이틀 화면 추가 + 기본 상태를 Title로

게임명·최고점수·START/QUIT 메뉴·조작 힌트. 부팅이 Title에서 시작하고
START로 Playing 진입, QUIT로 종료. boots_to_title 테스트.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 4: 일시정지 화면 — ESC 토글 + 오버레이 + 메뉴

**Files:**
- Create: `src/ui/pause.rs`
- Modify: `src/ui.rs`(모듈 등록 + 스케줄)
- Test: `src/ui/pause.rs`(tests 모듈)

**Interfaces:**
- Consumes: `RunPhase`(Task 1); `MenuItem`, `MenuAction`, `MenuSelection`(Task 2).
- Produces: `PauseUi` 마커, `pause_input`, `resume_on_esc`, `spawn_pause_menu`, `despawn_pause_menu`.

- [ ] **Step 1: 실패하는 테스트 작성** — `src/ui/pause.rs` 생성, 하단:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn esc_pauses_when_running() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<RunPhase>();
        // Playing/Running 진입
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::Escape);
        app.insert_resource(input);
        app.world_mut().run_system_once(pause_input).unwrap();
        app.update();
        assert_eq!(*app.world().resource::<State<RunPhase>>().get(), RunPhase::Paused);
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --lib ui::pause 2>&1 | tail -15`
Expected: 컴파일 실패(`pause_input`/모듈 미정의).

- [ ] **Step 3: 일시정지 화면 구현** — `src/ui/pause.rs`:

```rust
//! 일시정지: ESC 토글, 어둠 오버레이, RESUME/RESTART/QUIT TO TITLE 메뉴.

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::core::state::{GameState, RunPhase};
use crate::ui::menu::{MenuAction, MenuItem, MenuSelection};

#[derive(Component)]
pub(super) struct PauseUi;

/// 진행 중 ESC → 일시정지.
pub(super) fn pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_phase: ResMut<NextState<RunPhase>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_phase.set(RunPhase::Paused);
    }
}

/// 일시정지 중 ESC → 재개(빠른 해제).
pub(super) fn resume_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_phase: ResMut<NextState<RunPhase>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_phase.set(RunPhase::Running);
    }
}

pub(super) fn spawn_pause_menu(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    // 전체 화면 어둠 오버레이
    commands.spawn((
        PauseUi,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
    ));
    // PAUSED
    commands.spawn((
        PauseUi,
        Text::new("PAUSED"),
        TextFont { font_size: FontSize::Px(56.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(28.0),
            left: Val::Percent(40.0),
            ..default()
        },
    ));
    // 메뉴 항목 3개
    let items = [
        (MenuAction::Resume, "RESUME"),
        (MenuAction::Restart, "RESTART"),
        (MenuAction::QuitToTitle, "QUIT TO TITLE"),
    ];
    for (i, (action, label)) in items.iter().enumerate() {
        commands.spawn((
            PauseUi,
            MenuItem { index: i, action: *action, label },
            Text::new(label.to_string()),
            TextFont { font_size: FontSize::Px(34.0), ..default() },
            TextColor(Color::srgb(0.55, 0.55, 0.55)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(46.0 + i as f32 * 8.0),
                left: Val::Percent(40.0),
                ..default()
            },
        ));
    }
    *selection = MenuSelection { index: 0, count: 3 };
}

pub(super) fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

- [ ] **Step 4: UiPlugin에 배선** — `src/ui.rs`:

`mod pause;` 추가. 스케줄:

```rust
            .add_systems(Update, pause::pause_input.run_if(in_state(RunPhase::Running)))
            .add_systems(Update, pause::resume_on_esc.run_if(in_state(RunPhase::Paused)))
            .add_systems(OnEnter(RunPhase::Paused), pause::spawn_pause_menu)
            .add_systems(OnExit(RunPhase::Paused), pause::despawn_pause_menu)
```

주의: `resume_on_esc`와 메뉴 엔진의 `menu_activate`는 서로 다른 키(ESC vs Enter)라 충돌하지 않는다. `pause_input`(Running)과 `resume_on_esc`(Paused)는 상태가 배타적이라 같은 프레임에 동시 실행되지 않는다.

- [ ] **Step 5: 테스트·린트·수동 확인**

Run: `cargo test 2>&1 | tail -15 && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: 통과(`esc_pauses_when_running` 포함).

Run: `cargo run` — 플레이 중 ESC → 게임 정지·어둠 오버레이·PAUSED 메뉴. RESUME/ESC로 재개(점수·엔티티 유지 확인), RESTART로 새 게임, QUIT TO TITLE로 타이틀 복귀.

- [ ] **Step 6: 커밋**

```bash
git add src/ui/pause.rs src/ui.rs
git commit -m "feat: 일시정지 화면(ESC) — 오버레이 + RESUME/RESTART/QUIT 메뉴

진행 중 ESC로 정지(RunPhase::Paused), 어둠 오버레이와 3항목 메뉴. ESC/RESUME
재개는 리셋 없이 유지, RESTART는 Restarting 경유 새 게임, QUIT TO TITLE은
타이틀 복귀. esc_pauses_when_running 테스트.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 5: 게임오버 화면을 메뉴 방식으로 전환

**Files:**
- Modify: `src/ui/game_over.rs`
- Modify: `src/ui.rs`(`restart_input` 등록 제거)
- Test: `src/ui/game_over.rs`(tests 모듈)

**Interfaces:**
- Consumes: `MenuItem`, `MenuAction`, `MenuSelection`(Task 2).
- Produces: 수정된 `spawn_game_over`(메뉴 항목 스폰), `restart_input` 제거.

- [ ] **Step 1: 실패하는 테스트 작성** — `src/ui/game_over.rs`에 tests 모듈 추가:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::menu::{MenuAction, MenuItem, MenuSelection};

    #[test]
    fn game_over_spawns_menu_items() {
        let mut app = App::new();
        app.insert_resource(Score(1234));
        // 최고점수 리소스(Persistent) 없이 테스트하기 위해 spawn_game_over가 Score만 읽도록 확인.
        // 아래 구현에서 High 표시는 유지하되 테스트는 항목 수만 검증.
        app.insert_resource(MenuSelection::default());
        app.world_mut().run_system_once(spawn_game_over_menu_only).unwrap();
        let count = app.world_mut().query::<&MenuItem>().iter(app.world()).count();
        assert_eq!(count, 2, "게임오버 메뉴는 RESTART/QUIT TO TITLE 2항목");
        assert_eq!(app.world().resource::<MenuSelection>().count, 2);
    }
}
```

> 참고: `spawn_game_over`는 `Persistent<HighScore>`를 요구해 테스트 셋업이 무겁다. 항목 생성 로직만 순수하게 검증하도록, 메뉴 항목 스폰을 별도 헬퍼 `spawn_game_over_menu_only(commands, selection)`로 분리하고 `spawn_game_over`가 이를 호출하게 한다(아래 구현).

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --lib ui::game_over 2>&1 | tail -15`
Expected: 컴파일 실패(`spawn_game_over_menu_only`/`MenuItem` 미정의 참조).

- [ ] **Step 3: 게임오버 화면 수정** — `src/ui/game_over.rs` 전체를 다음으로 교체:

```rust
//! 게임오버 화면과 메뉴(RESTART / QUIT TO TITLE).

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;

use crate::core::state::{HighScore, Score};
use crate::ui::menu::{MenuAction, MenuItem, MenuSelection};

#[derive(Component)]
pub(super) struct GameOverScreen;

pub(super) fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    high: Res<Persistent<HighScore>>,
    selection: ResMut<MenuSelection>,
) {
    commands.spawn((
        GameOverScreen,
        Text::new(format!("GAME OVER\nScore: {}\nHigh: {}", score.0, high.0)),
        TextFont { font_size: FontSize::Px(40.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(38.0),
            ..default()
        },
    ));
    spawn_game_over_menu_only(commands, selection);
}

/// 메뉴 항목 2개 스폰 + 선택 초기화(테스트가 이 로직만 검증한다).
pub(super) fn spawn_game_over_menu_only(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    let items = [
        (MenuAction::Restart, "RESTART"),
        (MenuAction::QuitToTitle, "QUIT TO TITLE"),
    ];
    for (i, (action, label)) in items.iter().enumerate() {
        commands.spawn((
            GameOverScreen,
            MenuItem { index: i, action: *action, label },
            Text::new(label.to_string()),
            TextFont { font_size: FontSize::Px(32.0), ..default() },
            TextColor(Color::srgb(0.55, 0.55, 0.55)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(58.0 + i as f32 * 8.0),
                left: Val::Percent(42.0),
                ..default()
            },
        ));
    }
    *selection = MenuSelection { index: 0, count: 2 };
}

pub(super) fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

주의: 기존 `restart_input` 함수는 삭제한다(메뉴로 대체).

- [ ] **Step 4: UiPlugin에서 restart_input 제거** — `src/ui.rs`의 다음 라인을 삭제:

```rust
            .add_systems(Update, game_over::restart_input.run_if(in_state(GameState::GameOver)));
```

`OnEnter(GameState::GameOver)` → `game_over::spawn_game_over`, `OnExit(GameState::GameOver)` → `game_over::despawn_game_over` 등록은 유지. 만약 이 삭제로 체인 마지막 `;` 위치가 바뀌면 문법 정리.

- [ ] **Step 5: 테스트·린트·수동 확인**

Run: `cargo test 2>&1 | tail -15 && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: 통과. 기존 게임오버 관련 테스트가 있으면 함께 통과.

Run: `cargo run` — 사망 후 게임오버 화면에 RESTART/QUIT TO TITLE 메뉴. ↑↓/Enter로 재시작·타이틀 복귀. R키는 더 이상 동작하지 않음.

- [ ] **Step 6: 커밋**

```bash
git add src/ui/game_over.rs src/ui.rs
git commit -m "feat: 게임오버 화면을 방향키 메뉴로 전환

기존 'Press R' 즉시 재시작을 제거하고 RESTART/QUIT TO TITLE 방향키 메뉴로
통일. 재시작은 Restarting 경유. 메뉴 항목 스폰 로직 회귀 테스트.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## 전체 완료 후 수동 통합 확인(로컬)

`cargo run`으로 전체 흐름:
1. 부팅 → 타이틀(스타필드 배경) → START.
2. 플레이 중 ESC → 정지(오버레이·PAUSED) → RESUME/ESC 재개(점수·엔티티 유지).
3. 정지 중 RESTART → 새 게임(점수 0). QUIT TO TITLE → 타이틀.
4. 사망 → 게임오버 메뉴 → RESTART / QUIT TO TITLE.
5. 타이틀 QUIT → 종료.

---

## Self-Review 결과(작성자 점검)

- **스펙 커버리지:** §2 상태 모델→T1, §3 내비게이션(MenuAction 디스패치)→T2, §4 정지 의미론(게이팅·ESC)→T1+T4, §5 메뉴 UI→T2, §6 화면 내용(Title/Pause/GameOver)→T3/T4/T5, §9 테스트(재개-무리셋·재시작-리셋·기본상태·순환·전이)→T1/T2/T3/T4. 누락 없음.
- **플레이스홀더:** 없음(모든 스텝에 실제 코드·명령·기대결과).
- **타입 일관성:** `MenuAction`/`MenuItem{index,action,label}`/`MenuSelection{index,count}`/`wrap_index`/`RunPhase{Running,Paused}`/`GameState{Title,Playing,Restarting,GameOver}`가 T1~T5에서 일관. `menu_move`/`menu_activate`/`highlight_menu` 시그니처가 T2 정의와 배선 일치. AppExit=`MessageWriter`+`write(AppExit::Success)` 일관.
