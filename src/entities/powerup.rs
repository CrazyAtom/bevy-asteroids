use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{Collider, Velocity};
use crate::core::config::{
    POWERUP_DRIFT_SPEED, POWERUP_LIFETIME_SECS, POWERUP_MAGNET_RANGE, POWERUP_MAGNET_SPEED,
    POWERUP_RADIUS, RAPID_FIRE_SECS, SHIELD_SECS, SPREAD_MAX_LEVEL, SPREAD_SECS, Z_ENTITY,
};
use crate::core::state::{GameState, GameplayEntity, Lives};
use crate::entities::player::{Player, RapidFire, Shield, Spread};
use crate::entities::special_weapon::{pick_weapon_kind, weapon_icon, SpecialWeapon, SpecialWeaponKind};
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::sprites::{sprite_size_for, SpriteAssets};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerupKind {
    Shield,
    RapidFire,
    Spread,
    ExtraLife,
    SpecialWeapon(SpecialWeaponKind),
}

#[derive(Component)]
pub struct Powerup {
    pub kind: PowerupKind,
    pub life: Timer,
}

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (powerup_lifetime, attract_powerups, collect_powerup)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// roll(0..1)을 5개 버킷으로 나눠 종류를 고른다(특수무기는 낮은 확률).
pub fn pick_powerup_kind(roll: f32) -> PowerupKind {
    match roll {
        r if r < 0.25 => PowerupKind::Shield,
        r if r < 0.50 => PowerupKind::RapidFire,
        r if r < 0.72 => PowerupKind::Spread,
        r if r < 0.88 => PowerupKind::ExtraLife,
        r => PowerupKind::SpecialWeapon(pick_weapon_kind((r - 0.88) / 0.12)),
    }
}

/// PowerupKind → SpriteAssets.powerup 배열 인덱스.
pub fn powerup_sprite_index(kind: PowerupKind) -> usize {
    match kind {
        PowerupKind::Shield => 0,
        PowerupKind::RapidFire => 1,
        PowerupKind::Spread => 2,
        PowerupKind::ExtraLife => 3,
        PowerupKind::SpecialWeapon(_) => 4,
    }
}

pub fn spawn_powerup(
    commands: &mut Commands,
    assets: &SpriteAssets,
    kind: PowerupKind,
    position: Vec2,
) {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let velocity = Vec2::new(angle.cos(), angle.sin()) * POWERUP_DRIFT_SPEED;
    let image = match kind {
        PowerupKind::SpecialWeapon(w) => weapon_icon(w, assets),
        _ => assets.powerup[powerup_sprite_index(kind)].clone(),
    };
    commands.spawn((
        Powerup { kind, life: Timer::from_seconds(POWERUP_LIFETIME_SECS, TimerMode::Once) },
        Sprite {
            image,
            custom_size: Some(sprite_size_for(POWERUP_RADIUS)),
            ..default()
        },
        Transform::from_translation(position.extend(Z_ENTITY)),
        Velocity(velocity),
        Collider { radius: POWERUP_RADIUS },
        GameplayEntity,
    ));
}

