//! 타이틀 화면: 아케이드 어트랙트 모드.
//! 진입 시 함선이 타이틀 라인을 좌→우로 훑으며 글자가 드러나는 인트로(추진 잔상 포함)를 재생하고,
//! 인트로가 끝나면(또는 스킵되면) HDR+Bloom으로 발광하는 타이틀 + 정박해 회전하는 함선 +
//! 메뉴(START/QUIT)를 보여준다. 타이틀 문구는 월드 공간 `Text2d`라 스프라이트처럼 Bloom을 탄다.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;
use rand::RngExt;

use crate::core::components::{AngularVelocity, Velocity};
use crate::core::config::{HALF_HEIGHT, HALF_WIDTH};
use crate::core::logic::{wrap_position, AsteroidSize};
use crate::core::state::HighScore;
use crate::fx::sprites::{sprite_size_for, SpriteAssets};
use crate::ui::help::HelpOpen;
use crate::ui::menu::{MenuAction, MenuItem, MenuSelection};
use crate::ui::scaling::spawn_stage;

const TITLE_TEXT: &str = "ASTEROIDS";
const INTRO_SECS: f32 = 1.4;
/// 타이틀 Text2d(월드)의 좌측 기준점 x와 세로 위치 y. 우주선 스윕도 이 라인을 따라 정렬된다.
const TITLE_LEFT_X: f32 = -235.0;
const TITLE_Y: f32 = 130.0;
/// "ASTEROIDS"(72px)의 대략 가로폭(시각 근사). 스윕 종료 x 계산에 쓴다.
const TITLE_WIDTH: f32 = 470.0;
const SHIP_START_X: f32 = TITLE_LEFT_X - 55.0;
const SHIP_END_X: f32 = TITLE_LEFT_X + TITLE_WIDTH;
/// 타이틀 월드 스프라이트/텍스트의 z(배경보다 위).
const TITLE_Z: f32 = crate::core::config::Z_ENTITY;

#[derive(Component)]
pub(super) struct TitleUi;

/// 타이틀 문구 Text2d 엔티티 마커.
#[derive(Component)]
pub(super) struct TitleBanner;

/// 장식용 함선 스프라이트 마커.
#[derive(Component)]
pub(super) struct TitleShip;

/// 어트랙트 모드 배경에 떠다니는 소행성 마커.
#[derive(Component)]
pub(super) struct TitleDebris;

/// 인트로 스윕 중 함선이 남기는 추진 잔상(수명 동안 페이드).
#[derive(Component)]
pub(super) struct TitleTrail {
    life: Timer,
}

/// 타이틀 인트로(스윕 연출) 진행 상태.
#[derive(Resource)]
pub(super) struct TitleAnim {
    timer: Timer,
    ready: bool,
}

