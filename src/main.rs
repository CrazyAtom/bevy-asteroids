mod core;
mod entities;
mod fx;
mod systems;
mod ui;

use bevy::prelude::*;
use bevy_persistent::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Asteroids".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
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
        .add_plugins(entities::ufo::UfoPlugin)
        .add_plugins(ui::UiPlugin)
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
