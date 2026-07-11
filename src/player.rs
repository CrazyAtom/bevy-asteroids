use bevy::prelude::*;

use crate::components::{Collider, Velocity, Wrapping};
use crate::config::{
    SHIP_BRAKE_RATE, SHIP_COLLIDER_RADIUS, SHIP_DAMPING, SHIP_MAX_SPEED, SHIP_ROTATION_SPEED,
    SHIP_THRUST,
};
use crate::logic::apply_brake;
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
            )
            .add_systems(
                FixedUpdate,
                apply_ship_damping.run_if(in_state(GameState::Playing)),
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

        if keys.pressed(KeyCode::ArrowDown) {
            velocity.0 = apply_brake(velocity.0, SHIP_BRAKE_RATE, dt);
        }
    }
}

/// 우주선 전용 감속(마찰)과 최고 속도 제한. FixedUpdate에서 매 틱 실행돼
/// 추진을 멈추면 속도가 서서히 줄어 정지하고, 속도가 상한을 넘지 않게 한다.
/// 소행성·총알에는 적용되지 않으므로 그들은 등속을 유지한다.
fn apply_ship_damping(mut query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 *= SHIP_DAMPING;
        velocity.0 = velocity.0.clamp_length_max(SHIP_MAX_SPEED);
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

    #[test]
    fn ship_damping_reduces_speed_but_keeps_direction() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((Player, Velocity(Vec2::new(200.0, 0.0))))
            .id();
        app.world_mut().run_system_once(apply_ship_damping).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        // 감속하되(속도 감소) 방향은 유지(+x)
        assert!(v.0.x < 200.0);
        assert!(v.0.x > 0.0);
    }

    #[test]
    fn ship_speed_is_capped_at_max() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((Player, Velocity(Vec2::new(10_000.0, 0.0))))
            .id();
        app.world_mut().run_system_once(apply_ship_damping).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        assert!(v.0.length() <= SHIP_MAX_SPEED + 1e-3);
    }
}
