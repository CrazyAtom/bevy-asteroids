use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;

use crate::core::state::{GameState, GameplayEntity, HighScore, Lives, Score};
use crate::entities::player::{Player, RapidFire, Shield, Spread, SpecialWeapon};
use crate::entities::boss::Boss;
use crate::fx::sprites::SpriteAssets;
use crate::systems::stage::{theme_name, Progression};

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct BossHealthBar;

#[derive(Component)]
struct BossHealthFill;

/// 목숨 하트 아이콘(인덱스). index < lives 이면 표시.
#[derive(Component)]
struct LifeIcon(usize);

/// 특수무기 충전 수 텍스트.
#[derive(Component)]
struct SpChargeText;

/// 활성 파워업 배지 아이콘(0=실드,1=연사,2=확산). 활성 시만 표시.
#[derive(Component)]
struct ModIcon(u8);

const MAX_HEART_ICONS: usize = 8;
const HUD_ICON_PX: f32 = 22.0;

#[derive(Component)]
struct GameOverScreen;

/// 웨이브가 오를 때 화면 중앙에 잠깐 떴다 사라지는 "WAVE N" 배너.
#[derive(Component)]
struct WaveBanner {
    life: Timer,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastStage>()
            .add_systems(OnEnter(GameState::Playing), (spawn_hud, reset_last_stage))
            .add_systems(
                Update,
                (
                    update_hud,
                    update_life_icons,
                    update_mod_icons,
                    announce_stage,
                    announce_boss,
                    wave_banner_lifetime,
                    update_boss_bar,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(Update, restart_input.run_if(in_state(GameState::GameOver)));
    }
}

fn spawn_hud(mut commands: Commands, assets: Res<SpriteAssets>) {
    let icon = || Node {
        width: Val::Px(HUD_ICON_PX),
        height: Val::Px(HUD_ICON_PX),
        ..default()
    };
    let row = || Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(3.0),
        ..default()
    };
    commands
        .spawn((
            GameplayEntity,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Score / High / Stage 텍스트
            root.spawn((
                Hud,
                Text::new("Score: 0   High: 0   Stage 1-1"),
                TextFont { font_size: FontSize::Px(22.0), ..default() },
                TextColor(Color::WHITE),
            ));
            // 목숨: 하트 나열
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(1.0),
                ..default()
            })
            .with_children(|hearts| {
                for i in 0..MAX_HEART_ICONS {
                    hearts.spawn((LifeIcon(i), ImageNode::new(assets.powerup[3].clone()), icon()));
                }
            });
            // 특수무기: 아이콘 + 충전 수
            root.spawn(row()).with_children(|sp| {
                sp.spawn((ImageNode::new(assets.powerup[4].clone()), icon()));
                sp.spawn((
                    SpChargeText,
                    Text::new("0"),
                    TextFont { font_size: FontSize::Px(22.0), ..default() },
                    TextColor(Color::WHITE),
                ));
            });
            // 활성 파워업 배지(실드/연사/확산; 활성 시만 표시)
            root.spawn(row()).with_children(|mods| {
                for i in 0..3u8 {
                    mods.spawn((
                        ModIcon(i),
                        ImageNode::new(assets.powerup[i as usize].clone()),
                        icon(),
                        Visibility::Hidden,
                    ));
                }
            });
        });

    // 보스 체력 바(기본 숨김; 보스 존재 시 표시)
    commands
        .spawn((
            BossHealthBar,
            GameplayEntity,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(46.0),
                left: Val::Percent(25.0),
                width: Val::Percent(50.0),
                height: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.8)),
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((
                BossHealthFill,
                Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(Color::srgb(1.0, 0.28, 0.28)),
            ));
        });
}

