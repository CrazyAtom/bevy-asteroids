//! 일시정지: ESC 토글, 어둠 오버레이, RESUME/RESTART/QUIT TO TITLE 메뉴.

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::core::state::RunPhase;
use crate::ui::menu::{MenuAction, MenuItem, MenuSelection, MusicMenuItem};
use crate::ui::scaling::spawn_stage;

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
    // 전체 화면 어둠 오버레이 (게임 스프라이트·HUD보다 위, 메뉴 텍스트보다 아래)
    commands.spawn((
        PauseUi,
        GlobalZIndex(1),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
    ));
    // PAUSED + 메뉴를 1280×720 스테이지 위에 올린다(위치 Percent는 그대로지만
    // 이제 스테이지 기준이라 ui_scale로 텍스트·간격이 함께 스케일된다).
    let items = [
        (MenuAction::Resume, "RESUME"),
        (MenuAction::Restart, "RESTART"),
        (MenuAction::ShowHelp, "HELP"),
        (MenuAction::ToggleMusic, "MUSIC"),
        (MenuAction::QuitToTitle, "QUIT TO TITLE"),
    ];
    let stage = spawn_stage(&mut commands, (PauseUi, GlobalZIndex(2)));
    commands.entity(stage).with_children(|s| {
        s.spawn((
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
        for (i, (action, label)) in items.iter().enumerate() {
            let mut e = s.spawn((
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
            if *action == MenuAction::ToggleMusic {
                e.insert(MusicMenuItem);
            }
        }
    });
    *selection = MenuSelection { index: 0, count: 5 };
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
