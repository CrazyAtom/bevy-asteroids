use bevy::prelude::*;

use crate::core::components::{Collider, Velocity, Wrapping};
use crate::core::config::{
    BULLET_COLLIDER_RADIUS, BULLET_LIFETIME_SECS, BULLET_SPEED, ENEMY_BULLET_COLLIDER_RADIUS,
    UFO_BULLET_LIFETIME_SECS,
};
use crate::entities::player::Player;
use crate::core::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Bullet {
    pub life: Timer,
}

#[derive(Component)]
pub struct EnemyBullet {
    pub life: Timer,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (fire_bullet, bullet_lifetime, draw_bullets, enemy_bullet_lifetime, draw_enemy_bullets)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn spawn_enemy_bullet(commands: &mut Commands, position: Vec2, velocity: Vec2) {
    commands.spawn((
        EnemyBullet {
            life: Timer::from_seconds(UFO_BULLET_LIFETIME_SECS, TimerMode::Once),
        },
        Transform::from_translation(position.extend(0.0)),
        Velocity(velocity),
        Collider { radius: ENEMY_BULLET_COLLIDER_RADIUS },
        GameplayEntity,
    ));
}

fn enemy_bullet_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyBullet)>,
) {
    for (entity, mut bullet) in &mut query {
        bullet.life.tick(time.delta());
        if bullet.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_enemy_bullets(mut gizmos: Gizmos, query: Query<&Transform, With<EnemyBullet>>) {
    for transform in &query {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            2.5,
            Color::srgb(1.0, 0.3, 0.3),
        );
    }
}

fn fire_bullet(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    let Ok(ship) = query.single() else {
        return;
    };
    let forward = (ship.rotation * Vec3::Y).truncate();
    let nose = ship.translation + (ship.rotation * Vec3::Y) * 18.0;
    commands.spawn((
        Bullet {
            life: Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once),
        },
        Transform::from_translation(nose),
        Velocity(forward * BULLET_SPEED),
        Collider { radius: BULLET_COLLIDER_RADIUS },
        Wrapping,
        GameplayEntity,
    ));
}

fn bullet_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in &mut query {
        bullet.life.tick(time.delta());
        if bullet.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_bullets(mut gizmos: Gizmos, query: Query<&Transform, With<Bullet>>) {
    for transform in &query {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            2.0,
            Color::WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn expired_bullet_is_despawned() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(2.0)); // 이미 만료
        let e = app.world_mut().spawn(Bullet { life: timer }).id();
        app.world_mut().run_system_once(bullet_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }

    #[test]
    fn live_bullet_survives() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let timer = Timer::from_seconds(1.0, TimerMode::Once); // 아직 살아있음
        let e = app.world_mut().spawn(Bullet { life: timer }).id();
        app.world_mut().run_system_once(bullet_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_ok());
    }

    #[test]
    fn spawn_enemy_bullet_creates_entity() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_enemy_bullet(&mut commands, Vec2::ZERO, Vec2::new(0.0, -100.0));
            })
            .unwrap();
        let mut q = app.world_mut().query::<&EnemyBullet>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn expired_enemy_bullet_is_despawned() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(2.0));
        let e = app.world_mut().spawn(EnemyBullet { life: timer }).id();
        app.world_mut().run_system_once(enemy_bullet_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
