use bevy::prelude::*;

use crate::core::components::{Collider, Velocity, Wrapping};
use crate::core::config::{
    BULLET_COLLIDER_RADIUS, BULLET_LIFETIME_SECS, BULLET_SPEED, ENEMY_BULLET_COLLIDER_RADIUS,
    FIRE_INTERVAL, RAPID_FIRE_INTERVAL, SPREAD_ANGLE, UFO_BULLET_LIFETIME_SECS,
};
use crate::entities::player::{FireCooldown, Player, RapidFire, Spread};
use crate::core::state::{GameState, GameplayEntity};
use crate::fx::audio::{Sfx, SfxEvent};

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

#[allow(clippy::type_complexity)]
fn fire_bullet(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut query: Query<(&Transform, &mut FireCooldown, Option<&RapidFire>, Option<&Spread>), With<Player>>,
) {
    let Ok((ship, mut cooldown, rapid, spread)) = query.single_mut() else {
        return;
    };
    cooldown.0.tick(time.delta());
    if !keys.pressed(KeyCode::Space) || !cooldown.0.is_finished() {
        return;
    }
    let interval = if rapid.is_some() { RAPID_FIRE_INTERVAL } else { FIRE_INTERVAL };
    cooldown.0 = Timer::from_seconds(interval, TimerMode::Once);
    let base = ship.rotation * Vec3::Y;
    let nose = ship.translation + base * 18.0;
    let angles: &[f32] = if spread.is_some() { &[-SPREAD_ANGLE, 0.0, SPREAD_ANGLE] } else { &[0.0] };
    for &a in angles {
        let dir = (Quat::from_rotation_z(a) * base).truncate();
        commands.spawn((
            Bullet { life: Timer::from_seconds(BULLET_LIFETIME_SECS, TimerMode::Once) },
            Transform::from_translation(nose),
            Velocity(dir * BULLET_SPEED),
            Collider { radius: BULLET_COLLIDER_RADIUS },
            Wrapping,
            GameplayEntity,
        ));
    }
    sfx.write(SfxEvent(Sfx::Fire));
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

    #[test]
    fn fires_only_when_cooldown_ready() {
        use crate::entities::player::{FireCooldown, Player};
        let mut app = App::new();
        app.add_message::<SfxEvent>();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        app.insert_resource(keys);
        // 준비완료 쿨다운
        let mut cd = Timer::from_seconds(0.25, TimerMode::Once);
        cd.tick(std::time::Duration::from_secs_f32(1.0));
        app.world_mut().spawn((Player, Transform::default(), FireCooldown(cd)));
        app.world_mut().run_system_once(fire_bullet).unwrap();
        let mut q = app.world_mut().query::<&Bullet>();
        assert_eq!(q.iter(app.world()).count(), 1); // 한 발
        // 발사 후 쿨다운이 재무장(미완료)됐는지 확인
        let mut cq = app.world_mut().query::<&FireCooldown>();
        let cd = cq.iter(app.world()).next().unwrap();
        assert!(!cd.0.is_finished());
    }

    #[test]
    fn spread_fires_three_bullets() {
        use crate::entities::player::{FireCooldown, Player, Spread};
        let mut app = App::new();
        app.add_message::<SfxEvent>();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        app.insert_resource(keys);
        let mut cd = Timer::from_seconds(0.25, TimerMode::Once);
        cd.tick(std::time::Duration::from_secs_f32(1.0));
        app.world_mut().spawn((
            Player,
            Transform::default(),
            FireCooldown(cd),
            Spread(Timer::from_seconds(6.0, TimerMode::Once)),
        ));
        app.world_mut().run_system_once(fire_bullet).unwrap();
        let mut q = app.world_mut().query::<&Bullet>();
        assert_eq!(q.iter(app.world()).count(), 3);
    }
}
