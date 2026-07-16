//! 특수무기(X 키) — 카트라이더 방식 FIFO 큐. 획득한 순서대로 쌓이고 X를 누르면
//! 맨 앞 무기부터 발동한다. 무기별 로직은 서브모듈에 있고 여기의 match 디스패치가
//! 위임한다(새 무기 = 변형 추가 → 컴파일러가 모든 지점 강제).

pub mod beam;
pub mod missile;
pub mod nova;
pub mod shield_burst;
pub mod shockwave;

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::core::config::SHAKE_SPECIAL;
use crate::core::state::RunPhase;
use crate::entities::player::Player;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::SpriteAssets;

pub use beam::SpecialBeam; // collision·boss의 기존 경로 유지

/// 특수무기 종류. 파워업 드롭·큐·HUD 아이콘이 모두 이 enum을 공유한다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpecialWeaponKind {
    LaserBeam,
    ScatterNova,
    HomingMissile,
    Shockwave,
    ShieldBurst,
}

/// 획득 순서(FIFO) 큐. 앞 = 다음 발동. 내부는 무제한, HUD는 앞 HUD_QUEUE_SLOTS개 표시.
#[derive(Component)]
pub struct SpecialWeapon {
    pub queue: VecDeque<SpecialWeaponKind>,
}

/// kind → 드롭/HUD 아이콘 핸들.
pub fn weapon_icon(kind: SpecialWeaponKind, assets: &SpriteAssets) -> Handle<Image> {
    match kind {
        SpecialWeaponKind::LaserBeam => assets.powerup[4].clone(),
        SpecialWeaponKind::ScatterNova => assets.weapon_nova.clone(),
        SpecialWeaponKind::HomingMissile => assets.weapon_missile.clone(),
        SpecialWeaponKind::Shockwave => assets.weapon_shockwave.clone(),
        SpecialWeaponKind::ShieldBurst => assets.weapon_burst.clone(),
    }
}

/// 특수무기 드롭 종류 추첨(roll 0..1 균등 버킷). 파워업 드롭에서 사용.
pub fn pick_weapon_kind(roll: f32) -> SpecialWeaponKind {
    match roll {
        r if r < 0.2 => SpecialWeaponKind::LaserBeam,
        r if r < 0.4 => SpecialWeaponKind::ScatterNova,
        r if r < 0.6 => SpecialWeaponKind::HomingMissile,
        r if r < 0.8 => SpecialWeaponKind::Shockwave,
        _ => SpecialWeaponKind::ShieldBurst,
    }
}

pub struct SpecialWeaponPlugin;

impl Plugin for SpecialWeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                activate_special,
                beam::tick_beam,
                missile::homing_steer,
                shockwave::apply_shockwave,
                shockwave::ring_fade,
                shield_burst::burst_tick,
                shield_burst::burst_contact_destroy,
            )
                .run_if(in_state(RunPhase::Running)),
        );
    }
}

