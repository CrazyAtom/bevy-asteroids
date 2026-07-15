//! 산탄 노바: 우주선 주변 전방위로 아군 탄을 방출한다(포위 탈출기).

use bevy::prelude::*;

use crate::core::config::NOVA_BULLETS;
use crate::entities::bullet::spawn_bullet;
use crate::fx::sprites::SpriteAssets;

/// n개 균등 각도의 단위 방향 벡터.
pub(super) fn nova_directions(n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let ang = i as f32 / n as f32 * std::f32::consts::TAU;
            Vec2::new(ang.cos(), ang.sin())
        })
        .collect()
}

/// 전방위 NOVA_BULLETS발. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, pos: Vec2) {
    for dir in nova_directions(NOVA_BULLETS) {
        spawn_bullet(commands, assets, pos, dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nova_directions_are_unit_and_distinct() {
        let dirs = nova_directions(16);
        assert_eq!(dirs.len(), 16);
        for d in &dirs {
            assert!((d.length() - 1.0).abs() < 1e-4);
        }
        // 첫 방향과 반대편(8번째)은 반대 방향
        assert!((dirs[0] + dirs[8]).length() < 1e-3);
    }
}
