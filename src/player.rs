use bevy::prelude::*;

use crate::components::{Collider, Velocity, Wrapping};
use crate::config::{SHIP_COLLIDER_RADIUS, SHIP_ROTATION_SPEED, SHIP_THRUST};
use crate::state::{GameState, GameplayEntity};

#[derive(Component)]
pub struct Player;

/// 우주선 로컬 좌표(정면 = +Y). 마지막 점은 첫 점과 같아 닫힌 외곽선을 만든다.
const SHIP_POINTS: [Vec2; 5] = [
    Vec2::new(0.0, 16.0),
    Vec2::new(-11.0, -12.0),
    Vec2::new(0.0, -6.0),
    Vec2::new(11.0, -12.0),
    Vec2::new(0.0, 16.0),
];

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (player_input, draw_player).run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn spawn_player_entity(commands: &mut Commands) {
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity(Vec2::ZERO),
        Collider { radius: SHIP_COLLIDER_RADIUS },
        Wrapping,
        GameplayEntity,
    ));
}

fn spawn_player(mut commands: Commands) {
    spawn_player_entity(&mut commands);
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut query {
        let mut turn = 0.0;
        if keys.pressed(KeyCode::ArrowLeft) {
            turn += 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            turn -= 1.0;
        }
        transform.rotate_z(turn * SHIP_ROTATION_SPEED * dt);

        if keys.pressed(KeyCode::ArrowUp) {
            let forward = (transform.rotation * Vec3::Y).truncate();
            velocity.0 += forward * SHIP_THRUST * dt;
        }
    }
}

fn draw_player(mut gizmos: Gizmos, query: Query<&Transform, With<Player>>) {
    for transform in &query {
        let points = SHIP_POINTS.map(|p| transform.transform_point(p.extend(0.0)).truncate());
        gizmos.linestrip_2d(points, Color::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Velocity;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn thrust_accelerates_forward() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowUp);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0), Velocity(Vec2::ZERO)))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        // 회전 없음 → 정면은 +Y. 위로 가속.
        assert!(v.0.y > 0.0);
        assert!(v.0.x.abs() < 1e-3);
    }

    #[test]
    fn left_key_rotates_ship() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowLeft);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0), Velocity(Vec2::ZERO)))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        // 좌회전 → z축 회전각 > 0
        let (_, angle) = t.rotation.to_axis_angle();
        assert!(angle > 0.0);
    }
}
