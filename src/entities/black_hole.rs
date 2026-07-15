//! 블랙홀 중력장(블랙홀 테마). 화면 중앙에 고정 블랙홀이 거리² 반비례로 우주선·소행성을
//! 끌어당기고, 사건의 지평선(중심)은 접촉 위험이다. on/off·중력세기는 `BlackHoleActive`가
//! 제어하고(테마 무관), `systems::stage::sync_black_hole`이 블랙홀 스테이지에서 켠다.

use bevy::prelude::*;

use crate::core::components::{AngularVelocity, Collider, GravityBody, Velocity};
use crate::core::config::{
    BLACK_HOLE_POS_Y, BLACK_HOLE_SPIN, BLACK_HOLE_VISUAL, EVENT_HORIZON, GRAVITY_MIN_DIST,
    GRAVITY_STRENGTH, Z_ENTITY,
};
use crate::core::logic::gravity_accel;
use crate::core::state::RunPhase;
use crate::entities::asteroid::Asteroid;
use crate::fx::sprites::SpriteAssets;

#[derive(Component)]
pub struct BlackHole;

/// 블랙홀 활성/중력세기. 기본 off. 블랙홀 테마에서만 켜진다.
#[derive(Resource)]
pub struct BlackHoleActive {
    pub active: bool,
    pub strength: f32,
}

impl Default for BlackHoleActive {
    fn default() -> Self {
        Self { active: false, strength: GRAVITY_STRENGTH }
    }
}

pub struct BlackHolePlugin;

impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlackHoleActive>()
            .add_systems(Startup, spawn_black_hole)
            // 가시성은 상태 무관 상시 실행(update_fog와 동형): GameOver에서 active=false가 되면
            // 즉시 Hidden 처리해 블랙홀 스프라이트 잔상을 막는다.
            .add_systems(Update, update_black_hole_visibility)
            .add_systems(Update, consume_asteroids.run_if(in_state(RunPhase::Running)))
            .add_systems(FixedUpdate, gravity_pull.run_if(in_state(RunPhase::Running)));
    }
}

fn spawn_black_hole(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        BlackHole,
        Sprite {
            image: assets.black_hole.clone(),
            custom_size: Some(Vec2::splat(BLACK_HOLE_VISUAL)),
            ..default()
        },
        Transform::from_xyz(0.0, BLACK_HOLE_POS_Y, Z_ENTITY),
        Collider { radius: EVENT_HORIZON },
        AngularVelocity(BLACK_HOLE_SPIN), // 강착원반이 천천히 회전(화려함)
        Visibility::Hidden,
    ));
}

fn update_black_hole_visibility(active: Res<BlackHoleActive>, mut q: Query<&mut Visibility, With<BlackHole>>) {
    let vis = if active.active { Visibility::Visible } else { Visibility::Hidden };
    for mut v in &mut q {
        *v = vis;
    }
}

/// 활성 시 GravityBody(우주선·소행성)를 블랙홀로 끌어당긴다.
fn gravity_pull(
    time: Res<Time>,
    active: Res<BlackHoleActive>,
    holes: Query<&Transform, (With<BlackHole>, Without<GravityBody>)>,
    mut bodies: Query<(&Transform, &mut Velocity), With<GravityBody>>,
) {
    if !active.active {
        return;
    }
    let Ok(hole_tf) = holes.single() else { return };
    let hole = hole_tf.translation.truncate();
    let dt = time.delta_secs();
    for (tf, mut vel) in &mut bodies {
        let accel = gravity_accel(tf.translation.truncate(), hole, active.strength, GRAVITY_MIN_DIST);
        vel.0 += accel * dt;
    }
}

/// 활성 시 사건의 지평선에 들어온 소행성을 소멸(빨려듦)시킨다.
fn consume_asteroids(
    mut commands: Commands,
    active: Res<BlackHoleActive>,
    holes: Query<&Transform, With<BlackHole>>,
    asteroids: Query<(Entity, &Transform), With<Asteroid>>,
) {
    if !active.active {
        return;
    }
    let Ok(hole_tf) = holes.single() else { return };
    let hole = hole_tf.translation.truncate();
    for (e, tf) in &asteroids {
        if tf.translation.truncate().distance(hole) < EVENT_HORIZON {
            // 같은 프레임에 총알/빔이 이 소행성을 이미 despawn했을 수 있으므로 try_despawn.
            commands.entity(e).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_hole_defaults_inactive() {
        let d = BlackHoleActive::default();
        assert!(!d.active);
        assert_eq!(d.strength, GRAVITY_STRENGTH);
    }
}
