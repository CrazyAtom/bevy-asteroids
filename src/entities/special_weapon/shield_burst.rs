//! 공격형 실드: 몇 초간 무적(기존 Shield 재사용) + 접촉한 소행성/UFO/적탄을 파괴.
//! 표현은 버블 없이 우주선 자체 번쩍임(주황 고주파 틴트) + 크기 펄스(±5%), 만료 시 원복.

use bevy::prelude::*;

use crate::core::components::Collider;
use crate::core::config::{
    BURST_FLASH_HZ, BURST_PULSE, EXPLOSION_PARTICLES, SHIELD_BURST_SECS, SHIP_COLLIDER_RADIUS,
};
use crate::core::logic::circles_overlap;
use crate::core::state::Score;
use crate::entities::asteroid::Asteroid;
use crate::entities::bullet::EnemyBullet;
use crate::entities::player::{Player, Shield};
use crate::entities::ufo::Ufo;
use crate::fx::effects::spawn_explosion;
use crate::fx::sprites::{sprite_size_for, SpriteAssets};

/// 공격형 실드 지속 타이머(플레이어에 부착).
#[derive(Component)]
pub struct ShieldBurst {
    pub timer: Timer,
}

/// 번쩍임 on/off(고주파 토글, 순수).
pub(super) fn burst_flash_on(t: f32) -> bool {
    (t * BURST_FLASH_HZ).sin() > 0.0
}

/// 크기 펄스 배율(1±BURST_PULSE, 순수).
pub(super) fn burst_pulse_scale(t: f32) -> f32 {
    1.0 + BURST_PULSE * (t * BURST_FLASH_HZ * 0.8).sin()
}

/// 발동: ShieldBurst + 기존 Shield(같은 시간, 무적 재사용) 부여. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, player: Entity) {
    commands.entity(player).insert((
        ShieldBurst { timer: Timer::from_seconds(SHIELD_BURST_SECS, TimerMode::Once) },
        Shield(Timer::from_seconds(SHIELD_BURST_SECS, TimerMode::Once)),
    ));
}

/// 지속 관리 + 우주선 번쩍임/펄스. 만료 프레임에 색·크기 원복(필수).
pub(super) fn burst_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut ShieldBurst, &mut Sprite), With<Player>>,
) {
    for (e, mut burst, mut sprite) in &mut q {
        burst.timer.tick(time.delta());
        if burst.timer.is_finished() {
            sprite.color = Color::WHITE;
            sprite.custom_size = Some(sprite_size_for(SHIP_COLLIDER_RADIUS));
            commands.entity(e).remove::<ShieldBurst>();
            continue;
        }
        let t = time.elapsed_secs();
        // HDR 금빛(색값 1.0 초과)으로 우주선을 실제 발광(bloom)시킨다. 두 밝기를 오가 '빛나는 오라'.
        sprite.color = if burst_flash_on(t) {
            Color::srgb(4.2, 3.3, 1.3) // 밝은 금빛 발광
        } else {
            Color::srgb(2.6, 2.0, 0.85) // 약한 금빛 발광
        };
        sprite.custom_size = Some(sprite_size_for(SHIP_COLLIDER_RADIUS) * burst_pulse_scale(t));
    }
}

/// 지속 중 접촉한 소행성/UFO/적탄 파괴(점수+폭발, 분열 없음; 보스는 무적만).
#[allow(clippy::type_complexity)]
pub(super) fn burst_contact_destroy(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    players: Query<(&Transform, &Collider), (With<Player>, With<ShieldBurst>)>,
    asteroids: Query<(Entity, &Transform, &Collider, &Asteroid), Without<Player>>,
    ufos: Query<(Entity, &Transform, &Collider, &Ufo), Without<Player>>,
    enemy_bullets: Query<(Entity, &Transform, &Collider), (With<EnemyBullet>, Without<Player>)>,
) {
    let Ok((ptf, pcol)) = players.single() else { return };
    let ppos = ptf.translation.truncate();
    for (e, tf, col, asteroid) in &asteroids {
        if circles_overlap(ppos, pcol.radius, tf.translation.truncate(), col.radius) {
            commands.entity(e).try_despawn();
            score.0 += asteroid.size.score();
            spawn_explosion(&mut commands, &assets, tf.translation.truncate(), EXPLOSION_PARTICLES);
        }
    }
    for (e, tf, col, ufo) in &ufos {
        if circles_overlap(ppos, pcol.radius, tf.translation.truncate(), col.radius) {
            commands.entity(e).try_despawn();
            score.0 += ufo.size.score();
            spawn_explosion(&mut commands, &assets, tf.translation.truncate(), EXPLOSION_PARTICLES);
        }
    }
    for (e, tf, col) in &enemy_bullets {
        if circles_overlap(ppos, pcol.radius, tf.translation.truncate(), col.radius) {
            commands.entity(e).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::logic::AsteroidSize;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn pulse_stays_within_band_and_flash_toggles() {
        let mut saw_on = false;
        let mut saw_off = false;
        for i in 0..200 {
            let t = i as f32 * 0.01;
            let s = burst_pulse_scale(t);
            assert!((1.0 - BURST_PULSE - 1e-4..=1.0 + BURST_PULSE + 1e-4).contains(&s));
            if burst_flash_on(t) { saw_on = true } else { saw_off = true }
        }
        assert!(saw_on && saw_off, "번쩍임이 켜짐/꺼짐을 오가야 한다");
    }

    #[test]
    fn burst_contact_destroys_overlapping_asteroid() {
        let mut app = App::new();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        app.insert_resource(Score(0));
        app.world_mut().spawn((
            Player,
            ShieldBurst { timer: Timer::from_seconds(SHIELD_BURST_SECS, TimerMode::Once) },
            Transform::default(),
            Collider { radius: SHIP_COLLIDER_RADIUS },
        ));
        let asteroid_entity = app
            .world_mut()
            .spawn((
                Asteroid { size: AsteroidSize::Small },
                Transform::default(),
                Collider { radius: AsteroidSize::Small.radius() },
            ))
            .id();
        app.world_mut().run_system_once(burst_contact_destroy).unwrap();
        assert!(app.world().get_entity(asteroid_entity).is_err());
        assert!(app.world().resource::<Score>().0 > 0);
    }
}
