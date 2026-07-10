use std::collections::HashSet;

use bevy::prelude::*;

use crate::asteroid::{spawn_asteroid, Asteroid};
use crate::bullet::Bullet;
use crate::components::Collider;
use crate::logic::{circles_overlap, next_asteroid_size};
use crate::player::{spawn_player_entity, Player};
use crate::state::{GameState, Lives, Score};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (bullet_vs_asteroid, player_vs_asteroid).run_if(in_state(GameState::Playing)),
        );
    }
}

fn bullet_vs_asteroid(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    asteroids: Query<(Entity, &Transform, &Collider, &Asteroid)>,
) {
    let mut destroyed: HashSet<Entity> = HashSet::new();

    for (bullet_entity, bullet_tf, bullet_col) in &bullets {
        for (asteroid_entity, asteroid_tf, asteroid_col, asteroid) in &asteroids {
            if destroyed.contains(&asteroid_entity) {
                continue;
            }
            if circles_overlap(
                bullet_tf.translation.truncate(),
                bullet_col.radius,
                asteroid_tf.translation.truncate(),
                asteroid_col.radius,
            ) {
                destroyed.insert(asteroid_entity);
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();
                score.0 += asteroid.size.score();

                if let Some(next) = next_asteroid_size(asteroid.size) {
                    let base = asteroid_tf.translation.truncate();
                    for dir in [1.0_f32, -1.0] {
                        let velocity = Vec2::new(dir * 70.0, dir * 45.0);
                        spawn_asteroid(&mut commands, next, base, velocity);
                    }
                }
                break; // 이 총알은 소진됨
            }
        }
    }
}

fn player_vs_asteroid(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    asteroids: Query<(&Transform, &Collider), With<Asteroid>>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (asteroid_tf, asteroid_col) in &asteroids {
        if circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            asteroid_tf.translation.truncate(),
            asteroid_col.radius,
        ) {
            lives.0 = lives.0.saturating_sub(1);
            commands.entity(player_entity).despawn();
            if lives.0 == 0 {
                next_state.set(GameState::GameOver);
            } else {
                spawn_player_entity(&mut commands);
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Collider;
    use crate::logic::AsteroidSize;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn bullet_splits_large_asteroid_and_scores() {
        let mut app = App::new();
        app.insert_resource(Score(0));
        let bullet = app
            .world_mut()
            .spawn((
                Bullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Collider { radius: 2.0 },
            ))
            .id();
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(bullet_vs_asteroid).unwrap();

        // 총알 소멸
        assert!(app.world().get_entity(bullet).is_err());
        // 점수 증가
        assert_eq!(app.world().resource::<Score>().0, AsteroidSize::Large.score());
        // 큰 소행성 → 중간 2개
        let mut q = app.world_mut().query::<&Asteroid>();
        let sizes: Vec<_> = q.iter(app.world()).map(|a| a.size).collect();
        assert_eq!(sizes.len(), 2);
        assert!(sizes.iter().all(|s| *s == AsteroidSize::Medium));
    }

    #[test]
    fn small_asteroid_vanishes_without_children() {
        let mut app = App::new();
        app.insert_resource(Score(0));
        app.world_mut().spawn((
            Bullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 2.0 },
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Small },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Small.radius() },
        ));

        app.world_mut().run_system_once(bullet_vs_asteroid).unwrap();

        let mut q = app.world_mut().query::<&Asteroid>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn player_hit_loses_life_and_respawns() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(Lives(3));
        app.world_mut().spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 12.0 },
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(player_vs_asteroid).unwrap();

        assert_eq!(app.world().resource::<Lives>().0, 2);
        let mut q = app.world_mut().query::<&Player>();
        assert_eq!(q.iter(app.world()).count(), 1); // 리스폰됨
    }

    #[test]
    fn last_life_triggers_game_over() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(Lives(1));
        app.world_mut().spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 12.0 },
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(player_vs_asteroid).unwrap();

        assert_eq!(app.world().resource::<Lives>().0, 0);
        assert!(matches!(
            app.world().resource::<NextState<GameState>>(),
            NextState::Pending(GameState::GameOver)
        ));
    }
}
