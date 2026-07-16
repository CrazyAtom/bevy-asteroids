use bevy::prelude::*;

use crate::core::components::{AngularVelocity, EdgeReflect, Velocity, Wrapping};
use crate::core::config::{HALF_HEIGHT, HALF_WIDTH, SHIP_DAMPING};
use crate::core::logic::{reflect_edge, wrap_position};
use crate::core::state::RunPhase;

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
            ((apply_velocity, wrap_around, reflect_or_wrap).chain(), apply_spin)
                .run_if(in_state(RunPhase::Running)),
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
    use crate::core::state::GameState;
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

    /// 일시정지 중에는 실제 물리 적분(apply_velocity)이 멈춰야 한다(스펙 §4).
    /// FixedUpdate 스케줄 타이밍에 흔들리지 않도록 동일한 게이팅을 Update에 등록해 검증한다.
    #[test]
    fn real_movement_freezes_when_paused() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<RunPhase>();
        // MovementPlugin과 동일한 게이팅으로 실제 apply_velocity를 등록(FixedUpdate 타이밍 배제 위해 Update 사용)
        app.add_systems(Update, apply_velocity.run_if(in_state(RunPhase::Running)));

        let e = app
            .world_mut()
            .spawn((Transform::default(), Velocity(Vec2::new(100.0, 0.0))))
            .id();

        // Playing/Running 진입 → 시간 진행 → 엔티티가 이동해야 한다
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        app.update();
        let x_after_running = app.world().entity(e).get::<Transform>().unwrap().translation.x;
        assert!(
            x_after_running > 0.0,
            "Running 중에는 실제 물리 적분이 동작해 엔티티가 이동해야 한다 (got {x_after_running})"
        );

        // 일시정지 → 시간이 흘러도 위치가 멈춰야 한다
        app.world_mut().resource_mut::<NextState<RunPhase>>().set(RunPhase::Paused);
        app.update();
        let frozen = app.world().entity(e).get::<Transform>().unwrap().translation.x;
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        app.update();
        let x_after_paused = app.world().entity(e).get::<Transform>().unwrap().translation.x;
        assert_eq!(
            x_after_paused, frozen,
            "일시정지 중에는 실제 물리 적분이 멈춰 엔티티 위치가 변하지 않아야 한다"
        );
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