pub(super) fn spawn_title(
    mut commands: Commands,
    high: Res<Persistent<HighScore>>,
    assets: Res<SpriteAssets>,
    mut selection: ResMut<MenuSelection>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    // Pause/GameOver에서 Enter로 QuitToTitle 하면 그 Enter의 just_pressed가 첫 Title
    // 프레임까지 남아 인트로가 즉시 스킵될 수 있다. 진입 시 입력 엣지를 비워, 재진입에도
    // 어트랙트 모드 인트로가 항상 재생되게 한다(이후 프레임의 실제 스킵 입력은 정상 처리).
    keys.clear();

    commands.insert_resource(TitleAnim {
        timer: Timer::from_seconds(INTRO_SECS, TimerMode::Once),
        ready: false,
    });

    // 타이틀(월드 Text2d → HDR+Bloom 발광). 인트로 동안 글자가 점차 드러난다(시작은 빈 문자열).
    // CENTER_LEFT 앵커로 좌측 기준점에서 오른쪽으로 자라 우주선 스윕과 정렬된다.
    commands.spawn((
        TitleUi,
        TitleBanner,
        Text2d::new(""),
        TextFont { font_size: FontSize::Px(72.0), ..default() },
        TextColor(Color::WHITE),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(TITLE_LEFT_X, TITLE_Y, TITLE_Z),
    ));
    // 최고점수·버전은 UI 노드(월드 Text2d/스프라이트와 달리 카메라 스케일을 안 탐)라
    // 1280×720 스테이지에 올려 다른 UI와 같은 비율로 스케일·정렬한다.
    let stage = spawn_stage(&mut commands, (TitleUi,));
    commands.entity(stage).with_children(|s| {
        // 최고점수(UI)
        s.spawn((
            Text::new(format!("HIGH SCORE: {}", high.0)),
            TextFont { font_size: FontSize::Px(28.0), ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(46.0),
                left: Val::Percent(38.0),
                ..default()
            },
        ));
        // 버전(우측 하단)
        s.spawn((
            Text::new(concat!("v", env!("CARGO_PKG_VERSION"))),
            TextFont { font_size: FontSize::Px(16.0), ..default() },
            TextColor(Color::srgb(0.4, 0.4, 0.4)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(3.0),
                right: Val::Percent(3.0),
                ..default()
            },
        ));
    });

    // 배경을 떠다니는 소행성 5개(중앙 회피: 좌/우 절반에 편향 배치)
    let mut rng = rand::rng();
    for i in 0..5 {
        let size = if i % 2 == 0 { AsteroidSize::Medium } else { AsteroidSize::Small };
        let image =
            if i % 2 == 0 { assets.meteor_medium.clone() } else { assets.meteor_small.clone() };
        let x = if i % 2 == 0 {
            rng.random_range(-HALF_WIDTH..-HALF_WIDTH * 0.4)
        } else {
            rng.random_range(HALF_WIDTH * 0.4..HALF_WIDTH)
        };
        let y = rng.random_range(-HALF_HEIGHT..HALF_HEIGHT);
        let vx = rng.random_range(-30.0..30.0);
        let vy = rng.random_range(-30.0..30.0);
        let spin = rng.random_range(-0.6..0.6);
        commands.spawn((
            TitleUi,
            TitleDebris,
            Sprite { image, custom_size: Some(sprite_size_for(size.radius())), ..default() },
            Transform::from_xyz(x, y, TITLE_Z - 1.0), // 배경: 타이틀 뒤
            Velocity(Vec2::new(vx, vy)),
            AngularVelocity(spin),
        ));
    }

    // 장식용 함선(인트로 동안 타이틀 라인을 좌→우로 스윕하며 글자를 "그린다")
    commands.spawn((
        TitleUi,
        TitleShip,
        Sprite { image: assets.ship.clone(), custom_size: Some(sprite_size_for(16.0)), ..default() },
        // 우주선은 타이틀 위 레이어(z+1)에서 글자를 훑는다.
        Transform::from_xyz(SHIP_START_X, TITLE_Y, TITLE_Z + 1.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
    ));

    // 인트로 동안은 메뉴가 없다(START 오발동 방지).
    *selection = MenuSelection { index: 0, count: 0 };
}

/// 인트로(스윕) 진행: 타이틀 문구를 점차 드러내고 함선을 좌→우로 이동시키며 추진 잔상을 남긴다.
/// 타이머 종료 또는 아무 키 입력으로 즉시 Ready로 전환하며, 그 순간 메뉴를 스폰한다.
pub(super) fn advance_title_intro(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut anim: ResMut<TitleAnim>,
    mut banner: Query<&mut Text2d, With<TitleBanner>>,
    mut ship: Query<&mut Transform, With<TitleShip>>,
    mut selection: ResMut<MenuSelection>,
) {
    if anim.ready {
        return;
    }

    anim.timer.tick(time.delta());

    let skip = keys.get_just_pressed().next().is_some();
    let done = anim.timer.is_finished();

    if !(skip || done) {
        let p = anim.timer.fraction();
        let n = ((p * TITLE_TEXT.len() as f32).ceil() as usize).min(TITLE_TEXT.len());
        if let Ok(mut text) = banner.single_mut() {
            *text = Text2d::new(&TITLE_TEXT[..n]);
        }
        let sx = SHIP_START_X + (SHIP_END_X - SHIP_START_X) * p;
        if let Ok(mut transform) = ship.single_mut() {
            transform.translation.x = sx;
            transform.translation.y = TITLE_Y;
        }
        return;
    }

    // 스킵 또는 타이머 종료 → Ready로 전환(한 번만).
    anim.ready = true;

    if let Ok(mut text) = banner.single_mut() {
        *text = Text2d::new(TITLE_TEXT);
    }
    // 함선은 스윕이 끝나는 지점(타이틀 오른쪽 끝)에 그대로 정박해 회전한다(텔레포트 없음).
    if let Ok(mut transform) = ship.single_mut() {
        transform.translation.x = SHIP_END_X;
        transform.translation.y = TITLE_Y;
    }

    // 웹(WASM)에선 브라우저 탭을 스크립트로 닫을 수 없어 "QUIT"(AppExit)이 화면만
    // 얼리므로 제외한다. 네이티브는 정상 종료되므로 유지. HELP는 양쪽 공통.
    #[cfg(not(target_arch = "wasm32"))]
    let items: &[(MenuAction, &str)] = &[
        (MenuAction::StartGame, "START"),
        (MenuAction::ShowHelp, "HELP"),
        (MenuAction::QuitApp, "QUIT"),
    ];
    #[cfg(target_arch = "wasm32")]
    let items: &[(MenuAction, &str)] =
        &[(MenuAction::StartGame, "START"), (MenuAction::ShowHelp, "HELP")];
    let stage = spawn_stage(&mut commands, (TitleUi,));
    commands.entity(stage).with_children(|s| {
        for (i, (action, label)) in items.iter().enumerate() {
            s.spawn((
                MenuItem { index: i, action: *action, label },
                Text::new(label.to_string()),
                TextFont { font_size: FontSize::Px(28.0), ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(56.0 + i as f32 * 8.0),
                    left: Val::Percent(45.0),
                    ..default()
                },
            ));
        }
        s.spawn((
            Text::new("UP/DOWN SELECT   ENTER CONFIRM"),
            TextFont { font_size: FontSize::Px(20.0), ..default() },
            TextColor(Color::srgb(0.45, 0.45, 0.45)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(82.0),
                left: Val::Percent(36.0),
                ..default()
            },
        ));
    });
    *selection = MenuSelection { index: 0, count: items.len() };
}

/// Ready 상태에서 타이틀 문구를 HDR 값으로 은은히 맥동시켜 Bloom과 함께 발광하게 한다.
pub(super) fn glow_title(
    time: Res<Time>,
    anim: Res<TitleAnim>,
    mut banner: Query<&mut TextColor, With<TitleBanner>>,
) {
    if !anim.ready {
        return;
    }
    let g = 1.7 + 0.5 * (time.elapsed_secs() * 2.0).sin();
    if let Ok(mut color) = banner.single_mut() {
        color.0 = Color::srgb(g * 0.9, g, g * 1.2); // 살짝 푸른 백색 발광
    }
}

/// Ready 상태에서 타이틀 오른쪽 끝에 정박한 장식용 함선을 천천히 회전시킨다.
pub(super) fn rotate_title_ship(
    time: Res<Time>,
    anim: Res<TitleAnim>,
    mut ship: Query<&mut Transform, With<TitleShip>>,
) {
    if !anim.ready {
        return;
    }
    if let Ok(mut transform) = ship.single_mut() {
        transform.rotate_z(time.delta_secs() * 0.6);
    }
}

/// Ready 상태에서 현재 선택된 메뉴 항목을 숨쉬듯 밝기 펄스시킨다.
/// `highlight_menu` 이후에 실행되어야 베이스 색을 덮어쓸 수 있다.
pub(super) fn pulse_title_selection(
    time: Res<Time>,
    anim: Res<TitleAnim>,
    selection: Res<MenuSelection>,
    mut items: Query<(&MenuItem, &mut TextColor)>,
) {
    if !anim.ready {
        return;
    }
    let breathe = 0.75 + 0.25 * (time.elapsed_secs() * 3.0).sin();
    for (item, mut color) in &mut items {
        if item.index == selection.index {
            color.0 = Color::srgb(1.5 * breathe, 1.2 * breathe, 0.3 * breathe);
        }
    }
}

/// 배경 소행성을 인트로/Ready 양쪽 상태에서 계속 표류시킨다.
pub(super) fn drift_title_debris(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity, &AngularVelocity), With<TitleDebris>>,
) {
    let dt = time.delta_secs();
    let half = Vec2::new(HALF_WIDTH, HALF_HEIGHT);
    for (mut transform, velocity, angular) in &mut query {
        let pos = transform.translation.truncate() + velocity.0 * dt;
        let wrapped = wrap_position(pos, half);
        transform.translation.x = wrapped.x;
        transform.translation.y = wrapped.y;
        transform.rotate_z(angular.0 * dt);
    }
}

