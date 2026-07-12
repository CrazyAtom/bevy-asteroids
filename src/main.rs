mod core;
mod entities;
mod fx;
mod systems;
mod ui;

use bevy::prelude::*;
use bevy_persistent::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Asteroids".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                // IDE(F5)나 바이너리 직접 실행 시 CARGO_MANIFEST_DIR가 없어 Bevy가
                // 실행 파일 옆(target/debug/assets)에서 에셋을 찾는 문제를 방지한다.
                // 컴파일 타임 프로젝트 경로를 박아, 실행 방식과 무관하게 항상
                // <project>/assets 에서 에셋(사운드 WAV)을 찾게 한다.
                .set(AssetPlugin {
                    file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string(),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(systems::movement::MovementPlugin)
        .add_plugins(core::state::GameStatePlugin)
        .add_plugins(entities::player::PlayerPlugin)
        .add_plugins(entities::bullet::BulletPlugin)
        .add_plugins(entities::asteroid::AsteroidPlugin)
        .add_plugins(systems::collision::CollisionPlugin)
        .add_plugins(fx::effects::EffectsPlugin)
        .add_plugins(fx::background::BackgroundPlugin)
        .add_plugins(fx::shake::ShakePlugin)
        .add_plugins(fx::audio::AudioPlugin)
        .add_plugins(entities::ufo::UfoPlugin)
        .add_plugins(entities::powerup::PowerupPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(fx::sprites::SpritesPlugin)
        .add_systems(Startup, (setup_camera, setup_high_score))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_high_score(mut commands: Commands) {
    let dir = dirs::config_dir()
        .map(|d| d.join("bevy-asteroids"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    commands.insert_resource(
        Persistent::<core::state::HighScore>::builder()
            .name("high score")
            .format(StorageFormat::Json)
            .path(dir.join("highscore.json"))
            .default(core::state::HighScore(0))
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .expect("최고점수 리소스 초기화 실패"),
    );
}
