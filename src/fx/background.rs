use bevy::prelude::*;

use crate::core::config::{WINDOW_HEIGHT, WINDOW_WIDTH, Z_BACKGROUND};
use crate::core::state::GameState;
use crate::fx::sprites::SpriteAssets;
use crate::systems::stage::{Progression, ThemeId};

#[derive(Component)]
struct BackgroundSprite;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_background)
            .add_systems(Update, update_theme_background.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_background(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        BackgroundSprite,
        Sprite {
            image: assets.background.clone(),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
    ));
}

fn theme_bg(t: ThemeId, assets: &SpriteAssets) -> Handle<Image> {
    match t {
        ThemeId::AsteroidBelt => assets.bg_belt.clone(),
        ThemeId::AlienFleet => assets.bg_fleet.clone(),
        ThemeId::SolarFlare => assets.bg_flare.clone(),
    }
}

/// 현재 테마 배경이 아니면 교체한다(상태 비의존 → 재시작 시에도 자동 교정).
fn update_theme_background(
    prog: Res<Progression>,
    assets: Res<SpriteAssets>,
    mut q: Query<&mut Sprite, With<BackgroundSprite>>,
) {
    let want = theme_bg(prog.current_theme(), &assets);
    for mut sprite in &mut q {
        if sprite.image != want {
            sprite.image = want.clone();
        }
    }
}
