use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;

use crate::core::state::{GameState, GameplayEntity, HighScore, Lives, Score};
use crate::entities::player::{Player, RapidFire, Shield, Spread, SpecialWeapon};
use crate::systems::stage::{theme_name, Progression};

#[derive(Component)]
struct Hud;

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
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(
                Update,
                (update_hud, announce_stage, wave_banner_lifetime)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(Update, restart_input.run_if(in_state(GameState::GameOver)));
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Hud,
        GameplayEntity,
        Text::new("Score: 0   Lives: 3   Wave: 1"),
        TextFont { font_size: FontSize::Px(24.0), ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

#[allow(clippy::type_complexity)]
fn update_hud(
    score: Res<Score>,
    lives: Res<Lives>,
    high: Res<Persistent<HighScore>>,
    prog: Res<Progression>,
    player: Query<(Option<&SpecialWeapon>, Option<&Shield>, Option<&RapidFire>, Option<&Spread>), With<Player>>,
    mut query: Query<&mut Text, With<Hud>>,
) {
    let (charges, mods) = if let Ok((sw, sh, rf, sp)) = player.single() {
        let charges = sw.map(|w| w.charges).unwrap_or(0);
        let mut mods = String::new();
        if sh.is_some() { mods.push_str(" [실드]"); }
        if rf.is_some() { mods.push_str(" [연사]"); }
        if sp.is_some() { mods.push_str(" [확산]"); }
        (charges, mods)
    } else {
        (0, String::new())
    };
    for mut text in &mut query {
        text.0 = format!(
            "Score: {}   Lives: {}   Stage {}-{}   High: {}   특수: {}{}",
            score.0, lives.0, prog.cycle + 1, prog.stage_in_cycle + 1, high.0, charges, mods
        );
    }
}

/// 스테이지(사이클·순번)가 바뀌면 "STAGE c-s 테마명" 배너를 잠깐 띄운다.
fn announce_stage(
    mut commands: Commands,
    prog: Res<Progression>,
    mut last: Local<Option<(u32, usize)>>,
    existing: Query<Entity, With<WaveBanner>>,
) {
    let key = (prog.cycle, prog.stage_in_cycle);
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
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
        assert!(text.0.contains("Lives: 2"));
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
