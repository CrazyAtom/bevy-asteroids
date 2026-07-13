use std::collections::HashSet;

use bevy::prelude::*;

use crate::entities::asteroid::{random_velocity, spawn_asteroid, Asteroid};
use crate::entities::bullet::{Bullet, EnemyBullet};
use crate::entities::powerup::{pick_powerup_kind, spawn_powerup};
use crate::core::components::Collider;
use crate::core::config::EXPLOSION_PARTICLES;
use crate::core::config::POWERUP_DROP_CHANCE;
use crate::core::config::{BEAM_LENGTH, BEAM_WIDTH, SHAKE_EXPLOSION, SHAKE_HIT};
use crate::fx::effects::spawn_explosion;
use crate::fx::sprites::SpriteAssets;
use crate::fx::shake::ShakeEvent;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::core::logic::{circles_overlap, next_asteroid_size, segment_circle_hit};
use crate::entities::player::{spawn_player_entity, Player, Shield, SpecialBeam};
use crate::core::state::{GameState, Lives, Score};
use crate::entities::boss::Boss;
use crate::entities::ufo::Ufo;
use rand::RngExt;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (bullet_vs_asteroid, bullet_vs_ufo, player_damage, beam_vs_targets)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn bullet_vs_asteroid(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut shake: MessageWriter<ShakeEvent>,
    mut sfx: MessageWriter<SfxEvent>,
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
                spawn_explosion(&mut commands, &assets, asteroid_tf.translation.truncate(), EXPLOSION_PARTICLES);
                shake.write(ShakeEvent(SHAKE_EXPLOSION));
                sfx.write(SfxEvent(Sfx::Explosion));

                if let Some(next) = next_asteroid_size(asteroid.size) {
                    let base = asteroid_tf.translation.truncate();
                    for _ in 0..2 {
                        spawn_asteroid(&mut commands, &assets, next, base, random_velocity(next));
                    }
                }
                {
                    let mut rng = rand::rng();
                    if rng.random_range(0.0..1.0) < POWERUP_DROP_CHANCE {
                        spawn_powerup(&mut commands, &assets, pick_powerup_kind(rng.random_range(0.0..1.0)), asteroid_tf.translation.truncate());
                    }
                }
                break; // 이 총알은 소진됨
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bullet_vs_ufo(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut shake: MessageWriter<ShakeEvent>,
    mut sfx: MessageWriter<SfxEvent>,
    bullets: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo)>,
) {
    let mut destroyed: HashSet<Entity> = HashSet::new();
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
                spawn_explosion(&mut commands, &assets, ufo_tf.translation.truncate(), EXPLOSION_PARTICLES);
                shake.write(ShakeEvent(SHAKE_EXPLOSION));
                sfx.write(SfxEvent(Sfx::Explosion));
                {
                    let mut rng = rand::rng();
                    if rng.random_range(0.0..1.0) < POWERUP_DROP_CHANCE {
                        spawn_powerup(&mut commands, &assets, pick_powerup_kind(rng.random_range(0.0..1.0)), ufo_tf.translation.truncate());
                    }
                }
                break;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn beam_vs_targets(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut sfx: MessageWriter<SfxEvent>,
    beams: Query<&SpecialBeam>,
    asteroids: Query<(Entity, &Transform, &Collider, &Asteroid)>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo)>,
) {
    let mut destroyed: HashSet<Entity> = HashSet::new();

    for beam in &beams {
        for (e, tf, col, asteroid) in &asteroids {
            if destroyed.contains(&e) {
                continue;
            }
            if segment_circle_hit(beam.origin, beam.dir, BEAM_LENGTH, BEAM_WIDTH * 0.5, tf.translation.truncate(), col.radius) {
                destroyed.insert(e);
                commands.entity(e).despawn();
                score.0 += asteroid.size.score();
                spawn_explosion(&mut commands, &assets, tf.translation.truncate(), EXPLOSION_PARTICLES);
                sfx.write(SfxEvent(Sfx::Explosion));
            }
        }
        for (e, tf, col, ufo) in &ufos {
            if destroyed.contains(&e) {
                continue;
            }
            if segment_circle_hit(beam.origin, beam.dir, BEAM_LENGTH, BEAM_WIDTH * 0.5, tf.translation.truncate(), col.radius) {
                destroyed.insert(e);
                commands.entity(e).despawn();
                score.0 += ufo.size.score();
                spawn_explosion(&mut commands, &assets, tf.translation.truncate(), EXPLOSION_PARTICLES);
                sfx.write(SfxEvent(Sfx::Explosion));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn player_damage(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    mut shake: MessageWriter<ShakeEvent>,
    players: Query<(Entity, &Transform, &Collider, Option<&Shield>), With<Player>>,
    asteroids: Query<(&Transform, &Collider), With<Asteroid>>,
    ufos: Query<(&Transform, &Collider), With<Ufo>>,
    bosses: Query<(&Transform, &Collider), With<Boss>>,
    enemy_bullets: Query<(Entity, &Transform, &Collider), With<EnemyBullet>>,
) {
    let Ok((player_entity, player_tf, player_col, shield)) = players.single() else {
        return;
    };
    if shield.is_some() {
        return; // 실드 중 무피해
    }
    let ppos = player_tf.translation.truncate();
    let pr = player_col.radius;

    // 위험요소 확인 우선순위: 소행성 → UFO → 적 총알. 적 총알에 맞은 경우에만 해당 엔티티를 디스폰한다.
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
        for (b_tf, b_col) in &bosses {
            if circles_overlap(ppos, pr, b_tf.translation.truncate(), b_col.radius) {
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

    spawn_explosion(&mut commands, &assets, ppos, EXPLOSION_PARTICLES);
    shake.write(ShakeEvent(SHAKE_HIT));
    lives.0 = lives.0.saturating_sub(1);
    commands.entity(player_entity).despawn();
    if lives.0 == 0 {
        next_state.set(GameState::GameOver);
    } else {
        spawn_player_entity(&mut commands, &assets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::components::Collider;
    use crate::core::logic::AsteroidSize;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn bullet_splits_large_asteroid_and_scores() {
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
        app.add_message::<SfxEvent>();
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
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
        app.add_message::<SfxEvent>();
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
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
        app.add_message::<SfxEvent>();
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
        let mut q = app.world_mut().query::<&crate::fx::effects::Particle>();
        assert!(q.iter(app.world()).count() > 0);
    }

    #[test]
    fn player_hit_loses_life_and_respawns() {
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
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
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
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
        use crate::entities::ufo::{Ufo, UfoSize};
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
        app.add_message::<SfxEvent>();
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
    fn two_overlapping_beams_score_target_once() {
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<SfxEvent>();
        app.insert_resource(Score(0));
        app.world_mut().spawn(SpecialBeam {
            life: Timer::from_seconds(0.4, TimerMode::Once),
            origin: Vec2::ZERO,
            dir: Vec2::Y,
            damaged_boss: false,
        });
        app.world_mut().spawn(SpecialBeam {
            life: Timer::from_seconds(0.4, TimerMode::Once),
            origin: Vec2::ZERO,
            dir: Vec2::Y,
            damaged_boss: false,
        });
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large },
            Transform::from_xyz(0.0, 100.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));

        app.world_mut().run_system_once(beam_vs_targets).unwrap();

        assert_eq!(app.world().resource::<Score>().0, AsteroidSize::Large.score());
        let mut q = app.world_mut().query::<&Asteroid>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn enemy_bullet_hits_player_loses_life() {
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
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
    fn shielded_player_takes_no_damage() {
        use crate::entities::player::{Player, Shield};
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
        app.add_message::<SfxEvent>();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(Lives(3));
        app.world_mut().spawn((
            Player, Transform::from_xyz(0.0, 0.0, 0.0), Collider { radius: 12.0 },
            Shield(Timer::from_seconds(5.0, TimerMode::Once)),
        ));
        app.world_mut().spawn((
            Asteroid { size: AsteroidSize::Large }, Transform::from_xyz(0.0, 0.0, 0.0),
            Collider { radius: AsteroidSize::Large.radius() },
        ));
        app.world_mut().run_system_once(player_damage).unwrap();
        assert_eq!(app.world().resource::<Lives>().0, 3); // 무피해
    }

    #[test]
    fn simultaneous_hits_cost_only_one_life() {
        use crate::entities::ufo::{Ufo, UfoSize};
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.add_message::<crate::fx::shake::ShakeEvent>();
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
