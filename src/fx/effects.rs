use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::Velocity;
use crate::core::config::{PARTICLE_LIFETIME_SECS, PARTICLE_SPEED_MAX, PARTICLE_SPEED_MIN, Z_PARTICLE};
use crate::core::state::{GameplayEntity, RunPhase};
use crate::fx::sprites::SpriteAssets;

#[derive(Component)]
pub struct Particle {
    pub life: Timer,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (particle_lifetime, fade_particles).run_if(in_state(RunPhase::Running)),
        );
    }
}

/// 지정 위치에서 바깥 방향으로 흩어지는 단명 파티클을 count개 스폰한다.
pub fn spawn_explosion(commands: &mut Commands, assets: &SpriteAssets, position: Vec2, count: usize) {
    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED_MIN..PARTICLE_SPEED_MAX);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        commands.spawn((
            Particle { life: Timer::from_seconds(PARTICLE_LIFETIME_SECS, TimerMode::Once) },
            Sprite {
                image: assets.spark.clone(),
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_translation(position.extend(Z_PARTICLE)),
            Velocity(velocity),
            GameplayEntity,
        ));
    }
    // 폭발 지점에 프레임 애니메이션 폭발도 함께 재생.
    crate::fx::animation::spawn_explosion_anim(commands, assets, position);
}

fn particle_lifetime(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Particle)>) {
    for (entity, mut particle) in &mut query {
        particle.life.tick(time.delta());
        if particle.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// 남은 수명 비율(0~1)을 스프라이트 알파로 매핑.
pub fn particle_alpha(fraction_remaining: f32) -> f32 {
    fraction_remaining.clamp(0.0, 1.0)
}

/// 파티클의 남은 수명에 맞춰 스파크 스프라이트를 서서히 투명하게 만든다.
fn fade_particles(mut q: Query<(&Particle, &mut Sprite)>) {
    for (particle, mut sprite) in &mut q {
        sprite.color.set_alpha(particle_alpha(particle.life.fraction_remaining()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn particle_alpha_follows_remaining_life() {
        assert!((particle_alpha(1.0) - 1.0).abs() < 1e-6);
        assert!(particle_alpha(0.0).abs() < 1e-6);
        assert!(particle_alpha(0.5) > 0.0 && particle_alpha(0.5) < 1.0);
    }

    #[test]
    fn spawn_explosion_creates_particles() {
        let mut app = App::new();
        let assets = crate::fx::sprites::dummy_sprite_assets();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_explosion(&mut commands, &assets, Vec2::ZERO, 8);
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
