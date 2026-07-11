use std::collections::HashSet;

use bevy::prelude::*;

use crate::asteroid::{random_velocity, spawn_asteroid, Asteroid};
use crate::bullet::{Bullet, EnemyBullet};
use crate::components::Collider;
use crate::config::EXPLOSION_PARTICLES;
use crate::effects::spawn_explosion;
use crate::logic::{circles_overlap, next_asteroid_size};
use crate::player::{spawn_player_entity, Player};
use crate::state::{GameState, Lives, Score};
use crate::ufo::Ufo;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (bullet_vs_asteroid, bullet_vs_ufo, player_damage).run_if(in_state(GameState::Playing)),
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
                spawn_explosion(&mut commands, asteroid_tf.translation.truncate(), EXPLOSION_PARTICLES);

                if let Some(next) = next_asteroid_size(asteroid.size) {
                    let base = asteroid_tf.translation.truncate();
                    for _ in 0..2 {
                        spawn_asteroid(&mut commands, next, base, random_velocity(next));
                    }
                }
                break; // 이 총알은 소진됨
            }
        }
    }
}

fn bullet_vs_ufo(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo)>,
) {
    let mut destroyed: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for (bullet_entity, bullet_tf, bullet_col) in &bullets {
        for (ufo_entity, ufo_tf, ufo_col, ufo) in &ufos {
            if destroyed.contains(&ufo_entity) {
                continue;
            }
            if circles_overlap(
                bullet_tf.translation.truncate(),
                bullet_col.radius,
                ufo_tf.translation.truncate(),
                ufo_col.radius,
            ) {
                destroyed.insert(ufo_entity);
                commands.entity(bullet_entity).despawn();
                commands.entity(ufo_entity).despawn();
                score.0 += ufo.size.score();
                spawn_explosion(&mut commands, ufo_tf.translation.truncate(), EXPLOSION_PARTICLES);
                break;
            }
        }
    }
}

fn player_damage(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    asteroids: Query<(&Transform, &Collider), With<Asteroid>>,
    ufos: Query<(&Transform, &Collider), With<Ufo>>,
    enemy_bullets: Query<(Entity, &Transform, &Collider), With<EnemyBullet>>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    let ppos = player_tf.translation.truncate();
    let pr = player_col.radius;

    let mut hit = false;
    for (a_tf, a_col) in &asteroids {
        if circles_overlap(ppos, pr, a_tf.translation.truncate(), a_col.radius) {
            hit = true;
            break;
        }
    }
    if !hit {
        for (u_tf, u_col) in &ufos {
            if circles_overlap(ppos, pr, u_tf.translation.truncate(), u_col.radius) {
                hit = true;
                break;
            }
        }
    }
    if !hit {
        for (b_entity, b_tf, b_col) in &enemy_bullets {
            if circles_overlap(ppos, pr, b_tf.translation.truncate(), b_col.radius) {
                commands.entity(b_entity).despawn();
                hit = true;
                break;
            }
        }
    }
    if !hit {
        return;
    }

    spawn_explosion(&mut commands, ppos, EXPLOSION_PARTICLES);
    lives.0 = lives.0.saturating_sub(1);
    commands.entity(player_entity).despawn();
    if lives.0 == 0 {
        next_state.set(GameState::GameOver);
    } else {
        spawn_player_entity(&mut commands);
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
    fn destroying_asteroid_spawns_particles() {
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
        let mut q = app.world_mut().query::<&crate::effects::Particle>();
        assert!(q.iter(app.world()).count() > 0);
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

        app.world_mut().run_system_once(player_damage).unwrap();

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

        app.world_mut().run_system_once(player_damage).unwrap();

        assert_eq!(app.world().resource::<Lives>().0, 0);
        assert!(matches!(
            app.world().resource::<NextState<GameState>>(),
            NextState::Pending(GameState::GameOver)
        ));
    }

    #[test]
    fn bullet_destroys_ufo_and_scores() {
        use crate::ufo::{Ufo, UfoSize};
        let mut app = App::new();
        app.insert_resource(Score(0));
        let bullet = app.world_mut().spawn((
            Bullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 2.0 },
        )).id();
        app.world_mut().spawn((
            Ufo { size: UfoSize::Small, fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: UfoSize::Small.radius() },
        ));
        app.world_mut().run_system_once(bullet_vs_ufo).unwrap();
        assert!(app.world().get_entity(bullet).is_err());
        assert_eq!(app.world().resource::<Score>().0, 1000);
        let mut q = app.world_mut().query::<&Ufo>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn enemy_bullet_hits_player_loses_life() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(Lives(3));
        app.world_mut().spawn((
            Player, Transform::from_xyz(0.0, 0.0, 0.0), Collider { radius: 12.0 },
        ));
        app.world_mut().spawn((
            EnemyBullet { life: Timer::from_seconds(1.0, TimerMode::Once) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: 2.5 },
        ));
        app.world_mut().run_system_once(player_damage).unwrap();
        assert_eq!(app.world().resource::<Lives>().0, 2);
    }

    #[test]
    fn simultaneous_hits_cost_only_one_life() {
        use crate::ufo::{Ufo, UfoSize};
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
        app.world_mut().spawn((
            Ufo { size: UfoSize::Small, fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: UfoSize::Small.radius() },
        ));

        app.world_mut().run_system_once(player_damage).unwrap();

        assert_eq!(app.world().resource::<Lives>().0, 2);
        let mut q = app.world_mut().query::<&Player>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }
}