fn powerup_lifetime(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Powerup)>) {
    for (entity, mut p) in &mut q {
        p.life.tick(time.delta());
        if p.life.is_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

/// 파워업이 자석 범위 안이면 플레이어를 향하는 속도를, 아니면 None을 반환.
pub fn magnet_velocity(powerup_pos: Vec2, player_pos: Vec2, range: f32, speed: f32) -> Option<Vec2> {
    let to_player = player_pos - powerup_pos;
    let dist = to_player.length();
    if dist < range && dist > 0.01 {
        Some(to_player / dist * speed)
    } else {
        None
    }
}

/// 자석 범위 안의 파워업 속도를 플레이어 방향으로 덮어써 끌어당긴다.
fn attract_powerups(
    players: Query<&Transform, With<Player>>,
    mut powerups: Query<(&Transform, &mut Velocity), With<Powerup>>,
) {
    let Ok(player_tf) = players.single() else { return };
    let ppos = player_tf.translation.truncate();
    for (tf, mut vel) in &mut powerups {
        if let Some(v) = magnet_velocity(
            tf.translation.truncate(),
            ppos,
            POWERUP_MAGNET_RANGE,
            POWERUP_MAGNET_SPEED,
        ) {
            vel.0 = v;
        }
    }
}

/// 우주선이 파워업에 닿으면 효과를 적용한다. (실드/연사/확산탄/특수무기는 후속 태스크에서 각자 컴포넌트를 부여)
fn collect_powerup(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut sfx: MessageWriter<SfxEvent>,
    players: Query<(Entity, &Transform, &Collider), With<Player>>,
    powerups: Query<(Entity, &Transform, &Collider, &Powerup)>,
    mut special_q: Query<&mut SpecialWeapon>,
    spread_q: Query<&Spread>,
) {
    let Ok((player_entity, player_tf, player_col)) = players.single() else {
        return;
    };
    for (powerup_entity, powerup_tf, powerup_col, powerup) in &powerups {
        if crate::core::logic::circles_overlap(
            player_tf.translation.truncate(),
            player_col.radius,
            powerup_tf.translation.truncate(),
            powerup_col.radius,
        ) {
            match powerup.kind {
                PowerupKind::ExtraLife => lives.0 += 1,
                PowerupKind::Shield => {
                    commands
                        .entity(player_entity)
                        .insert(Shield(Timer::from_seconds(SHIELD_SECS, TimerMode::Once)));
                }
                PowerupKind::RapidFire => {
                    commands.entity(player_entity).insert(RapidFire(
                        Timer::from_seconds(RAPID_FIRE_SECS, TimerMode::Once),
                    ));
                }
                PowerupKind::Spread => {
                    // 재획득 시 레벨+1(최대 SPREAD_MAX_LEVEL), 시간은 매번 리셋.
                    let level = spread_q
                        .get(player_entity)
                        .map(|s| (s.level + 1).min(SPREAD_MAX_LEVEL))
                        .unwrap_or(1);
                    commands.entity(player_entity).insert(Spread {
                        timer: Timer::from_seconds(SPREAD_SECS, TimerMode::Once),
                        level,
                    });
                }
                PowerupKind::SpecialWeapon(kind) => {
                    if let Ok(mut weapon) = special_q.get_mut(player_entity) {
                        weapon.queue.push_back(kind);
                    }
                }
            }
            commands.entity(powerup_entity).try_despawn();
            sfx.write(SfxEvent(Sfx::Pickup));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn pick_kind_covers_all_buckets() {
        // 경계값이 유효한 종류를 반환하는지(패닉/누락 없음)
        for i in 0..=10 {
            let _ = pick_powerup_kind(i as f32 / 10.0);
        }
        assert!(matches!(pick_powerup_kind(0.0), PowerupKind::Shield));
        // 마지막 버킷은 특수무기
        assert!(matches!(pick_powerup_kind(0.99), PowerupKind::SpecialWeapon(_)));
    }

    #[test]
    fn powerup_index_is_stable_and_distinct() {
        use PowerupKind::*;
        assert_eq!(powerup_sprite_index(Shield), 0);
        assert_eq!(powerup_sprite_index(RapidFire), 1);
        assert_eq!(powerup_sprite_index(Spread), 2);
        assert_eq!(powerup_sprite_index(ExtraLife), 3);
        assert_eq!(powerup_sprite_index(SpecialWeapon(SpecialWeaponKind::LaserBeam)), 4);
    }

    #[test]
    fn magnet_pulls_only_within_range() {
        // 범위 밖 → None
        assert!(magnet_velocity(Vec2::ZERO, Vec2::new(500.0, 0.0), 140.0, 320.0).is_none());
        // 범위 안 → 플레이어 방향 단위벡터 * speed
        let v = magnet_velocity(Vec2::ZERO, Vec2::new(100.0, 0.0), 140.0, 320.0).unwrap();
        assert!((v.x - 320.0).abs() < 1e-3);
        assert!(v.y.abs() < 1e-3);
    }

    #[test]
    fn spawn_powerup_creates_entity() {
        let mut app = App::new();
        let assets = crate::fx::sprites::dummy_sprite_assets();
        app.world_mut()
            .run_system_once(move |mut c: Commands| {
                spawn_powerup(&mut c, &assets, PowerupKind::ExtraLife, Vec2::ZERO);
            })
            .unwrap();
        let mut q = app.world_mut().query::<&Powerup>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }
}
