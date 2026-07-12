use bevy::prelude::*;

use crate::core::config::{WINDOW_HEIGHT, WINDOW_WIDTH, Z_BACKGROUND};
use crate::fx::sprites::{load_sprite_assets, SpriteAssets};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        // SpriteAssets가 먼저 삽입된 뒤 배경을 스폰한다.
        app.add_systems(Startup, spawn_background.after(load_sprite_assets));
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
