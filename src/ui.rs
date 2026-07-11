use bevy::prelude::*;
use bevy::text::FontSize;

use crate::state::{GameState, GameplayEntity, Lives, Score};

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct GameOverScreen;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(Update, restart_input.run_if(in_state(GameState::GameOver)));
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Hud,
        GameplayEntity,
        Text::new("Score: 0   Lives: 3"),
        TextFont { font_size: FontSize::Px(24.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn update_hud(score: Res<Score>, lives: Res<Lives>, mut query: Query<&mut Text, With<Hud>>) {
    for mut text in &mut query {
        text.0 = format!("Score: {}   Lives: {}", score.0, lives.0);
    }
}

fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        GameOverScreen,
        Text::new(format!("GAME OVER\nScore: {}\nPress R to restart", score.0)),
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

fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn restart_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn hud_shows_score_and_lives() {
        let mut app = App::new();
        app.insert_resource(Score(150));
        app.insert_resource(Lives(2));
        let e = app.world_mut().spawn((Hud, Text::new("초기값"))).id();
        app.world_mut().run_system_once(update_hud).unwrap();
        let text = app.world().entity(e).get::<Text>().unwrap();
        assert!(text.0.contains("150"));
        assert!(text.0.contains('2'));
    }
}
