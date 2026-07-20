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
                // 에셋 루트는 asset_path()가 런타임에 해결한다. 배포본은 실행 파일
                // 위치 기준(.app의 ../Resources/assets → 실행 파일 옆 assets)으로 먼저
                // 찾고, 없으면 개발용 소스 트리(CARGO_MANIFEST_DIR/assets)로 폴백한다.
                // wasm은 상대경로("assets")로 HTTP 로딩한다.
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
        // 음소거 설정도 빌드 시점에 삽입(sync_music이 첫 프레임부터 참조).
        .insert_resource(fx::music::load_music_enabled())
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

/// 실행파일 디렉터리 기준으로 배포 에셋 위치를 찾는다(순수 파일시스템 검사).
/// macOS `.app`(../Resources/assets) → 실행파일 옆(assets) 순으로 우선.
#[cfg(not(target_arch = "wasm32"))]
fn resolve_asset_dir(exe_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let bundle = exe_dir.join("../Resources/assets");
    if bundle.is_dir() {
        return Some(bundle);
    }
    let sibling = exe_dir.join("assets");
    if sibling.is_dir() {
        return Some(sibling);
    }
    None
}

/// 에셋 루트 경로. 배포본은 실행 위치 기준, 개발(cargo run)은 소스 트리 fallback,
/// wasm은 상대경로(HTTP 로딩).
fn asset_path() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .and_then(|dir| resolve_asset_dir(&dir))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string())
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod asset_path_tests {
    use super::resolve_asset_dir;
    use std::fs;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("bevy_ast_assettest_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn prefers_macos_bundle_resources() {
        let root = temp_root("bundle");
        let macos = root.join("Contents/MacOS");
        let res_assets = root.join("Contents/Resources/assets");
        fs::create_dir_all(&macos).unwrap();
        fs::create_dir_all(&res_assets).unwrap();
        let got = resolve_asset_dir(&macos).expect("번들 Resources/assets를 찾아야 함");
        assert_eq!(got.canonicalize().unwrap(), res_assets.canonicalize().unwrap());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn falls_back_to_sibling_assets() {
        let root = temp_root("sibling");
        let exe_dir = root.join("bin");
        let sibling = exe_dir.join("assets");
        fs::create_dir_all(&sibling).unwrap();
        let got = resolve_asset_dir(&exe_dir).expect("실행파일 옆 assets를 찾아야 함");
        assert_eq!(got.canonicalize().unwrap(), sibling.canonicalize().unwrap());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn none_when_no_assets_present() {
        let root = temp_root("none");
        let exe_dir = root.join("bin");
        fs::create_dir_all(&exe_dir).unwrap();
        assert!(resolve_asset_dir(&exe_dir).is_none());
        fs::remove_dir_all(&root).ok();
    }
}
