//! 특수무기(X 키). 발동 시 우주선 정면으로 레이저 빔을 쏜다.
//! 충전(charges)을 소모하며, 빔 판정은 `systems::collision::beam_vs_targets`가,
//! 보스 피해는 `entities::boss::boss_combat`이 담당한다. 여기서는 발동과 수명만 다룬다.

use bevy::prelude::*;

use crate::core::config::{
    BEAM_LENGTH, BEAM_LIFETIME_SECS, BEAM_WIDTH, SHAKE_SPECIAL, Z_BEAM,
};
use crate::core::state::{GameState, GameplayEntity};
use crate::entities::player::Player;
use crate::entities::powerup::SpecialWeaponKind;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

/// 우주선이 보유한 특수무기와 남은 충전 수. `SpecialWeaponKind`로 종류를 확장한다.
#[derive(Component)]
pub struct SpecialWeapon {
    pub kind: SpecialWeaponKind,
    pub charges: u32,
}

/// 발사된 레이저 빔. `origin`에서 `dir` 방향으로 뻗는 선분으로 판정한다.
#[derive(Component)]
pub struct SpecialBeam {
    pub life: Timer,
    pub origin: Vec2,
    pub dir: Vec2,
    /// 보스에게는 프레임당이 아니라 빔 1회당 한 번만 피해를 준다.
    pub damaged_boss: bool,
}

pub struct SpecialWeaponPlugin;

impl Plugin for SpecialWeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (activate_special, tick_beam).run_if(in_state(GameState::Playing)),
        );
    }
}

fn activate_special(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut shake: MessageWriter<ShakeEvent>,
    mut query: Query<(&Transform, &mut SpecialWeapon), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let Ok((transform, mut weapon)) = query.single_mut() else { return };
    if weapon.charges == 0 {
        return;
    }
    weapon.charges -= 1;
    let origin = transform.translation.truncate();
    let dir = (transform.rotation * Vec3::Y).truncate();
    match weapon.kind {
        SpecialWeaponKind::LaserBeam => {
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
    }
    sfx.write(SfxEvent(Sfx::Special));
    shake.write(ShakeEvent(SHAKE_SPECIAL));
}

/// 빔 수명 관리(그리기는 스프라이트가 담당). 판정은 collision::beam_vs_targets.
fn tick_beam(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut SpecialBeam)>) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
