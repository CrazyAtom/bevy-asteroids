use bevy::prelude::*;
use rand::RngExt;

use crate::core::config::MAX_SHAKE_OFFSET;
use crate::core::logic::decay_trauma;
use crate::core::state::RunPhase;

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

#[derive(Message)]
pub struct ShakeEvent(pub f32);

pub struct ShakePlugin;

impl Plugin for ShakePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .add_message::<ShakeEvent>()
            .add_systems(Update, apply_screen_shake.run_if(not(in_state(RunPhase::Paused))));
    }
}

fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut events: MessageReader<ShakeEvent>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    for ShakeEvent(amount) in events.read() {
        shake.trauma = (shake.trauma + amount).clamp(0.0, 1.0);
    }
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let intensity = shake.trauma * shake.trauma;
    if intensity > 0.0 {
        let mut rng = rand::rng();
        transform.translation.x = rng.random_range(-1.0..1.0) * MAX_SHAKE_OFFSET * intensity;
        transform.translation.y = rng.random_range(-1.0..1.0) * MAX_SHAKE_OFFSET * intensity;
    } else {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
    shake.trauma = decay_trauma(shake.trauma, time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::GameState;
    use std::time::Duration;

    /// 일시정지 중에는 화면 흔들림(trauma 감쇠)이 멈춰야 한다(스펙 §4).
    #[test]
    fn shake_freezes_when_paused() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<RunPhase>();
        app.insert_resource(ScreenShake { trauma: 1.0 });
        app.add_message::<ShakeEvent>();
        app.add_systems(Update, apply_screen_shake.run_if(not(in_state(RunPhase::Paused))));
        app.world_mut().spawn((Camera2d, Transform::default()));

        // Playing/Running 진입 → 시간 진행 → trauma가 감쇠해야 한다
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        app.update();
        let after_running = app.world().resource::<ScreenShake>().trauma;
        assert!(
            after_running < 1.0,
            "Running 중에는 trauma가 감쇠해야 한다 (got {after_running})"
        );

        // 일시정지 → 시간이 흘러도 trauma가 멈춰야 한다
        app.world_mut().resource_mut::<NextState<RunPhase>>().set(RunPhase::Paused);
        app.update();
        let frozen = app.world().resource::<ScreenShake>().trauma;
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        app.update();
        assert_eq!(
            app.world().resource::<ScreenShake>().trauma,
            frozen,
            "일시정지 중에는 화면 흔들림이 멈춰야 한다"
        );
    }
}
