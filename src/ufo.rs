use bevy::prelude::*;
use rand::RngExt;

use crate::bullet::spawn_enemy_bullet;
use crate::components::{Collider, Velocity};
use crate::config::{
    HALF_HEIGHT, HALF_WIDTH, UFO_BULLET_SPEED, UFO_FIRE_INTERVAL_SECS, UFO_LARGE_RADIUS,
    UFO_SMALL_RADIUS, UFO_SPEED,
};
use crate::logic::aim_direction;
use crate::player::Player;
use crate::state::{GameState, GameplayEntity};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UfoSize {
    Large,
    Small,
}

impl UfoSize {
    pub fn radius(self) -> f32 {
        match self {
            UfoSize::Large => UFO_LARGE_RADIUS,
            UfoSize::Small => UFO_SMALL_RADIUS,
        }
    }
    pub fn score(self) -> u32 {
        match self {
            UfoSize::Large => 200,
            UfoSize::Small => 1000,
        }
    }
}

#[derive(Component)]
pub struct Ufo {
    pub size: UfoSize,
    pub fire_timer: Timer,
}

#[derive(Resource)]
pub struct UfoSpawnTimer(pub Timer);

pub struct UfoPlugin;

impl Plugin for UfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UfoSpawnTimer(Timer::from_seconds(
            crate::config::UFO_SPAWN_INTERVAL_BASE,
            TimerMode::Once,
        )))
        .add_systems(
            Update,
            (ufo_spawn_system, ufo_wobble, ufo_fire, despawn_offscreen_ufo, draw_ufos)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn ufo_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    wave: Res<crate::state::Wave>,
    mut timer: ResMut<UfoSpawnTimer>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    let mut rng = rand::rng();
    let size = if rng.random_range(0.0..1.0) < crate::logic::small_ufo_probability_for_wave(wave.0)
    {
        UfoSize::Small
    } else {
        UfoSize::Large
    };
    let from_left = rng.random_range(0.0..1.0) < 0.5;
    spawn_ufo(&mut commands, size, from_left);
    // 다음 간격을 웨이브 기반으로 재설정
    let interval = crate::logic::ufo_interval_for_wave(wave.0);
    timer.0 = Timer::from_seconds(interval, TimerMode::Once);
}

/// 화면 좌/우 가장자리에서 등장해 반대편으로 수평 이동하는 UFO 1기를 스폰.
pub fn spawn_ufo(commands: &mut Commands, size: UfoSize, from_left: bool) {
    let mut rng = rand::rng();
    let dir = if from_left { 1.0 } else { -1.0 };
    let x = -dir * (HALF_WIDTH + size.radius());
    let y = rng.random_range(-HALF_HEIGHT * 0.6..HALF_HEIGHT * 0.6);
    commands.spawn((
        Ufo {
            size,
            fire_timer: Timer::from_seconds(UFO_FIRE_INTERVAL_SECS, TimerMode::Repeating),
        },
        Transform::from_xyz(x, y, 0.0),
        Velocity(Vec2::new(dir * UFO_SPEED, 0.0)),
        Collider { radius: size.radius() },
        GameplayEntity,
    ));
}

fn ufo_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut ufos: Query<(&Transform, &mut Ufo)>,
    players: Query<&Transform, With<Player>>,
) {
    let player_pos = players.single().ok().map(|t| t.translation.truncate());
    for (ufo_tf, mut ufo) in &mut ufos {
        ufo.fire_timer.tick(time.delta());
        if !ufo.fire_timer.is_finished() {
            continue;
        }
        let origin = ufo_tf.translation.truncate();
        let dir = match ufo.size {
            UfoSize::Small => match player_pos {
                Some(p) => aim_direction(origin, p),
                None => {
                    let mut rng = rand::rng();
                    let a = rng.random_range(0.0..std::f32::consts::TAU);
                    Vec2::new(a.cos(), a.sin())
                }
            },
            UfoSize::Large => {
                let mut rng = rand::rng();
                let a = rng.random_range(0.0..std::f32::consts::TAU);
                Vec2::new(a.cos(), a.sin())
            }
        };
        spawn_enemy_bullet(&mut commands, origin, dir * UFO_BULLET_SPEED);
    }
}

/// 수직 사인 흔들림(수평 이동은 apply_velocity가 처리).
fn ufo_wobble(time: Res<Time>, mut query: Query<&mut Transform, With<Ufo>>) {
    let dt = time.delta_secs();
    let wobble = (time.elapsed_secs() * 2.0).cos() * 40.0 * dt;
    for mut transform in &mut query {
        transform.translation.y += wobble;
    }
}

fn despawn_offscreen_ufo(mut commands: Commands, query: Query<(Entity, &Transform), With<Ufo>>) {
    for (entity, transform) in &query {
        if transform.translation.x.abs() > HALF_WIDTH + 60.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_ufos(mut gizmos: Gizmos, query: Query<(&Transform, &Ufo)>) {
    for (transform, ufo) in &query {
        let r = ufo.size.radius();
        let pos = transform.translation.truncate();
        // 몸통(원)
        gizmos.circle_2d(Isometry2d::from_translation(pos), r, Color::srgb(0.7, 1.0, 0.7));
        // 상단 돔(작은 원)
        gizmos.circle_2d(
            Isometry2d::from_translation(pos + Vec2::new(0.0, r * 0.5)),
            r * 0.5,
            Color::srgb(0.7, 1.0, 0.7),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn ufo_scores_small_higher_than_large() {
        assert_eq!(UfoSize::Large.score(), 200);
        assert_eq!(UfoSize::Small.score(), 1000);
        assert!(UfoSize::Small.score() > UfoSize::Large.score());
    }

    #[test]
    fn spawn_ufo_creates_one_moving_ufo() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_ufo(&mut commands, UfoSize::Large, true);
            })
            .unwrap();
        let mut q = app.world_mut().query::<(&Ufo, &Velocity)>();
        let items: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(items.len(), 1);
        assert!(items[0].1 .0.x > 0.0); // from_left → 오른쪽으로 이동
    }

    #[test]
    fn offscreen_ufo_is_despawned() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((
                Ufo { size: UfoSize::Large, fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
                Transform::from_xyz(HALF_WIDTH + 100.0, 0.0, 0.0),
            ))
            .id();
        app.world_mut().run_system_once(despawn_offscreen_ufo).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
