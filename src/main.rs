mod core;
mod entities;
mod fx;
mod systems;
mod ui;

use bevy::camera::{Hdr, OrthographicProjection, Projection, ScalingMode};
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
                        // 웹 빌드에서 지정한 셀렉터의 캔버스에 렌더링한다(네이티브에선 no-op).
                        // fit_canvas_to_parent는 켜지 않는다 — 켜면 Bevy가 canvas에 인라인
                        // width/height:100%를 박아 아래 CSS를 덮어쓰고 UI(퍼센트 배치)가 창
                        // 전체로 퍼진다. 대신 index.html CSS가 캔버스를 16:9로 고정한다.
                        // (주의: 웹에선 winit ResizeObserver가 논리 해상도를 캔버스 크기에
                        // 계속 맞추므로 해상도는 1280x720 고정이 아니라 '항상 16:9'로 유동한다.
                        // 퍼센트 기반 UI는 어느 16:9 크기에서도 비율이 맞다.)
                        canvas: Some("#bevy-canvas".into()),
                        ..default()
                    }),
                    ..default()
                })
                // IDE(F5)나 바이너리 직접 실행 시 CARGO_MANIFEST_DIR가 없어 Bevy가
                // 실행 파일 옆(target/debug/assets)에서 에셋을 찾는 문제를 방지한다.
                // 컴파일 타임 프로젝트 경로를 박아, 실행 방식과 무관하게 항상
                // <project>/assets 에서 에셋(사운드 WAV)을 찾게 한다.
                .set(AssetPlugin {
                    file_path: asset_path(),
                    meta_check: asset_meta_check(),
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
        .add_plugins(fx::music::MusicPlugin)
        .add_plugins(entities::ufo::UfoPlugin)
        .add_plugins(entities::powerup::PowerupPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(fx::sprites::SpritesPlugin)
        .add_plugins(fx::animation::AnimationPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

/// 에셋 루트 경로. 네이티브는 절대경로(IDE 실행 대응), wasm은 상대경로(HTTP 로딩).
fn asset_path() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string()
    }
    #[cfg(target_arch = "wasm32")]
    {
        "assets".to_string()
    }
}

/// 에셋 메타(.meta) 검사 정책. wasm은 .meta 파일이 없어 매 에셋마다 HTTP 404가 나
/// 콘솔이 지저분해지므로 검사를 끈다(게임은 기본 메타로 정상 로딩). 네이티브는
/// 파일시스템 read_meta가 조용히 NotFound 처리하므로 기본값(Always) 유지.
fn asset_meta_check() -> bevy::asset::AssetMetaCheck {
    #[cfg(not(target_arch = "wasm32"))]
    {
        bevy::asset::AssetMetaCheck::default()
    }
    #[cfg(target_arch = "wasm32")]
    {
        bevy::asset::AssetMetaCheck::Never
    }
}

fn setup_camera(mut commands: Commands) {
    // HDR + Bloom으로 색값 1.0 초과 스프라이트(공격형 실드 우주선 등)를 실제 발광시킨다.
    // Tonemapping::None으로 나머지(LDR) 스프라이트 색은 그대로 유지해 게임 전체 룩은 불변.
    commands.spawn((
        Camera2d,
        // 웹에선 창 크기에 따라 논리 해상도가 유동한다. 고정 스케일링으로 게임 월드
        // (±640×±360 = 1280x720)가 어느 16:9 창에서도 화면을 꽉 채우게 한다(더 큰 창에서
        // 소행성이 화면 밖에서 순환하지 않도록). 네이티브(1280x720)에선 사실상 no-op.
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin { min_width: 1280.0, min_height: 720.0 },
            ..OrthographicProjection::default_2d()
        }),
        Hdr, // HDR 중간 렌더 텍스처 활성(0.19: Camera.hdr 필드 대신 마커 컴포넌트)
        Tonemapping::None,
        Bloom { intensity: 0.3, ..default() },
    ));
}

fn load_high_score() -> Persistent<core::state::HighScore> {
    // 네이티브는 OS 설정 디렉터리의 JSON 파일에, wasm은 브라우저 localStorage
    // 키에 저장한다(dirs::config_dir()가 wasm에서 항상 None을 반환해 그대로 쓰면
    // bevy-persistent 초기화가 패닉한다).
    #[cfg(not(target_arch = "wasm32"))]
    let path = dirs::config_dir()
        .map(|d| d.join("bevy-asteroids"))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("highscore.json");
    #[cfg(target_arch = "wasm32")]
    let path = std::path::PathBuf::from("local/bevy-asteroids-highscore");

    Persistent::<core::state::HighScore>::builder()
        .name("high score")
        .format(StorageFormat::Json)
        .path(path)
        .default(core::state::HighScore(0))
        .revertible(true)
        .revert_to_default_on_deserialization_errors(true)
        .build()
        .expect("최고점수 리소스 초기화 실패")
}
