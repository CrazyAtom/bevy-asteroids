use bevy::prelude::*;

use crate::core::components::{AngularVelocity, EdgeReflect, Velocity, Wrapping};
use crate::core::config::{HALF_HEIGHT, HALF_WIDTH, SHIP_DAMPING};
use crate::core::logic::{reflect_edge, wrap_position};

/// 현재 스테이지의 물리 트위스트. 배경처럼 매 프레임 현재 테마로 동기화된다.
#[derive(Resource)]
pub struct StageModifiers {
    pub wall_bounce: bool,  // 위험요소가 경계에서 반사되는가(얼음 = true)
    pub ship_damping: f32,  // 틱당 속도에 곱하는 계수(1에 가까울수록 마찰↓ → 더 미끄러움)
}

impl Default for StageModifiers {
    fn default() -> Self {
        Self { wall_bounce: false, ship_damping: SHIP_DAMPING }
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StageModifiers>().add_systems(
            FixedUpdate,
            ((apply_velocity, wrap_around, reflect_or_wrap).chain(), apply_spin),
        );
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

fn apply_spin(time: Res<Time>, mut query: Query<(&mut Transform, &AngularVelocity)>) {
    let dt = time.delta_secs();
    for (mut transform, angular) in &mut query {
        transform.rotate_z(angular.0 * dt);
    }
}

fn wrap_around(mut query: Query<&mut Transform, (With<Wrapping>, Without<EdgeReflect>)>) {
    let half = Vec2::new(HALF_WIDTH, HALF_HEIGHT);
    for mut transform in &mut query {
        let wrapped = wrap_position(transform.translation.truncate(), half);
        transform.translation.x = wrapped.x;
        transform.translation.y = wrapped.y;
    }
}

/// EdgeReflect 대상의 경계 처리. wall_bounce가 켜지면 반사, 아니면 순환(기존과 동일).
fn reflect_or_wrap(
    mods: Res<StageModifiers>,
    mut query: Query<(&mut Transform, &mut Velocity), With<EdgeReflect>>,
) {
    let half = Vec2::new(HALF_WIDTH, HALF_HEIGHT);
    for (mut transform, mut velocity) in &mut query {
        let pos = transform.translation.truncate();
        if mods.wall_bounce {
            let (p, v) = reflect_edge(pos, velocity.0, half);
            transform.translation.x = p.x;
            transform.translation.y = p.y;
            velocity.0 = v;
        } else {
            let w = wrap_position(pos, half);
            transform.translation.x = w.x;
            transform.translation.y = w.y;
        }
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

    #[test]
    fn spin_rotates_entity() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.5));
        app.insert_resource(time);
        let e = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), AngularVelocity(2.0)))
            .id();
        app.world_mut().run_system_once(apply_spin).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        let (_, angle) = t.rotation.to_axis_angle();
        assert!(angle > 0.0); // 회전 발생
    }

    #[test]
    fn edge_reflect_entity_wraps_when_bounce_off() {
        let mut app = App::new();
        app.insert_resource(StageModifiers { wall_bounce: false, ship_damping: 0.985 });
        let e = app
            .world_mut()
            .spawn((
                Transform::from_xyz(HALF_WIDTH + 50.0, 0.0, 0.0),
                Velocity(Vec2::new(100.0, 0.0)),
                crate::core::components::EdgeReflect,
            ))
            .id();
        app.world_mut().run_system_once(reflect_or_wrap).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        assert!(t.translation.x < 0.0); // 순환(반대편)
    }

    #[test]
    fn edge_reflect_entity_bounces_when_on() {
        let mut app = App::new();
        app.insert_resource(StageModifiers { wall_bounce: true, ship_damping: 0.985 });
        let e = app
            .world_mut()
            .spawn((
                Transform::from_xyz(HALF_WIDTH + 50.0, 0.0, 0.0),
                Velocity(Vec2::new(100.0, 0.0)),
                crate::core::components::EdgeReflect,
            ))
            .id();
        app.world_mut().run_system_once(reflect_or_wrap).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        assert!((t.translation.x - HALF_WIDTH).abs() < 1e-3); // 경계로 클램프
        assert!(v.0.x < 0.0); // 반사
    }
}
