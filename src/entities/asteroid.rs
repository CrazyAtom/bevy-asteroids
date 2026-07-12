use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{AngularVelocity, Collider, Velocity, Wrapping};
use crate::core::config::{ASTEROID_MAX_SPEED, ASTEROID_MIN_SPEED, ASTEROID_SPIN_MAX, HALF_HEIGHT, HALF_WIDTH};
use crate::core::logic::{asteroid_count_for_wave, asteroid_speed_scale_for_wave, AsteroidSize};
use crate::core::state::{GameState, GameplayEntity, Wave};

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

#[derive(Component)]
pub struct AsteroidShape {
    pub points: Vec<Vec2>,
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_initial_wave)
            .add_systems(
                Update,
                (wave_control, draw_asteroids).run_if(in_state(GameState::Playing)),
            );
    }
}

/// 반지름을 무작위로 흔든 닫힌 다각형 외곽선(로컬 좌표)을 만든다.
pub fn asteroid_shape(radius: f32) -> Vec<Vec2> {
    let mut rng = rand::rng();
    let sides = 10;
    let mut points: Vec<Vec2> = Vec::with_capacity(sides + 1);
    for i in 0..sides {
        let angle = i as f32 / sides as f32 * std::f32::consts::TAU;
        let r = radius * rng.random_range(0.75..1.15);
        points.push(Vec2::new(angle.cos() * r, angle.sin() * r));
    }
    let first = points[0];
    points.push(first); // 닫기
    points
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
) {
    let mut rng = rand::rng();
    let spin = rng.random_range(-ASTEROID_SPIN_MAX..ASTEROID_SPIN_MAX);
    commands.spawn((
        Asteroid { size },
        AsteroidShape { points: asteroid_shape(size.radius()) },
        Transform::from_translation(position.extend(0.0)),
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

fn spawn_wave(commands: &mut Commands, wave: u32) {
    let count = asteroid_count_for_wave(wave);
    let scale = asteroid_speed_scale_for_wave(wave);
    for _ in 0..count {
        let base = random_velocity(AsteroidSize::Large);
        spawn_asteroid(commands, AsteroidSize::Large, random_spawn_position(), base * scale);
    }
}

fn spawn_initial_wave(mut commands: Commands, wave: Res<Wave>) {
    spawn_wave(&mut commands, wave.0);
}

fn wave_control(mut commands: Commands, mut wave: ResMut<Wave>, asteroids: Query<(), With<Asteroid>>) {
    if asteroids.iter().count() == 0 {
        wave.0 += 1;
        spawn_wave(&mut commands, wave.0);
    }
}

fn draw_asteroids(mut gizmos: Gizmos, query: Query<(&Transform, &AsteroidShape)>) {
    for (transform, shape) in &query {
        let points = shape
            .points
            .iter()
            .map(|p| transform.transform_point(p.extend(0.0)).truncate());
        gizmos.linestrip_2d(points, Color::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn shape_is_closed_loop() {
        let pts = asteroid_shape(40.0);
        assert!(pts.len() >= 4);
        // 닫힌 외곽선: 첫 점 == 마지막 점
        assert_eq!(pts.first().unwrap(), pts.last().unwrap());
    }

    #[test]
    fn shape_points_near_radius() {
        let radius = 40.0;
        let pts = asteroid_shape(radius);
        for p in &pts {
            let len = p.length();
            // 반지름의 0.75~1.15배 범위 안에 있어야 함
            assert!(len >= radius * 0.7 && len <= radius * 1.2, "len={len}");
        }
    }

    #[test]
    fn spawn_asteroid_creates_entity_with_size() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_asteroid(&mut commands, AsteroidSize::Medium, Vec2::ZERO, Vec2::X);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Asteroid>();
        let sizes: Vec<_> = q.iter(app.world()).map(|a| a.size).collect();
        assert_eq!(sizes, vec![AsteroidSize::Medium]);
    }
}
