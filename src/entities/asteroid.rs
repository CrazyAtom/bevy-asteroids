use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{AngularVelocity, Collider, Velocity, Wrapping};
use crate::core::config::{
    ASTEROID_MAX_SPEED, ASTEROID_MIN_SPEED, ASTEROID_SPIN_MAX, HALF_HEIGHT, HALF_WIDTH, Z_ENTITY,
};
use crate::core::logic::AsteroidSize;
use crate::core::state::GameplayEntity;
use crate::fx::sprites::{sprite_size_for, SpriteAssets};

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

fn asteroid_image(size: AsteroidSize, assets: &SpriteAssets) -> Handle<Image> {
    match size {
        AsteroidSize::Large => assets.meteor_large.clone(),
        AsteroidSize::Medium => assets.meteor_medium.clone(),
        AsteroidSize::Small => assets.meteor_small.clone(),
    }
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    assets: &SpriteAssets,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
) {
    let mut rng = rand::rng();
    let spin = rng.random_range(-ASTEROID_SPIN_MAX..ASTEROID_SPIN_MAX);
    commands.spawn((
        Asteroid { size },
        Sprite {
            image: asteroid_image(size, assets),
            custom_size: Some(sprite_size_for(size.radius())),
            ..default()
        },
        Transform::from_translation(position.extend(Z_ENTITY)),
        Velocity(velocity),
        AngularVelocity(spin),
        Collider { radius: size.radius() },
        Wrapping,
        GameplayEntity,
    ));
}

/// 화면 내 임의 스폰 좌표. 중심(우주선 스폰 위치) 근처면 바깥으로 밀어 즉사 방지.
pub fn random_spawn_position() -> Vec2 {
    let mut rng = rand::rng();
    let pos = Vec2::new(
        rng.random_range(-HALF_WIDTH..HALF_WIDTH),
        rng.random_range(-HALF_HEIGHT..HALF_HEIGHT),
    );
    if pos.length() < 150.0 {
        pos.normalize_or_zero() * 200.0
    } else {
        pos
    }
}

pub fn random_velocity(size: AsteroidSize) -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(ASTEROID_MIN_SPEED..ASTEROID_MAX_SPEED) * size.speed_scale();
    Vec2::new(angle.cos(), angle.sin()) * speed
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn spawn_asteroid_creates_entity_with_size() {
        let mut app = App::new();
        let assets = crate::fx::sprites::dummy_sprite_assets();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_asteroid(&mut commands, &assets, AsteroidSize::Medium, Vec2::ZERO, Vec2::X);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Asteroid>();
        let sizes: Vec<_> = q.iter(app.world()).map(|a| a.size).collect();
        assert_eq!(sizes, vec![AsteroidSize::Medium]);
    }
}
