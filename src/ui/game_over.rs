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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::ui::menu::MenuItem;

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
