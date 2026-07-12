use bevy::prelude::*;

use crate::core::components::{Collider, Velocity, Wrapping};
use crate::core::config::{
    BEAM_LENGTH, BEAM_LIFETIME_SECS, FLAME_COLOR, HYPERSPACE_COOLDOWN_SECS, SHAKE_SPECIAL,
    SHIP_BRAKE_RATE, SHIP_COLLIDER_RADIUS, SHIP_COLOR, SHIP_DAMPING, SHIP_MAX_SPEED,
    SHIP_ROTATION_SPEED, SHIP_THRUST, STARTING_SPECIAL_CHARGES,
};
use crate::core::logic::apply_brake;
use crate::core::state::{GameState, GameplayEntity};
use crate::entities::powerup::SpecialWeaponKind;
use crate::fx::audio::{Sfx, SfxEvent};
use crate::fx::shake::ShakeEvent;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct EngineState {
    pub thrusting: bool,
    pub braking: bool,
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
}

/// 우주선 로컬 좌표(정면 = +Y). 마지막 점은 첫 점과 같아 닫힌 외곽선을 만든다.
const SHIP_POINTS: [Vec2; 5] = [
    Vec2::new(0.0, 16.0),
    Vec2::new(-11.0, -12.0),
    Vec2::new(0.0, -6.0),
    Vec2::new(11.0, -12.0),
    Vec2::new(0.0, 16.0),
];

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_input,
                    draw_player,
                    shield_tick,
                    draw_shield,
                    tick_fire_mods,
                    activate_special,
                    tick_and_draw_beam,
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

pub fn spawn_player_entity(commands: &mut Commands) {
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
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
    ));
}

fn spawn_player(mut commands: Commands) {
    spawn_player_entity(&mut commands);
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
        transform.rotate_z(turn * SHIP_ROTATION_SPEED * dt);

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

fn draw_player(
    mut gizmos: Gizmos,
    time: Res<Time>,
    query: Query<(&Transform, &EngineState), With<Player>>,
) {
    for (transform, engine) in &query {
        let points = SHIP_POINTS.map(|p| transform.transform_point(p.extend(0.0)).truncate());
        gizmos.linestrip_2d(points, SHIP_COLOR);

        // 깜빡임 계수(0.6~1.0)
        let flicker = 0.6 + 0.4 * (time.elapsed_secs() * 30.0).sin().abs();
        if engine.thrusting {
            let flame = [
                Vec2::new(-6.0, -12.0),
                Vec2::new(0.0, -12.0 - 10.0 * flicker),
                Vec2::new(6.0, -12.0),
            ]
            .map(|p| transform.transform_point(p.extend(0.0)).truncate());
            gizmos.linestrip_2d(flame, FLAME_COLOR);
        }
        if engine.braking {
            let flame = [
                Vec2::new(-4.0, 14.0),
                Vec2::new(0.0, 14.0 + 7.0 * flicker),
                Vec2::new(4.0, 14.0),
            ]
            .map(|p| transform.transform_point(p.extend(0.0)).truncate());
            gizmos.linestrip_2d(flame, FLAME_COLOR);
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

fn draw_shield(mut gizmos: Gizmos, q: Query<&Transform, (With<Player>, With<Shield>)>) {
    for transform in &q {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            18.0,
            Color::srgb(0.3, 0.7, 1.0),
        );
    }
}

fn activate_special(
    mut commands: Commands,
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
            commands.spawn((
                SpecialBeam { life: Timer::from_seconds(BEAM_LIFETIME_SECS, TimerMode::Once), origin, dir },
                GameplayEntity,
            ));
        }
    }
    sfx.write(SfxEvent(Sfx::Special));
    shake.write(ShakeEvent(SHAKE_SPECIAL));
}

fn tick_and_draw_beam(
    mut commands: Commands,
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut query: Query<(Entity, &mut SpecialBeam)>,
) {
    for (entity, mut beam) in &mut query {
        beam.life.tick(time.delta());
        if beam.life.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let end = beam.origin + beam.dir.normalize_or_zero() * BEAM_LENGTH;
        // 굵게 보이도록 평행선 여러 개
        for off in [-8.0, -4.0, 0.0, 4.0, 8.0] {
            let perp = Vec2::new(-beam.dir.y, beam.dir.x).normalize_or_zero() * off;
            gizmos.line_2d(beam.origin + perp, end + perp, Color::srgb(1.0, 0.3, 1.0));
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
    fn player_starts_with_special_charge() {
        let mut app = App::new();
        app.world_mut().run_system_once(spawn_player).unwrap();
        let mut q = app
            .world_mut()
            .query_filtered::<&SpecialWeapon, With<Player>>();
        let weapon = q.single(app.world()).unwrap();
        assert_eq!(weapon.charges, STARTING_SPECIAL_CHARGES);
        assert!(weapon.charges > 0, "게임 시작 시 특수무기를 최소 1개 보유해야 한다");
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
