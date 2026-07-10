// src/state.rs
use bevy::prelude::*;

use crate::config::STARTING_LIVES;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Resource)]
pub struct Lives(pub u32);

/// 한 판(Playing) 동안 존재하는 모든 엔티티에 붙는 마커. 판이 끝나면 일괄 정리된다.
#[derive(Component)]
pub struct GameplayEntity;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(Score(0))
            .insert_resource(Lives(STARTING_LIVES))
            .add_systems(OnEnter(GameState::Playing), reset_game)
            .add_systems(OnExit(GameState::Playing), despawn_gameplay_entities);
    }
}

fn reset_game(mut score: ResMut<Score>, mut lives: ResMut<Lives>) {
    score.0 = 0;
    lives.0 = STARTING_LIVES;
}

fn despawn_gameplay_entities(mut commands: Commands, query: Query<Entity, With<GameplayEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
