use bevy::prelude::*;
use rand::RngExt;

use crate::core::config::{HALF_HEIGHT, HALF_WIDTH, STAR_COUNT, TWINKLE_SPEED};

#[derive(Component)]
pub struct Star {
    pub phase: f32,
    pub base_brightness: f32,
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_stars)
            .add_systems(Update, draw_stars);
    }
}

fn spawn_stars(mut commands: Commands) {
    let mut rng = rand::rng();
    for _ in 0..STAR_COUNT {
        let x = rng.random_range(-HALF_WIDTH..HALF_WIDTH);
        let y = rng.random_range(-HALF_HEIGHT..HALF_HEIGHT);
        commands.spawn((
            Star {
                phase: rng.random_range(0.0..std::f32::consts::TAU),
                base_brightness: rng.random_range(0.3..1.0),
            },
            Transform::from_xyz(x, y, 0.0),
        ));
    }
}

fn draw_stars(mut gizmos: Gizmos, time: Res<Time>, query: Query<(&Transform, &Star)>) {
    let t = time.elapsed_secs();
    for (transform, star) in &query {
        let b = star.base_brightness * (0.5 + 0.5 * (t * TWINKLE_SPEED + star.phase).sin());
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            1.0,
            Color::srgb(b, b, b),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn spawns_configured_star_count() {
        let mut app = App::new();
        app.world_mut().run_system_once(spawn_stars).unwrap();
        let mut q = app.world_mut().query::<&Star>();
        assert_eq!(q.iter(app.world()).count(), STAR_COUNT);
    }
}
