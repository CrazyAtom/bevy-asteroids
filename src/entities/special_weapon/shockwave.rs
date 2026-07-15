//! 충격파: 반경 내 적탄 소멸 + 소행성/UFO 밀쳐내기 + Small 소행성 파괴(방어형 탈출기).

use bevy::prelude::*;

use crate::core::components::Velocity;
use crate::core::config::{
    EXPLOSION_PARTICLES, SHAKE_EXPLOSION, SHOCKWAVE_IMPULSE, SHOCKWAVE_RADIUS,
    SHOCKWAVE_RING_SECS, Z_SHIELD,
};
use crate::core::logic::AsteroidSize;
use crate::core::state::{GameplayEntity, Score};
use crate::entities::asteroid::Asteroid;
use crate::entities::bullet::EnemyBullet;
use crate::entities::ufo::Ufo;
use crate::fx::effects::spawn_explosion;
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

/// 이번 프레임에 적용할 충격파(발동 지점). apply_shockwave가 소비 후 제거.
#[derive(Component)]
pub struct ShockwaveBlast {
    pub center: Vec2,
}

/// 링 연출(확대 + 페이드).
#[derive(Component)]
pub(super) struct ShockRing {
    life: Timer,
}

/// 반경 판정(순수).
pub(super) fn shockwave_hits(center: Vec2, radius: f32, pos: Vec2) -> bool {
    center.distance_squared(pos) <= radius * radius
}

/// 발동: Blast + 링 연출 스폰. 디스패치에서 호출.
pub(super) fn fire(commands: &mut Commands, assets: &SpriteAssets, pos: Vec2) {
    commands.spawn(ShockwaveBlast { center: pos });
    commands.spawn((
        ShockRing { life: Timer::from_seconds(SHOCKWAVE_RING_SECS, TimerMode::Once) },
        Sprite {
            image: assets.shield.clone(),
            custom_size: Some(Vec2::splat(10.0)),
            color: Color::srgba(0.5, 1.0, 1.0, 0.8),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, Z_SHIELD),
        GameplayEntity,
    ));
}

/// Blast 적용: 적탄 소멸, Small 소행성 파괴(점수+폭발), 소행성/UFO 밀쳐내기.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn apply_shockwave(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut score: ResMut<Score>,
    mut shake: MessageWriter<ShakeEvent>,
    blasts: Query<(Entity, &ShockwaveBlast)>,
    enemy_bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
    mut asteroids: Query<(Entity, &Transform, &mut Velocity, &Asteroid)>,
    mut ufos: Query<(Entity, &Transform, &mut Velocity), (With<Ufo>, Without<Asteroid>)>,
) {
    for (blast_e, blast) in &blasts {
        let c = blast.center;
        for (e, tf) in &enemy_bullets {
            if shockwave_hits(c, SHOCKWAVE_RADIUS, tf.translation.truncate()) {
                commands.entity(e).try_despawn();
            }
        }
        for (e, tf, mut vel, asteroid) in &mut asteroids {
            let p = tf.translation.truncate();
            if !shockwave_hits(c, SHOCKWAVE_RADIUS, p) {
                continue;
            }
            if asteroid.size == AsteroidSize::Small {
                // Small은 파괴(점수+폭발, 분열 없음)
                commands.entity(e).try_despawn();
                score.0 += asteroid.size.score();
                spawn_explosion(&mut commands, &assets, p, EXPLOSION_PARTICLES);
            } else {
                // 중심 → 바깥으로 강하게 밀침
                let out = (p - c).normalize_or_zero();
                vel.0 = out * SHOCKWAVE_IMPULSE;
            }
        }
        for (_e, tf, mut vel) in &mut ufos {
            let p = tf.translation.truncate();
            if shockwave_hits(c, SHOCKWAVE_RADIUS, p) {
                let out = (p - c).normalize_or_zero();
                vel.0 = out * SHOCKWAVE_IMPULSE;
            }
        }
        shake.write(ShakeEvent(SHAKE_EXPLOSION));
        commands.entity(blast_e).despawn();
    }
}

/// 링을 SHOCKWAVE_RADIUS*2까지 키우며 페이드아웃 후 제거.
pub(super) fn ring_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut rings: Query<(Entity, &mut ShockRing, &mut Sprite)>,
) {
    for (e, mut ring, mut sprite) in &mut rings {
        ring.life.tick(time.delta());
        let f = ring.life.fraction();
        sprite.custom_size = Some(Vec2::splat(10.0 + f * (SHOCKWAVE_RADIUS * 2.0 - 10.0)));
        sprite.color.set_alpha(0.8 * (1.0 - f));
        if ring.life.is_finished() {
            commands.entity(e).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_inside_not_outside() {
        assert!(shockwave_hits(Vec2::ZERO, 220.0, Vec2::new(200.0, 0.0)));
        assert!(!shockwave_hits(Vec2::ZERO, 220.0, Vec2::new(230.0, 0.0)));
    }
}
