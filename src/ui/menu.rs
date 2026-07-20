//! 타이틀·일시정지·게임오버가 공유하는 방향키 메뉴 엔진.
//! 화면은 MenuItem(Text) 엔티티들을 스폰하고 MenuSelection을 초기화한다.
//! 이 모듈은 이동(menu_move)·확정(menu_activate)·강조(highlight_menu)를 담당한다.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_persistent::prelude::*;

use crate::core::state::{GameState, RunPhase};
use crate::fx::music::MusicEnabled;
use crate::ui::help::HelpOpen;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuAction {
    StartGame,
    Resume,
    Restart,
    QuitToTitle,
    // 웹(WASM)에선 타이틀에서 QUIT를 빼므로 이 변형이 생성되지 않는다(의도된 미사용).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    QuitApp,
    ShowHelp,
    ToggleMusic,
}

/// 메뉴 한 항목(Text 엔티티에 부착). 화면 마커와 함께 스폰된다.
#[derive(Component)]
pub struct MenuItem {
    pub index: usize,
    pub action: MenuAction,
    pub label: &'static str,
}

/// 음악 on/off 토글 메뉴 항목 마커. 라벨이 상태를 반영하도록 별도 시스템이 갱신한다.
#[derive(Component)]
pub struct MusicMenuItem;

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
#[allow(clippy::too_many_arguments)]
pub fn menu_activate(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<MenuSelection>,
    items: Query<&MenuItem>,
    mut next_game: ResMut<NextState<GameState>>,
    mut next_phase: ResMut<NextState<RunPhase>>,
    mut exit: MessageWriter<AppExit>,
    mut help: ResMut<HelpOpen>,
    mut music: ResMut<Persistent<MusicEnabled>>,
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
        // 하부 상태(Title/Paused)는 그대로 두고 HELP 오버레이만 켠다. close_help가
        // 다음 프레임부터 아무 키로 닫으며, 그동안 이 메뉴 시스템은 run_if로 억제된다.
        MenuAction::ShowHelp => help.0 = true,
        // 상태 전이 없이 음소거만 토글(지속). 라벨은 update_music_menu_label이 갱신.
        MenuAction::ToggleMusic => {
            music.0 = !music.0;
            let _ = music.persist();
        }
    }
}

/// 음악 토글 항목의 라벨을 현재 상태로 갱신한다(`MUSIC: ON/OFF`, 선택 시 `> `).
/// `highlight_menu`가 정적 라벨로 덮어쓴 뒤 실행되어 동적 상태로 다시 쓴다.
pub fn update_music_menu_label(
    selection: Res<MenuSelection>,
    music: Res<Persistent<MusicEnabled>>,
    mut items: Query<(&MenuItem, &mut Text), With<MusicMenuItem>>,
) {
    let state = if music.0 { "ON" } else { "OFF" };
    for (item, mut text) in &mut items {
        let prefix = if item.index == selection.index { "> " } else { "" };
        *text = Text::new(format!("{prefix}MUSIC: {state}"));
    }
}

/// 선택 항목은 밝은 금빛 + "> " 프리픽스, 나머지는 흐리게.
/// 색값 > 1.0은 HDR+Bloom(카메라에 설정됨) 지원 시 발광한다.
pub fn highlight_menu(
    selection: Res<MenuSelection>,
    mut items: Query<(&MenuItem, &mut Text, &mut TextColor)>,
) {
    for (item, mut text, mut color) in &mut items {
        if item.index == selection.index {
            *text = Text::new(format!("> {}", item.label));
            color.0 = Color::srgb(1.5, 1.2, 0.3);
        } else {
            *text = Text::new(item.label.to_string());
            color.0 = Color::srgb(0.55, 0.55, 0.55);
        }
    }
}

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
