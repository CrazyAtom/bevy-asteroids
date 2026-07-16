mod core;
mod entities;
mod fx;
mod systems;
mod ui;

use bevy::camera::Hdr;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
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
        // 최고점수는 App 빌드 시점에 동기 삽입한다. Bevy 0.19에서 초기 상태 전이
        // (OnEnter(Title)→spawn_title, Persistent<HighScore> 참조)는 Startup보다 먼저
        // 도는 StateTransition 패스에서 실행되므로, Startup 시스템에서 지연 삽입하면
        // 부팅 첫 프레임에 "Resource does not exist" 패닉이 난다.
        .insert_resource(load_high_score())
        .add_plugins(systems::movement::MovementPlugin)
        .add_plugins(core::state::GameStatePlugin)
        .add_plugins(entities::player::PlayerPlugin)
        .add_plugins(entities::black_hole::BlackHolePlugin)
        .add_plugins(entities::special_weapon::SpecialWeaponPlugin)
        .add_plugins(entities::bullet::BulletPlugin)
        .add_plugins(systems::stage::StagePlugin)
        .add_plugins(entities::boss::BossPlugin)
        .add_plugins(systems::collision::CollisionPlugin)
        .add_plugins(fx::effects::EffectsPlugin)
        .add_plugins(fx::background::BackgroundPlugin)
        .add_plugins(fx::fog::FogPlugin)
        .add_plugins(fx::shake::ShakePlugin)
        .add_plugins(fx::audio::AudioPlugin)
        .add_plugins(entities::ufo::UfoPlugin)
        .add_plugins(entities::powerup::PowerupPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(fx::sprites::SpritesPlugin)
        .add_plugins(fx::animation::AnimationPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    // HDR + Bloom으로 색값 1.0 초과 스프라이트(공격형 실드 우주선 등)를 실제 발광시킨다.
    // Tonemapping::None으로 나머지(LDR) 스프라이트 색은 그대로 유지해 게임 전체 룩은 불변.
    commands.spawn((
        Camera2d,
        Hdr, // HDR 중간 렌더 텍스처 활성(0.19: Camera.hdr 필드 대신 마커 컴포넌트)
        Tonemapping::None,
        Bloom { intensity: 0.3, ..default() },
    ));
}

fn load_high_score() -> Persistent<core::state::HighScore> {
    let dir = dirs::config_dir()
        .map(|d| d.join("bevy-asteroids"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    Persistent::<core::state::HighScore>::builder()
        .name("high score")
        .format(StorageFormat::Json)
        .path(dir.join("highscore.json"))
        .default(core::state::HighScore(0))
        .revertible(true)
        .revert_to_default_on_deserialization_errors(true)
        .build()
        .expect("최고점수 리소스 초기화 실패")
}
