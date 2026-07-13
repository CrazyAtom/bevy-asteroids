use bevy::prelude::*;

use crate::core::config::{WINDOW_HEIGHT, WINDOW_WIDTH, Z_BACKGROUND};
use crate::fx::sprites::SpriteAssets;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        // SpriteAssets는 PreStartup에서 로드되므로 Startup 시점엔 이미 존재한다.
        app.add_systems(Startup, spawn_background);
    }
}

fn spawn_background(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        Sprite {
            image: assets.background.clone(),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
    ));
}