/// 인트로 스윕 중, 함선 살짝 뒤에 짧게 페이드하는 발광 스파크(추진 잔상)를 매 프레임 남긴다.
pub(super) fn emit_title_trail(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    anim: Res<TitleAnim>,
    ship: Query<&Transform, With<TitleShip>>,
) {
    if anim.ready {
        return;
    }
    if let Ok(tf) = ship.single() {
        commands.spawn((
            TitleUi,
            TitleTrail { life: Timer::from_seconds(0.35, TimerMode::Once) },
            Sprite {
                image: assets.spark.clone(),
                custom_size: Some(Vec2::splat(12.0)),
                color: Color::srgb(1.6, 1.1, 0.5), // 따뜻한 발광 잔상(HDR)
                ..default()
            },
            // 화염(잔상)은 글자 위(z+0.5), 우주선 아래에 그려진다.
            Transform::from_xyz(tf.translation.x - 14.0, tf.translation.y, TITLE_Z + 0.5),
        ));
    }
}

/// 추진 잔상을 수명 동안 서서히 사라지게 하고 만료 시 제거한다.
pub(super) fn fade_title_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut TitleTrail, &mut Sprite)>,
) {
    for (entity, mut trail, mut sprite) in &mut q {
        trail.life.tick(time.delta());
        if trail.life.is_finished() {
            commands.entity(entity).despawn();
        } else {
            sprite.color = sprite.color.with_alpha(1.0 - trail.life.fraction());
        }
    }
}