/// 보스가 있으면 체력 바를 표시하고 폭을 체력 비율로 갱신, 없으면 숨긴다.
fn update_boss_bar(
    bosses: Query<&Boss>,
    mut bar: Query<&mut Visibility, With<BossHealthBar>>,
    mut fill: Query<&mut Node, With<BossHealthFill>>,
) {
    let boss = bosses.iter().next();
    for mut vis in &mut bar {
        *vis = if boss.is_some() { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Some(boss) = boss {
        let frac = (boss.health / boss.max_health).clamp(0.0, 1.0);
        for mut node in &mut fill {
            node.width = Val::Percent(frac * 100.0);
        }
    }
}

fn update_hud(
    score: Res<Score>,
    high: Res<Persistent<HighScore>>,
    prog: Res<Progression>,
    player: Query<&SpecialWeapon, With<Player>>,
    mut hud: Query<&mut Text, (With<Hud>, Without<SpChargeText>)>,
    mut sp: Query<&mut Text, (With<SpChargeText>, Without<Hud>)>,
) {
    for mut text in &mut hud {
        text.0 = format!(
            "Score: {}   High: {}   Stage {}-{}",
            score.0, high.0, prog.cycle + 1, prog.stage_in_cycle + 1
        );
    }
    let charges = player.single().map(|w| w.charges).unwrap_or(0);
    for mut text in &mut sp {
        text.0 = format!("{charges}");
    }
}

/// 목숨 수만큼 하트 아이콘을 표시한다.
fn update_life_icons(lives: Res<Lives>, mut q: Query<(&LifeIcon, &mut Visibility)>) {
    for (icon, mut vis) in &mut q {
        *vis = if (icon.0 as u32) < lives.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 활성 파워업 배지 아이콘을 표시/숨김한다(실드/연사/확산).
#[allow(clippy::type_complexity)]
fn update_mod_icons(
    player: Query<(Has<Shield>, Has<RapidFire>, Has<Spread>), With<Player>>,
    mut icons: Query<(&ModIcon, &mut Visibility)>,
) {
    let (sh, rf, sp) = player.single().unwrap_or((false, false, false));
    for (icon, mut vis) in &mut icons {
        let on = match icon.0 {
            0 => sh,
            1 => rf,
            _ => sp,
        };
        *vis = if on { Visibility::Visible } else { Visibility::Hidden };
    }
}

#[derive(Resource, Default)]
struct LastStage(Option<(u32, usize)>);

/// 재시작 시 스테이지 배너 중복 억제 상태를 초기화한다(Local이 아니라 리소스라 리셋 가능).
fn reset_last_stage(mut last: ResMut<LastStage>) {
    last.0 = None;
}

/// 스테이지(사이클·순번)가 바뀌면 "STAGE c-s 테마명" 배너를 잠깐 띄운다.
fn announce_stage(
    mut commands: Commands,
    prog: Res<Progression>,
    mut last: ResMut<LastStage>,
    existing: Query<Entity, With<WaveBanner>>,
) {
    let key = (prog.cycle, prog.stage_in_cycle);
    if last.0 == Some(key) {
        return;
    }
    last.0 = Some(key);
    // 이전 배너가 남아 있으면 제거해 중첩 방지
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        WaveBanner { life: Timer::from_seconds(1.8, TimerMode::Once) },
        GameplayEntity,
        Text::new(format!(
            "STAGE {}-{}  {}",
            prog.cycle + 1,
            prog.stage_in_cycle + 1,
            theme_name(prog.current_theme())
        )),
        TextFont { font_size: FontSize::Px(44.0), ..default() },
        TextColor(Color::srgb(0.4, 0.9, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(32.0),
            ..default()
        },
    ));
}

/// 수명이 다한 웨이브 배너를 제거한다.
fn wave_banner_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut WaveBanner)>,
) {
    for (entity, mut banner) in &mut query {
        banner.life.tick(time.delta());
        if banner.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// 보스가 등장한 프레임에 "⚠ BOSS" 배너를 잠깐 띄운다.
fn announce_boss(
    mut commands: Commands,
    bosses: Query<(), Added<Boss>>,
    existing: Query<Entity, With<WaveBanner>>,
) {
    if bosses.is_empty() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        WaveBanner { life: Timer::from_seconds(1.5, TimerMode::Once) },
        GameplayEntity,
        Text::new("! BOSS !"),
        TextFont { font_size: FontSize::Px(52.0), ..default() },
        TextColor(Color::srgb(1.0, 0.35, 0.35)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(28.0),
            left: Val::Percent(40.0),
            ..default()
        },
    ));
}

fn spawn_game_over(mut commands: Commands, score: Res<Score>, high: Res<Persistent<HighScore>>) {
    commands.spawn((
        GameOverScreen,
        Text::new(format!(
            "GAME OVER\nScore: {}\nHigh: {}\nPress R to restart",
            score.0, high.0
        )),
        TextFont { font_size: FontSize::Px(40.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(34.0),
            ..default()
        },
    ));
}

fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn restart_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn hud_shows_score_lives_wave() {
        let mut app = App::new();
        app.insert_resource(Score(150));
        app.insert_resource(Lives(2));
        app.insert_resource(crate::systems::stage::new_progression());
        let hs = Persistent::<HighScore>::builder()
            .name("test high score")
            .format(StorageFormat::Json)
            .path(std::env::temp_dir().join("bevy-asteroids-test").join("highscore.json"))
            .default(HighScore(0))
            .build()
            .unwrap();
        app.insert_resource(hs);
        let e = app.world_mut().spawn((Hud, Text::new("초기값"))).id();
        app.world_mut().run_system_once(update_hud).unwrap();
        let text = app.world().entity(e).get::<Text>().unwrap();
        assert!(text.0.contains("150"));
        assert!(text.0.contains("Stage 1-1"));
        assert!(text.0.contains("High: 0"));
    }

    #[test]
    fn wave_banner_expires() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(1.5, TimerMode::Once);
        timer.tick(std::time::Duration::from_secs_f32(2.0)); // 이미 만료
        let e = app.world_mut().spawn(WaveBanner { life: timer }).id();
        app.world_mut().run_system_once(wave_banner_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
