use bevy::prelude::*;

use crate::core::components::{Collider, Velocity, Wrapping};
use crate::core::config::{
    BEAM_LENGTH, BEAM_LIFETIME_SECS, BEAM_WIDTH, HYPERSPACE_COOLDOWN_SECS, SHAKE_SPECIAL,
    SHIP_BRAKE_RATE, SHIP_COLLIDER_RADIUS, SHIP_DAMPING, SHIP_MAX_SPEED, SHIP_ROTATION_SPEED,
    SHIP_THRUST, SHIP_TURN_ACCEL, SHIP_TURN_MIN, SPAWN_INVINCIBILITY_SECS, STARTING_SPECIAL_CHARGES,
    Z_BEAM, Z_ENTITY, Z_FLAME,
};
use crate::core::logic::apply_brake;
use crate::core::state::{GameState, GameplayEntity};
use crate::entities::powerup::SpecialWeaponKind;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::shake::ShakeEvent;
use crate::fx::sprites::{sprite_size_for, SpriteAssets};

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct EngineState {
    pub thrusting: bool,
    pub braking: bool,
    /// 현재 회전 각속도(누르는 동안 ramp up). 떼면 0으로 리셋.
    pub turn_speed: f32,
}

#[derive(Component)]
pub struct FireCooldown(pub Timer);

#[derive(Component)]
pub struct Shield(pub Timer);

#[derive(Component)]
pub struct RapidFire(pub Timer);

#[derive(Component)]
pub struct Spread(pub Timer);

#[derive(Component)]
pub struct HyperspaceCooldown(pub Timer);

#[derive(Component)]
pub struct SpecialWeapon {
    pub kind: SpecialWeaponKind,
    pub charges: u32,
}

#[derive(Component)]
pub struct SpecialBeam {
    pub life: Timer,
    pub origin: Vec2,
    pub dir: Vec2,
    /// 보스에게는 프레임당이 아니라 빔 1회당 한 번만 피해를 준다.
    pub damaged_boss: bool,
}

/// 추진/브레이크 시 우주선 뒤/앞에 나타나는 화염 스프라이트(자식 엔티티) 마커.
#[derive(Component)]
pub struct Flame;

/// 우주선 실드 버블 스프라이트(자식 엔티티) 마커. Shield 컴포넌트 유무로 가시성 토글.
#[derive(Component)]
pub struct ShieldSprite;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_input,
                    update_flame,
                    shield_tick,
                    update_shield_sprite,
                    tick_fire_mods,
                    activate_special,
                    tick_beam,
                    hyperspace,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                apply_ship_damping.run_if(in_state(GameState::Playing)),
            );
    }
}

/// 화면 경계 내(90% 범위) 임의 좌표를 반환한다. 하이퍼스페이스 순간이동 목적지 계산에 쓰인다.
pub fn random_hyperspace_position(half: Vec2) -> Vec2 {
    use rand::RngExt;
    let mut rng = rand::rng();
    Vec2::new(
        rng.random_range(-half.x * 0.9..half.x * 0.9),
        rng.random_range(-half.y * 0.9..half.y * 0.9),
    )
}

