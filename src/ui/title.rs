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
        Text::new("UP/DOWN SELECT   ENTER CONFIRM"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::core::state::{GameState, GameStatePlugin};

    #[test]
    fn boots_to_title() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(GameStatePlugin);
        app.update();
        assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::Title);
    }

    #[test]
    fn spawn_title_creates_two_menu_items_with_expected_actions() {
        let dir = std::env::temp_dir().join("bevy-asteroids-spawn-title-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("highscore.json");

        let hs = Persistent::<HighScore>::builder()
            .name("test high score spawn title")
            .format(StorageFormat::Json)
            .path(path)
            .default(HighScore(0))
            .build()
            .unwrap();

        let mut app = App::new();
        app.insert_resource(hs);
        app.insert_resource(MenuSelection::default());
        app.world_mut().run_system_once(spawn_title).unwrap();

        let mut items: Vec<(usize, MenuAction)> = app
            .world_mut()
            .query::<&MenuItem>()
            .iter(app.world())
            .map(|item| (item.index, item.action))
            .collect();
        items.sort_by_key(|(index, _)| *index);

        assert_eq!(items, vec![(0, MenuAction::StartGame), (1, MenuAction::QuitApp)]);

        let selection = app.world().resource::<MenuSelection>();
        assert_eq!(selection.index, 0);
        assert_eq!(selection.count, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
