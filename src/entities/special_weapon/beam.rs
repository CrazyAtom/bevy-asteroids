//! 레이저 빔(기존 특수무기). 판정은 systems::collision::beam_vs_targets,
//! 보스 피해는 entities::boss::boss_combat이 담당한다. 여기서는 발사와 수명만.

use bevy::prelude::*;

use crate::core::config::{BEAM_LENGTH, BEAM_LIFETIME_SECS, BEAM_WIDTH, Z_BEAM};
use crate::core::state::GameplayEntity;
use crate::fx::sprites::SpriteAssets;

/// 발사된 레이저 빔. `origin`에서 `dir` 방향으로 뻗는 선분으로 판정한다.
#[derive(Component)]
pub struct SpecialBeam {
    pub life: Timer,
    pub origin: Vec2,
    pub dir: Vec2,
    /// 보스에게는 프레임당이 아니라 빔 1회당 한 번만 피해를 준다.
    pub damaged_boss: bool,
}

/// 우주선 정면으로 빔을 스폰한다. 디스패치(activate_special)에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, transform: &Transform) {
    let origin = transform.translation.truncate();
    let dir = (transform.rotation * Vec3::Y).truncate();
    // 빔 스프라이트는 세로(+Y)로 그려짐 → dir 방향으로 회전, 원점~사거리 중앙에 배치.
    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    let center = origin + dir.normalize_or_zero() * (BEAM_LENGTH * 0.5);
    commands.spawn((
        SpecialBeam {
            life: Timer::from_seconds(BEAM_LIFETIME_SECS, TimerMode::Once),
            origin,
            dir,
            damaged_boss: false,
        },
        Sprite {
            image: assets.beam.clone(),
            custom_size: Some(Vec2::new(BEAM_WIDTH, BEAM_LENGTH)),
            ..default()
        },
        Transform {
            translation: center.extend(Z_BEAM),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        GameplayEntity,
    ));
}

/// 빔 수명 관리(그리기는 스프라이트가 담당). 판정은 collision::beam_vs_targets.
pub(super) fn tick_beam(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut SpecialBeam)>) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
