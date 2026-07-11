mod asteroid;
mod bullet;
mod collision;
mod components;
mod config;
mod effects;
mod logic;
mod movement;
mod player;
mod state;
mod ufo;
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
        .add_plugins(movement::MovementPlugin)
        .add_plugins(state::GameStatePlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(bullet::BulletPlugin)
        .add_plugins(asteroid::AsteroidPlugin)
        .add_plugins(collision::CollisionPlugin)
        .add_plugins(effects::EffectsPlugin)
        .add_plugins(ufo::UfoPlugin)
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
        Persistent::<state::HighScore>::builder()
            .name("high score")
            .format(StorageFormat::Json)
            .path(dir.join("highscore.json"))
            .default(state::HighScore(0))
            .build()
            .expect("최고점수 리소스 초기화 실패"),
    );
}
