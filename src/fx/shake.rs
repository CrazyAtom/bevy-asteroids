use bevy::prelude::*;
use rand::RngExt;

use crate::core::config::MAX_SHAKE_OFFSET;
use crate::core::logic::decay_trauma;

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
            .add_systems(Update, apply_screen_shake);
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
