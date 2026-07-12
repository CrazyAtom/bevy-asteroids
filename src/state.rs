use bevy::prelude::*;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::STARTING_LIVES;
use crate::logic::update_high_score;

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

#[derive(Resource)]
pub struct Wave(pub u32);

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct HighScore(pub u32);

/// 한 판(Playing) 동안 존재하는 모든 엔티티에 붙는 마커. 판이 끝나면 일괄 정리된다.
#[derive(Component)]
pub struct GameplayEntity;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(Score(0))
            .insert_resource(Lives(STARTING_LIVES))
            .insert_resource(Wave(1))
            .add_systems(OnEnter(GameState::Playing), reset_game)
            .add_systems(OnExit(GameState::Playing), despawn_gameplay_entities)
            .add_systems(OnEnter(GameState::GameOver), save_high_score);
    }
}

fn reset_game(
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut wave: ResMut<Wave>,
    mut ufo_spawn_timer: ResMut<crate::ufo::UfoSpawnTimer>,
) {
    score.0 = 0;
    lives.0 = STARTING_LIVES;
    wave.0 = 1;
    ufo_spawn_timer.0 = Timer::from_seconds(crate::config::UFO_SPAWN_INTERVAL_BASE, TimerMode::Once);
}

fn despawn_gameplay_entities(mut commands: Commands, query: Query<Entity, With<GameplayEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn save_high_score(score: Res<Score>, mut high: ResMut<Persistent<HighScore>>) {
    let updated = update_high_score(high.0, score.0);
    if updated != high.0 {
        high.0 = updated;
        let _ = high.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::config::UFO_SPAWN_INTERVAL_BASE;
    use crate::ufo::UfoSpawnTimer;

    #[test]
    fn restart_resets_ufo_spawn_timer() {
        let mut app = App::new();
        app.insert_resource(Score(50));
        app.insert_resource(Lives(0));
        app.insert_resource(Wave(5));
        let mut stale_timer = Timer::from_seconds(UFO_SPAWN_INTERVAL_BASE, TimerMode::Once);
        stale_timer.tick(std::time::Duration::from_secs_f32(UFO_SPAWN_INTERVAL_BASE));
        app.insert_resource(UfoSpawnTimer(stale_timer));

        app.world_mut().run_system_once(reset_game).unwrap();

        let timer = &app.world().resource::<UfoSpawnTimer>().0;
        assert_eq!(timer.duration().as_secs_f32(), UFO_SPAWN_INTERVAL_BASE);
        assert!(!timer.is_finished());
    }

    #[test]
    fn corrupt_high_score_file_reverts_to_default() {
        let dir = std::env::temp_dir().join("bevy-asteroids-revert-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("highscore.json");
        std::fs::write(&path, "not valid json{{{").unwrap();

        let result = Persistent::<HighScore>::builder()
            .name("test high score revert")
            .format(StorageFormat::Json)
            .path(path)
            .default(HighScore(0))
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
