use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{AngularVelocity, Collider, Velocity, Wrapping};
use crate::core::config::{
    ASTEROID_MAX_SPEED, ASTEROID_MIN_SPEED, ASTEROID_SPIN_MAX, HALF_HEIGHT, HALF_WIDTH, Z_ENTITY,
};
use crate::core::logic::{asteroid_count_for_wave, asteroid_speed_scale_for_wave, AsteroidSize};
use crate::core::state::{GameState, GameplayEntity, Wave};
use crate::fx::sprites::{sprite_size_for, SpriteAssets};

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_initial_wave)
            .add_systems(Update, wave_control.run_if(in_state(GameState::Playing)));
    }
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

fn random_spawn_position() -> Vec2 {
    let mut rng = rand::rng();
    let pos = Vec2::new(
        rng.random_range(-HALF_WIDTH..HALF_WIDTH),
        rng.random_range(-HALF_HEIGHT..HALF_HEIGHT),
    );
    // 중심(우주선 스폰 위치) 근처면 바깥으로 밀어 즉사 방지
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

fn spawn_wave(commands: &mut Commands, assets: &SpriteAssets, wave: u32) {
    let count = asteroid_count_for_wave(wave);
    let scale = asteroid_speed_scale_for_wave(wave);
    for _ in 0..count {
        let base = random_velocity(AsteroidSize::Large);
        spawn_asteroid(commands, assets, AsteroidSize::Large, random_spawn_position(), base * scale);
    }
}

fn spawn_initial_wave(mut commands: Commands, assets: Res<SpriteAssets>, wave: Res<Wave>) {
    spawn_wave(&mut commands, &assets, wave.0);
}

fn wave_control(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut wave: ResMut<Wave>,
    asteroids: Query<(), With<Asteroid>>,
) {
    if asteroids.iter().count() == 0 {
        wave.0 += 1;
        spawn_wave(&mut commands, &assets, wave.0);
    }
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
