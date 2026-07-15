use bevy::prelude::*;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::config::STARTING_LIVES;
use crate::core::logic::update_high_score;
use crate::fx::audio::{Sfx, SfxEvent};

// `Title`과 `RunPhase::Paused`는 이 태스크(상태 뼈대)에서는 아직 어떤 시스템도 진입시키지
// 않는다(타이틀 화면·일시정지 입력 배선은 후속 태스크). 프로덕션 코드에서 미사용이라
// clippy dead_code 경고가 뜨므로 명시적으로 allow. resume_does_not_reset_score /
// restart_via_bounce_resets_score 테스트가 `Paused`/`Restarting` 전이를 검증한다.
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    Paused,
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Resource)]
pub struct Lives(pub u32);

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct HighScore(pub u32);

/// 한 판(Playing) 동안 존재하는 모든 엔티티에 붙는 마커. 판이 끝나면 일괄 정리된다.
#[derive(Component)]
pub struct GameplayEntity;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<RunPhase>()
            .insert_resource(Score(0))
            .insert_resource(Lives(STARTING_LIVES))
            .insert_resource(crate::systems::stage::new_progression())
            .add_systems(OnEnter(GameState::Playing), reset_game)
            .add_systems(OnExit(GameState::Playing), despawn_gameplay_entities)
            .add_systems(OnEnter(GameState::Restarting), bounce_to_playing)
            .add_systems(OnEnter(GameState::GameOver), (save_high_score, play_game_over_sfx));
    }
}

pub(crate) fn reset_game(
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut prog: ResMut<crate::systems::stage::Progression>,
    mut ufo_spawn_timer: ResMut<crate::entities::ufo::UfoSpawnTimer>,
) {
    score.0 = 0;
    lives.0 = STARTING_LIVES;
    *prog = crate::systems::stage::new_progression();
    ufo_spawn_timer.0 = Timer::from_seconds(crate::core::config::UFO_SPAWN_INTERVAL_BASE, TimerMode::Once);
}

/// Restarting은 1프레임 바운스 상태다. 진입 즉시 Playing으로 넘겨
/// OnExit(Playing)→OnEnter(Playing)의 전면 teardown+rebuild를 유발한다.
fn bounce_to_playing(mut next: ResMut<NextState<GameState>>) {
    next.set(GameState::Playing);
}

fn despawn_gameplay_entities(mut commands: Commands, query: Query<Entity, With<GameplayEntity>>) {
    for entity in &query {
        // 같은 프레임에 이미 despawn된 엔티티(충돌·수명 등)가 섞일 수 있으므로 try_despawn.
        commands.entity(entity).try_despawn();
    }
}

fn save_high_score(score: Res<Score>, mut high: ResMut<Persistent<HighScore>>) {
    let updated = update_high_score(high.0, score.0);
    if updated != high.0 {
        high.0 = updated;
        let _ = high.persist();
    }
}

fn play_game_over_sfx(mut sfx: MessageWriter<SfxEvent>) {
    sfx.write(SfxEvent(Sfx::GameOver));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::core::config::UFO_SPAWN_INTERVAL_BASE;
    use crate::entities::ufo::UfoSpawnTimer;

    #[test]
    fn restart_resets_ufo_spawn_timer() {
        let mut app = App::new();
        app.insert_resource(Score(50));
        app.insert_resource(Lives(0));
        app.insert_resource(crate::systems::stage::new_progression());
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
}