/// HELP가 열려 있는 동안 타이틀 요소를 숨겨 헬프 텍스트만 보이게 한다.
/// 부유 소행성(`TitleDebris`)만 남기고 나머지 `TitleUi`(발광 문구·함선·추진 잔상 +
/// HIGH SCORE·메뉴·버전·힌트 UI)를 모두 숨긴다. UI 텍스트는 스테이지 컨테이너
/// (`TitleUi`)를 숨기면 자식이 상속으로 함께 사라진다. 우주 배경은 `TitleUi`가
/// 아니라 그대로 보인다. 닫으면 기본값 Inherited로 되돌린다.
pub(super) fn hide_title_while_help(
    help: Res<HelpOpen>,
    mut q: Query<&mut Visibility, (With<TitleUi>, Without<TitleDebris>)>,
) {
    let want = if help.0 { Visibility::Hidden } else { Visibility::Inherited };
    for mut vis in &mut q {
        if *vis != want {
            *vis = want;
        }
    }
}

pub(super) fn despawn_title(mut commands: Commands, query: Query<Entity, With<TitleUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TitleAnim>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::core::state::{GameState, GameStatePlugin};
    use crate::fx::sprites::dummy_sprite_assets;

    #[test]
    fn boots_to_title() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(GameStatePlugin);
        app.update();
        assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::Title);
    }

    fn build_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(dummy_sprite_assets());
        app.insert_resource(MenuSelection::default());
        app.insert_resource(Time::<()>::default());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app
    }

    fn insert_high_score(app: &mut App, dir_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(dir_name);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("highscore.json");

        let hs = Persistent::<HighScore>::builder()
            .name(dir_name)
            .format(StorageFormat::Json)
            .path(path)
            .default(HighScore(0))
            .build()
            .unwrap();
        app.insert_resource(hs);
        dir
    }

    #[test]
    fn spawn_title_starts_intro_with_no_menu_yet() {
        let mut app = build_test_app();
        let dir = insert_high_score(&mut app, "bevy-asteroids-spawn-title-test");

        app.world_mut().run_system_once(spawn_title).unwrap();

        // 인트로 동안은 메뉴 항목이 없어야 한다(오발동 방지).
        let item_count = app.world_mut().query::<&MenuItem>().iter(app.world()).count();
        assert_eq!(item_count, 0);

        let selection = app.world().resource::<MenuSelection>();
        assert_eq!(selection.index, 0);
        assert_eq!(selection.count, 0);

        // 타이틀 배너는 존재해야 한다(빈 문자열로 시작).
        let banner_count = app.world_mut().query::<&TitleBanner>().iter(app.world()).count();
        assert_eq!(banner_count, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn intro_skip_spawns_menu_items_with_expected_actions() {
        let mut app = build_test_app();
        let dir = insert_high_score(&mut app, "bevy-asteroids-intro-skip-test");

        app.world_mut().run_system_once(spawn_title).unwrap();

        // 아무 키나 누르면 인트로를 즉시 스킵하고 Ready로 전환된다.
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
        app.world_mut().run_system_once(advance_title_intro).unwrap();

        let mut items: Vec<(usize, MenuAction)> = app
            .world_mut()
            .query::<&MenuItem>()
            .iter(app.world())
            .map(|item| (item.index, item.action))
            .collect();
        items.sort_by_key(|(index, _)| *index);

        // 네이티브 테스트 타깃: START / HELP / QUIT 3항목(웹에선 QUIT 제외).
        assert_eq!(
            items,
            vec![
                (0, MenuAction::StartGame),
                (1, MenuAction::ShowHelp),
                (2, MenuAction::QuitApp)
            ]
        );

        let selection = app.world().resource::<MenuSelection>();
        assert_eq!(selection.index, 0);
        assert_eq!(selection.count, 3);

        let anim = app.world().resource::<TitleAnim>();
        assert!(anim.ready);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
