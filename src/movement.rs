// src/movement.rs
use bevy::prelude::*;

use crate::components::{Velocity, Wrapping};
use crate::config::{HALF_HEIGHT, HALF_WIDTH};
use crate::logic::wrap_position;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (apply_velocity, wrap_around).chain());
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

fn wrap_around(mut query: Query<&mut Transform, With<Wrapping>>) {
    let half = Vec2::new(HALF_WIDTH, HALF_HEIGHT);
    for mut transform in &mut query {
        let wrapped = wrap_position(transform.translation.truncate(), half);
        transform.translation.x = wrapped.x;
        transform.translation.y = wrapped.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn velocity_moves_entity() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);
        let e = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), Velocity(Vec2::new(10.0, -20.0))))
            .id();
        app.world_mut().run_system_once(apply_velocity).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        assert!((t.translation.x - 5.0).abs() < 1e-3);
        assert!((t.translation.y + 10.0).abs() < 1e-3);
    }

    #[test]
    fn wrapping_teleports_out_of_bounds_entity() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((Transform::from_xyz(HALF_WIDTH + 50.0, 0.0, 0.0), Wrapping))
            .id();
        app.world_mut().run_system_once(wrap_around).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        assert!(t.translation.x < 0.0);
    }
}
