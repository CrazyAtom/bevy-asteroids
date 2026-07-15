//! 일시정지: ESC 토글, 어둠 오버레이, RESUME/RESTART/QUIT TO TITLE 메뉴.

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::core::state::RunPhase;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::core::state::GameState;

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
