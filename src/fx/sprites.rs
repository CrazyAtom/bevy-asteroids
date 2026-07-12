use bevy::prelude::*;

use crate::core::config::{EXPLOSION_FRAME_COUNT, VISUAL_FIT};

/// 게임의 모든 스프라이트 이미지 핸들. Startup에 1회 로드해 보관한다.
#[derive(Resource)]
pub struct SpriteAssets {
    pub ship: Handle<Image>,
    pub flame: Handle<Image>,
    pub meteor_large: Handle<Image>,
    pub meteor_medium: Handle<Image>,
    pub meteor_small: Handle<Image>,
    pub ufo_large: Handle<Image>,
    pub ufo_small: Handle<Image>,
    pub bullet: Handle<Image>,
    pub enemy_bullet: Handle<Image>,
    pub beam: Handle<Image>,
    pub spark: Handle<Image>,
    pub shield: Handle<Image>,
    pub background: Handle<Image>,
    pub powerup: [Handle<Image>; 5], // Shield, RapidFire, Spread, ExtraLife, SpecialWeapon
    pub explosion_frames: Vec<Handle<Image>>,
}

/// 콜라이더 반경에 맞춘 정사각 스프라이트 표시 크기.
pub fn sprite_size_for(radius: f32) -> Vec2 {
    Vec2::splat(radius * 2.0 * VISUAL_FIT)
}

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        // AssetServer는 DefaultPlugins(AssetPlugin)에서 이미 삽입됨. 빌드 시점에
        // 핸들을 로드해 리소스로 즉시 넣으면, 어떤 스케줄(Startup·OnEnter·Update)
        // 보다도 먼저 존재가 보장되어 리소스 순서 문제가 원천 차단된다.
        let asset_server = app.world().resource::<AssetServer>().clone();
        app.insert_resource(build_sprite_assets(&asset_server));
    }
}

fn build_sprite_assets(asset_server: &AssetServer) -> SpriteAssets {
    let explosion_frames: Vec<Handle<Image>> = (0..EXPLOSION_FRAME_COUNT)
        .map(|i| asset_server.load(format!("sprites/explosion_{i}.png")))
        .collect();
    SpriteAssets {
        ship: asset_server.load("sprites/ship.png"),
        flame: asset_server.load("sprites/flame.png"),
        meteor_large: asset_server.load("sprites/meteor_large.png"),
        meteor_medium: asset_server.load("sprites/meteor_medium.png"),
        meteor_small: asset_server.load("sprites/meteor_small.png"),
        ufo_large: asset_server.load("sprites/ufo_large.png"),
        ufo_small: asset_server.load("sprites/ufo_small.png"),
        bullet: asset_server.load("sprites/bullet.png"),
        enemy_bullet: asset_server.load("sprites/enemy_bullet.png"),
        beam: asset_server.load("sprites/beam.png"),
        spark: asset_server.load("sprites/spark.png"),
        shield: asset_server.load("sprites/shield.png"),
        background: asset_server.load("sprites/background.png"),
        powerup: [
            asset_server.load("sprites/powerup_shield.png"),
            asset_server.load("sprites/powerup_rapid.png"),
            asset_server.load("sprites/powerup_spread.png"),
            asset_server.load("sprites/powerup_life.png"),
            asset_server.load("sprites/powerup_special.png"),
        ],
        explosion_frames,
    }
}

/// 테스트에서 `SpriteAssets`가 필요한 스폰 시스템을 돌릴 때 쓰는 더미 리소스.
#[cfg(test)]
pub fn dummy_sprite_assets() -> SpriteAssets {
    let h = Handle::<Image>::default();
    SpriteAssets {
        ship: h.clone(),
        flame: h.clone(),
        meteor_large: h.clone(),
        meteor_medium: h.clone(),
        meteor_small: h.clone(),
        ufo_large: h.clone(),
        ufo_small: h.clone(),
        bullet: h.clone(),
        enemy_bullet: h.clone(),
        beam: h.clone(),
        spark: h.clone(),
        shield: h.clone(),
        background: h.clone(),
        powerup: [h.clone(), h.clone(), h.clone(), h.clone(), h.clone()],
        explosion_frames: vec![h.clone(); 6],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_size_scales_with_radius() {
        let small = sprite_size_for(14.0);
        let large = sprite_size_for(45.0);
        assert!(large.x > small.x && large.y > small.y);
        // 정사각형, 반경*2*VISUAL_FIT
        assert!((small.x - 14.0 * 2.0 * VISUAL_FIT).abs() < 1e-3);
        assert_eq!(small.x, small.y);
    }
}
