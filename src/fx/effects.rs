use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::Velocity;
use crate::core::config::{PARTICLE_LIFETIME_SECS, PARTICLE_SPEED_MAX, PARTICLE_SPEED_MIN};
use crate::core::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Particle {
    pub life: Timer,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (particle_lifetime, draw_particles).run_if(in_state(GameState::Playing)),
        );
    }
}

/// 지정 위치에서 바깥 방향으로 흩어지는 단명 파티클을 count개 스폰한다.
pub fn spawn_explosion(commands: &mut Commands, position: Vec2, count: usize) {
    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED_MIN..PARTICLE_SPEED_MAX);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        commands.spawn((
            Particle { life: Timer::from_seconds(PARTICLE_LIFETIME_SECS, TimerMode::Once) },
            Transform::from_translation(position.extend(0.0)),
            Velocity(velocity),
            GameplayEntity,
        ));
    }
}

fn particle_lifetime(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Particle)>) {
    for (entity, mut particle) in &mut query {
        particle.life.tick(time.delta());
        if particle.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_particles(mut gizmos: Gizmos, query: Query<(&Transform, &Particle)>) {
    for (transform, particle) in &query {
        // 남은 수명 비율로 밝기 감소
        let frac = particle.life.fraction_remaining();
        let color = Color::srgb(frac, frac, frac * 0.6);
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            1.5,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn spawn_explosion_creates_particles() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_explosion(&mut commands, Vec2::ZERO, 8);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Particle>();
        assert_eq!(q.iter(app.world()).count(), 8);
    }

    #[test]
    fn expired_particle_is_despawned() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(0.5, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(1.0));
        let e = app.world_mut().spawn(Particle { life: timer }).id();
        app.world_mut().run_system_once(particle_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