pub fn spawn_player_entity(commands: &mut Commands, assets: &SpriteAssets) {
    commands
        .spawn((
            Player,
            Sprite {
                image: assets.ship.clone(),
                custom_size: Some(sprite_size_for(SHIP_COLLIDER_RADIUS)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, Z_ENTITY),
            Velocity(Vec2::ZERO),
            Collider { radius: SHIP_COLLIDER_RADIUS },
            Wrapping,
            GameplayEntity,
            EngineState::default(),
            FireCooldown({
                let mut t = Timer::from_seconds(crate::core::config::FIRE_INTERVAL, TimerMode::Once);
                t.tick(t.duration()); // 시작 시 준비완료
                t
            }),
            SpecialWeapon { kind: SpecialWeaponKind::LaserBeam, charges: STARTING_SPECIAL_CHARGES },
            HyperspaceCooldown({
                let mut t = Timer::from_seconds(HYPERSPACE_COOLDOWN_SECS, TimerMode::Once);
                t.tick(t.duration()); // 시작 시 준비완료
                t
            }),
            // (재)스폰 직후 잠깐 무적: 리스폰 지점에 위험요소가 있어도 즉사 연쇄를 막는다.
            Shield(Timer::from_seconds(SPAWN_INVINCIBILITY_SECS, TimerMode::Once)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Flame,
                Sprite {
                    image: assets.flame.clone(),
                    custom_size: Some(Vec2::new(12.0, 16.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, -16.0, Z_FLAME),
                Visibility::Hidden,
            ));
            parent.spawn((
                ShieldSprite,
                Sprite {
                    image: assets.shield.clone(),
                    custom_size: Some(Vec2::splat(48.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, crate::core::config::Z_SHIELD),
                Visibility::Hidden,
            ));
        });
}

fn spawn_player(mut commands: Commands, assets: Res<SpriteAssets>) {
    spawn_player_entity(&mut commands, &assets);
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut EngineState), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity, mut engine) in &mut query {
        let mut turn = 0.0;
        if keys.pressed(KeyCode::ArrowLeft) {
            turn += 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            turn -= 1.0;
        }
        // 회전 각속도 ramp: 살짝 누르면 느리게(미세 조준), 계속 누르면 상한까지 가속.
        if turn != 0.0 {
            engine.turn_speed =
                (engine.turn_speed.max(SHIP_TURN_MIN) + SHIP_TURN_ACCEL * dt).min(SHIP_ROTATION_SPEED);
        } else {
            engine.turn_speed = 0.0;
        }
        transform.rotate_z(turn * engine.turn_speed * dt);

        engine.thrusting = keys.pressed(KeyCode::ArrowUp);
        engine.braking = keys.pressed(KeyCode::ArrowDown);

        if engine.thrusting {
            let forward = (transform.rotation * Vec3::Y).truncate();
            velocity.0 += forward * SHIP_THRUST * dt;
        }
        if engine.braking {
            velocity.0 = apply_brake(velocity.0, SHIP_BRAKE_RATE, dt);
        }
    }
}

/// 우주선 전용 감속(마찰)과 최고 속도 제한. FixedUpdate에서 매 틱 실행돼
/// 추진을 멈추면 속도가 서서히 줄어 정지하고, 속도가 상한을 넘지 않게 한다.
/// 소행성·총알에는 적용되지 않으므로 그들은 등속을 유지한다.
fn apply_ship_damping(mut query: Query<&mut Velocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 *= SHIP_DAMPING;
        velocity.0 = velocity.0.clamp_length_max(SHIP_MAX_SPEED);
    }
}

/// 추진/브레이크 상태에 따라 화염 자식 스프라이트의 가시성과 위치(후미/전방)를 갱신한다.
fn update_flame(
    engine_q: Query<&EngineState, With<Player>>,
    mut flame_q: Query<(&mut Visibility, &mut Transform), With<Flame>>,
) {
    let Ok(engine) = engine_q.single() else { return };
    for (mut vis, mut tf) in &mut flame_q {
        if engine.thrusting {
            *vis = Visibility::Visible;
            tf.translation.y = -16.0; // 후미
            tf.rotation = Quat::IDENTITY;
        } else if engine.braking {
            *vis = Visibility::Visible;
            tf.translation.y = 16.0; // 전방
            tf.rotation = Quat::from_rotation_z(std::f32::consts::PI);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn shield_tick(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Shield)>) {
    for (entity, mut shield) in &mut q {
        shield.0.tick(time.delta());
        if shield.0.is_finished() {
            commands.entity(entity).remove::<Shield>();
        }
    }
}

fn tick_fire_mods(
    mut commands: Commands,
    time: Res<Time>,
    mut rapid: Query<(Entity, &mut RapidFire)>,
    mut spread: Query<(Entity, &mut Spread)>,
) {
    for (e, mut t) in &mut rapid {
        t.0.tick(time.delta());
        if t.0.is_finished() {
            commands.entity(e).remove::<RapidFire>();
        }
    }
    for (e, mut t) in &mut spread {
        t.0.tick(time.delta());
        if t.0.is_finished() {
            commands.entity(e).remove::<Spread>();
        }
    }
}

/// Shield 컴포넌트 유무에 따라 실드 버블 스프라이트의 가시성을 토글한다.
fn update_shield_sprite(
    player_q: Query<Has<Shield>, With<Player>>,
    mut shield_q: Query<&mut Visibility, With<ShieldSprite>>,
) {
    let Ok(has_shield) = player_q.single() else { return };
    for mut vis in &mut shield_q {
        *vis = if has_shield {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn activate_special(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut shake: MessageWriter<ShakeEvent>,
    mut query: Query<(&Transform, &mut SpecialWeapon), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    let Ok((transform, mut weapon)) = query.single_mut() else { return };
    if weapon.charges == 0 {
        return;
    }
    weapon.charges -= 1;
    let origin = transform.translation.truncate();
    let dir = (transform.rotation * Vec3::Y).truncate();
    match weapon.kind {
        SpecialWeaponKind::LaserBeam => {
            // 빔 스프라이트는 세로(+Y)로 그려짐 → dir 방향으로 회전, 원점~사거리 중앙에 배치.
            let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
            let center = origin + dir.normalize_or_zero() * (BEAM_LENGTH * 0.5);
            commands.spawn((
                SpecialBeam {
                    life: Timer::from_seconds(BEAM_LIFETIME_SECS, TimerMode::Once),
                    origin,
                    dir,
                    damaged_boss: false,
                },
                Sprite {
                    image: assets.beam.clone(),
                    custom_size: Some(Vec2::new(BEAM_WIDTH, BEAM_LENGTH)),
                    ..default()
                },
                Transform {
                    translation: center.extend(Z_BEAM),
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                GameplayEntity,
            ));
        }
    }
    sfx.write(SfxEvent(Sfx::Special));
    shake.write(ShakeEvent(SHAKE_SPECIAL));
}

/// 빔 수명 관리(그리기는 스프라이트가 담당). 판정은 collision::beam_vs_targets.
fn tick_beam(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpecialBeam)>,
) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn hyperspace(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sfx: MessageWriter<SfxEvent>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut HyperspaceCooldown), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut cooldown)) = query.single_mut() else { return };
    cooldown.0.tick(time.delta());
    if !keys.just_pressed(KeyCode::KeyH) || !cooldown.0.is_finished() {
        return;
    }
    let pos = random_hyperspace_position(Vec2::new(
        crate::core::config::HALF_WIDTH,
        crate::core::config::HALF_HEIGHT,
    ));
    transform.translation.x = pos.x;
    transform.translation.y = pos.y;
    velocity.0 = Vec2::ZERO;
    cooldown.0 = Timer::from_seconds(HYPERSPACE_COOLDOWN_SECS, TimerMode::Once);
    sfx.write(SfxEvent(Sfx::Hyperspace));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::components::Velocity;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[test]
    fn player_spawns_with_special_charge() {
        let mut app = App::new();
        let assets = crate::fx::sprites::dummy_sprite_assets();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_player_entity(&mut commands, &assets);
            })
            .unwrap();
        let mut q = app.world_mut().query_filtered::<&SpecialWeapon, With<Player>>();
        let weapon = q.single(app.world()).unwrap();
        assert_eq!(weapon.charges, STARTING_SPECIAL_CHARGES);
        assert!(weapon.charges > 0, "게임 시작 시 특수무기를 최소 1개 보유해야 한다");
    }

    #[test]
    fn player_spawns_with_brief_invincibility() {
        let mut app = App::new();
        let assets = crate::fx::sprites::dummy_sprite_assets();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_player_entity(&mut commands, &assets);
            })
            .unwrap();
        // (재)스폰 시 일시 무적(Shield)을 받아 즉사 연쇄를 막는다.
        let mut q = app
            .world_mut()
            .query_filtered::<(), (With<Player>, With<Shield>)>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn thrust_accelerates_forward() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowUp);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity(Vec2::ZERO),
                EngineState::default(),
            ))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        // 회전 없음 → 정면은 +Y. 위로 가속.
        assert!(v.0.y > 0.0);
        assert!(v.0.x.abs() < 1e-3);
    }

    #[test]
    fn left_key_rotates_ship() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowLeft);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity(Vec2::ZERO),
                EngineState::default(),
            ))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let t = app.world().entity(e).get::<Transform>().unwrap();
        // 좌회전 → z축 회전각 > 0
        let (_, angle) = t.rotation.to_axis_angle();
        assert!(angle > 0.0);
    }

    #[test]
    fn ship_damping_reduces_speed_but_keeps_direction() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((Player, Velocity(Vec2::new(200.0, 0.0))))
            .id();
        app.world_mut().run_system_once(apply_ship_damping).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        // 감속하되(속도 감소) 방향은 유지(+x)
        assert!(v.0.x < 200.0);
        assert!(v.0.x > 0.0);
    }

    #[test]
    fn ship_speed_is_capped_at_max() {
        let mut app = App::new();
        let e = app
            .world_mut()
            .spawn((Player, Velocity(Vec2::new(10_000.0, 0.0))))
            .id();
        app.world_mut().run_system_once(apply_ship_damping).unwrap();
        let v = app.world().entity(e).get::<Velocity>().unwrap();
        assert!(v.0.length() <= SHIP_MAX_SPEED + 1e-3);
    }

    #[test]
    fn hyperspace_position_within_bounds() {
        let half = Vec2::new(640.0, 360.0);
        for _ in 0..20 {
            let p = random_hyperspace_position(half);
            assert!(p.x.abs() <= half.x && p.y.abs() <= half.y);
        }
    }

    #[test]
    fn thrust_key_sets_engine_state() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(0.1));
        app.insert_resource(time);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowUp);
        app.insert_resource(keys);
        let e = app
            .world_mut()
            .spawn((Player, Transform::default(), Velocity(Vec2::ZERO), EngineState::default()))
            .id();
        app.world_mut().run_system_once(player_input).unwrap();
        let s = app.world().entity(e).get::<EngineState>().unwrap();
        assert!(s.thrusting);
        assert!(!s.braking);
    }
}
