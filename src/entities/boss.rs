use bevy::prelude::*;

use crate::core::components::Collider;
use crate::core::config::{
    BEAM_BOSS_DAMAGE, BEAM_LENGTH, BEAM_WIDTH, BOSS_BASE_HEALTH, BOSS_HEALTH_PER_CYCLE,
    BOSS_SCORE_BONUS, BULLET_BOSS_DAMAGE, EXPLOSION_PARTICLES, SHAKE_EXPLOSION, Z_ENTITY,
};
use crate::core::logic::{circles_overlap, segment_circle_hit};
use crate::core::state::{GameState, GameplayEntity, Score};
use crate::entities::bullet::Bullet;
use crate::entities::player::SpecialBeam;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::effects::spawn_explosion;
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;
use crate::systems::stage::{advance_stage, spawn_first_wave_of_stage, Progression, ThemeId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    MotherRock,
    Mothership,
    BlazingCore,
}

pub fn boss_for_theme(t: ThemeId) -> BossKind {
    match t {
        ThemeId::AsteroidBelt => BossKind::MotherRock,
        ThemeId::AlienFleet => BossKind::Mothership,
        ThemeId::SolarFlare => BossKind::BlazingCore,
    }
}

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
}

/// 보스 종류별 기준 체력(모암이 가장 높음) × 사이클 증가.
pub fn boss_max_health(kind: BossKind, cycle: u32) -> f32 {
    let base = match kind {
        BossKind::MotherRock => BOSS_BASE_HEALTH * 1.5,
        BossKind::Mothership => BOSS_BASE_HEALTH,
        BossKind::BlazingCore => BOSS_BASE_HEALTH * 1.2,
    };
    base + cycle as f32 * BOSS_HEALTH_PER_CYCLE
}

/// 체력을 깎고, 0 이하이면 true(격파).
pub fn apply_boss_damage(boss: &mut Boss, dmg: f32) -> bool {
    boss.health -= dmg;
    boss.health <= 0.0
}

pub fn boss_image(kind: BossKind, assets: &SpriteAssets) -> Handle<Image> {
    match kind {
        BossKind::MotherRock => assets.boss_mother_rock.clone(),
        BossKind::Mothership => assets.boss_mothership.clone(),
        BossKind::BlazingCore => assets.boss_blazing_core.clone(),
    }
}

pub fn boss_radius(kind: BossKind) -> f32 {
    match kind {
        BossKind::MotherRock => 55.0,
        BossKind::Mothership => 70.0,
        BossKind::BlazingCore => 45.0,
    }
}

pub fn spawn_boss(commands: &mut Commands, assets: &SpriteAssets, kind: BossKind, cycle: u32) {
    let hp = boss_max_health(kind, cycle);
    let r = boss_radius(kind);
    commands.spawn((
        Boss { kind, health: hp, max_health: hp },
        Sprite {
            image: boss_image(kind, assets),
            custom_size: Some(Vec2::splat(r * 2.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 120.0, Z_ENTITY),
        Collider { radius: r },
        GameplayEntity,
    ));
}

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (boss_movement, boss_combat).run_if(in_state(GameState::Playing)),
        );
    }
}

/// 기본 이동(느린 좌우 스윕). Task 5~7에서 kind별로 확장.
fn boss_movement(time: Res<Time>, mut q: Query<&mut Transform, With<Boss>>) {
    let t = time.elapsed_secs();
    for mut tf in &mut q {
        tf.translation.x = (t * 0.5).sin() * 220.0;
    }
}

/// 총알/빔이 보스 체력을 깎고, 격파 시 폭발·보너스·다음 스테이지로.
#[allow(clippy::too_many_arguments)]
fn boss_combat(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut prog: ResMut<Progression>,
    mut shake: MessageWriter<ShakeEvent>,
    mut sfx: MessageWriter<SfxEvent>,
    mut bosses: Query<(Entity, &mut Boss, &Transform, &Collider)>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    beams: Query<&SpecialBeam>,
) {
    for (boss_e, mut boss, boss_tf, boss_col) in &mut bosses {
        let bpos = boss_tf.translation.truncate();
        let mut defeated = false;
        for (be, btf, bcol) in &bullets {
            if circles_overlap(bpos, boss_col.radius, btf.translation.truncate(), bcol.radius) {
                commands.entity(be).despawn();
                if apply_boss_damage(&mut boss, BULLET_BOSS_DAMAGE) {
                    defeated = true;
                    break;
                }
            }
        }
        if !defeated {
            for beam in &beams {
                if segment_circle_hit(beam.origin, beam.dir, BEAM_LENGTH, BEAM_WIDTH * 0.5, bpos, boss_col.radius)
                    && apply_boss_damage(&mut boss, BEAM_BOSS_DAMAGE)
                {
                    defeated = true;
                    break;
                }
            }
        }
        if defeated {
            commands.entity(boss_e).despawn();
            spawn_explosion(&mut commands, &assets, bpos, EXPLOSION_PARTICLES * 3);
            shake.write(ShakeEvent(SHAKE_EXPLOSION * 2.0));
            sfx.write(SfxEvent(Sfx::Explosion));
            score.0 += BOSS_SCORE_BONUS;
            advance_stage(&mut prog);
            spawn_first_wave_of_stage(&mut commands, &assets, &prog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::stage::ThemeId;

    #[test]
    fn boss_kind_matches_theme() {
        assert_eq!(boss_for_theme(ThemeId::AsteroidBelt), BossKind::MotherRock);
        assert_eq!(boss_for_theme(ThemeId::AlienFleet), BossKind::Mothership);
        assert_eq!(boss_for_theme(ThemeId::SolarFlare), BossKind::BlazingCore);
    }

    #[test]
    fn health_scales_with_cycle() {
        let h0 = boss_max_health(BossKind::MotherRock, 0);
        let h2 = boss_max_health(BossKind::MotherRock, 2);
        assert!(h2 > h0);
    }

    #[test]
    fn damage_reduces_and_defeats() {
        let mut b = Boss { kind: BossKind::Mothership, health: 3.0, max_health: 10.0 };
        assert!(!apply_boss_damage(&mut b, 1.0)); // 2 남음
        assert!((b.health - 2.0).abs() < 1e-6);
        assert!(apply_boss_damage(&mut b, 5.0)); // 0 이하 → 격파
    }
}
