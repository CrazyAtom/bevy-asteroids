//! 게임오버 화면과 재시작 입력(R).

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;

use crate::core::state::{GameState, HighScore, Score};

#[derive(Component)]
pub(super) struct GameOverScreen;

pub(super) fn spawn_game_over(mut commands: Commands, score: Res<Score>, high: Res<Persistent<HighScore>>) {
    commands.spawn((
        GameOverScreen,
        Text::new(format!(
            "GAME OVER\nScore: {}\nHigh: {}\nPress R to restart",
            score.0, high.0
        )),
        TextFont { font_size: FontSize::Px(40.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(34.0),
            ..default()
        },
    ));
}

pub(super) fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(super) fn restart_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}