/// X = 큐 맨 앞 무기 발동(pop). 무기별 fire는 서브모듈로 위임.
#[allow(clippy::type_complexity)]
fn activate_special(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut shake: MessageWriter<ShakeEvent>,
    mut query: Query<(Entity, &Transform, &mut SpecialWeapon), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let Ok((player_entity, transform, mut weapon)) = query.single_mut() else { return };
    let Some(kind) = weapon.queue.pop_front() else { return };
    match kind {
        SpecialWeaponKind::LaserBeam => beam::fire(&mut commands, &assets, transform),
        SpecialWeaponKind::ScatterNova => {
            nova::fire(&mut commands, &assets, transform.translation.truncate())
        }
        SpecialWeaponKind::HomingMissile => missile::fire(&mut commands, &assets, transform),
        SpecialWeaponKind::Shockwave => {
            shockwave::fire(&mut commands, &assets, transform.translation.truncate())
        }
        SpecialWeaponKind::ShieldBurst => shield_burst::fire(&mut commands, player_entity),
    }
    sfx.write(SfxEvent(Sfx::Special));
    shake.write(ShakeEvent(SHAKE_SPECIAL));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn app_with_input_and_assets(pressed: bool) -> App {
        let mut app = App::new();
        app.add_message::<SfxEvent>();
        app.add_message::<ShakeEvent>();
        app.insert_resource(crate::fx::sprites::dummy_sprite_assets());
        let mut keys = ButtonInput::<KeyCode>::default();
        if pressed {
            keys.press(KeyCode::KeyX);
        }
        app.insert_resource(keys);
        app.insert_resource(Time::<()>::default());
        app
    }

    #[test]
    fn x_fires_front_and_consumes_in_order() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((
            Player,
            Transform::default(),
            SpecialWeapon {
                queue: VecDeque::from(vec![SpecialWeaponKind::LaserBeam, SpecialWeaponKind::LaserBeam]),
            },
        ));
        app.world_mut().run_system_once(activate_special).unwrap();
        // 맨 앞 1개 소모 + 빔 1개 스폰
        let mut beams = app.world_mut().query::<&SpecialBeam>();
        assert_eq!(beams.iter(app.world()).count(), 1);
        let mut wq = app.world_mut().query::<&SpecialWeapon>();
        assert_eq!(wq.single(app.world()).unwrap().queue.len(), 1);
    }

    #[test]
    fn empty_queue_fires_nothing() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((Player, Transform::default(), SpecialWeapon { queue: VecDeque::new() }));
        app.world_mut().run_system_once(activate_special).unwrap();
        let mut beams = app.world_mut().query::<&SpecialBeam>();
        assert_eq!(beams.iter(app.world()).count(), 0);
    }

    #[test]
    fn scatter_nova_spawns_nova_bullets() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((
            Player,
            Transform::default(),
            SpecialWeapon { queue: VecDeque::from(vec![SpecialWeaponKind::ScatterNova]) },
        ));
        app.world_mut().run_system_once(activate_special).unwrap();
        let mut bullets = app.world_mut().query::<&crate::entities::bullet::Bullet>();
        assert_eq!(bullets.iter(app.world()).count(), crate::core::config::NOVA_BULLETS);
    }

    #[test]
    fn homing_missile_spawns_missiles() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((
            Player,
            Transform::default(),
            SpecialWeapon { queue: VecDeque::from(vec![SpecialWeaponKind::HomingMissile]) },
        ));
        app.world_mut().run_system_once(activate_special).unwrap();
        let mut homing = app.world_mut().query::<&missile::Homing>();
        assert_eq!(homing.iter(app.world()).count(), crate::core::config::MISSILE_COUNT);
    }

    #[test]
    fn shockwave_spawns_blast() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((
            Player,
            Transform::default(),
            SpecialWeapon { queue: VecDeque::from(vec![SpecialWeaponKind::Shockwave]) },
        ));
        app.world_mut().run_system_once(activate_special).unwrap();
        let mut blasts = app.world_mut().query::<&shockwave::ShockwaveBlast>();
        assert_eq!(blasts.iter(app.world()).count(), 1);
    }

    #[test]
    fn shield_burst_grants_burst_and_shield() {
        let mut app = app_with_input_and_assets(true);
        app.world_mut().spawn((
            Player,
            Transform::default(),
            SpecialWeapon { queue: VecDeque::from(vec![SpecialWeaponKind::ShieldBurst]) },
        ));
        app.world_mut().run_system_once(activate_special).unwrap();
        let mut q = app
            .world_mut()
            .query_filtered::<(), (With<Player>, With<shield_burst::ShieldBurst>, With<crate::entities::player::Shield>)>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn pick_weapon_kind_covers_five_buckets() {
        assert_eq!(pick_weapon_kind(0.0), SpecialWeaponKind::LaserBeam);
        assert_eq!(pick_weapon_kind(0.3), SpecialWeaponKind::ScatterNova);
        assert_eq!(pick_weapon_kind(0.5), SpecialWeaponKind::HomingMissile);
        assert_eq!(pick_weapon_kind(0.7), SpecialWeaponKind::Shockwave);
        assert_eq!(pick_weapon_kind(0.99), SpecialWeaponKind::ShieldBurst);
    }
}
