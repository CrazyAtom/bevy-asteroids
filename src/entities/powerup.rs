use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{Collider, Velocity};
use crate::core::config::{
    POWERUP_DRIFT_SPEED, POWERUP_LIFETIME_SECS, POWERUP_RADIUS, RAPID_FIRE_SECS, SHIELD_SECS,
    SPREAD_SECS,
};
use crate::core::state::{GameState, GameplayEntity, Lives};
use crate::entities::player::{Player, RapidFire, Shield, Spread};
use crate::fx::audio::{Sfx, SfxEvent};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpecialWeaponKind {
    LaserBeam,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerupKind {
    Shield,
    RapidFire,
    Spread,
    ExtraLife,
    SpecialWeapon(SpecialWeaponKind),
}

#[derive(Component)]
pub struct Powerup {
    pub kind: PowerupKind,
    pub life: Timer,
}

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (powerup_lifetime, draw_powerups, collect_powerup).run_if(in_state(GameState::Playing)),
        );
    }
}

/// roll(0..1)을 5개 버킷으로 나눠 종류를 고른다(특수무기는 낮은 확률).
pub fn pick_powerup_kind(roll: f32) -> PowerupKind {
    match roll {
        r if r < 0.25 => PowerupKind::Shield,
        r if r < 0.50 => PowerupKind::RapidFire,
        r if r < 0.72 => PowerupKind::Spread,
        r if r < 0.88 => PowerupKind::ExtraLife,
        _ => PowerupKind::SpecialWeapon(SpecialWeaponKind::LaserBeam),
    }
}

pub fn spawn_powerup(commands: &mut Commands, kind: PowerupKind, position: Vec2) {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let velocity = Vec2::new(angle.cos(), angle.sin()) * POWERUP_DRIFT_SPEED;
    commands.spawn((
        Powerup { kind, life: Timer::from_seconds(POWERUP_LIFETIME_SECS, TimerMode::Once) },
        Transform::from_translation(position.extend(0.0)),
        Velocity(velocity),
        Collider { radius: POWERUP_RADIUS },
        GameplayEntity,
    ));
}

fn powerup_lifetime(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Powerup)>) {
    for (entity, mut p) in &mut q {
        p.life.tick(time.delta());
        if p.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn powerup_color(kind: PowerupKind) -> Color {
    match kind {
        PowerupKind::Shield => Color::srgb(0.3, 0.6, 1.0),
        PowerupKind::RapidFire => Color::srgb(1.0, 0.8, 0.2),
        PowerupKind::Spread => Color::srgb(0.6, 1.0, 0.4),
        PowerupKind::ExtraLife => Color::srgb(1.0, 0.4, 0.6),
        PowerupKind::SpecialWeapon(_) => Color::srgb(1.0, 0.3, 1.0),
    }
}

fn draw_powerups(mut gizmos: Gizmos, q: Query<(&Transform, &Powerup)>) {
    for (transform, p) in &q {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            POWERUP_RADIUS,
            powerup_color(p.kind),
        );
    }
}

/// 우주선이 파워업에 닿으면 효과를 적용한다. (실드/연사/확산탄/특수무기는 후속 태스크에서 각자 컴포넌트를 부여)
fn collect_powerup(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut sfx: MessageWriter<SfxEvent>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    powerups: Query<(Entity, &Transform, &Collider, &Powerup)>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (powerup_entity, powerup_tf, powerup_col, powerup) in &powerups {
        if crate::core::logic::circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            powerup_tf.translation.truncate(),
            powerup_col.radius,
        ) {
            match powerup.kind {
                PowerupKind::ExtraLife => lives.0 += 1,
                PowerupKind::Shield => {
                    commands
                        .entity(player_entity)
                        .insert(Shield(Timer::from_seconds(SHIELD_SECS, TimerMode::Once)));
                }
                PowerupKind::RapidFire => {
                    commands.entity(player_entity).insert(RapidFire(
                        Timer::from_seconds(RAPID_FIRE_SECS, TimerMode::Once),
                    ));
                }
                PowerupKind::Spread => {
                    commands
                        .entity(player_entity)
                        .insert(Spread(Timer::from_seconds(SPREAD_SECS, TimerMode::Once)));
                }
                // 특수무기는 Task 9에서 이 match에 팔을 추가해 컴포넌트 부여
                _ => {}
            }
            commands.entity(powerup_entity).despawn();
            sfx.write(SfxEvent(Sfx::Pickup));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn pick_kind_covers_all_buckets() {
        // 경계값이 유효한 종류를 반환하는지(패닉/누락 없음)
        for i in 0..=10 {
            let _ = pick_powerup_kind(i as f32 / 10.0);
        }
        assert!(matches!(pick_powerup_kind(0.0), PowerupKind::Shield));
        // 마지막 버킷은 특수무기
        assert!(matches!(pick_powerup_kind(0.99), PowerupKind::SpecialWeapon(_)));
    }

    #[test]
    fn spawn_powerup_creates_entity() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut c: Commands| {
                spawn_powerup(&mut c, PowerupKind::ExtraLife, Vec2::ZERO);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Powerup>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }
}
