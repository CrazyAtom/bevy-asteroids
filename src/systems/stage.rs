use bevy::prelude::*;

use crate::core::config::{CYCLE_LEN, STARTING_LIVES, WAVES_PER_STAGE};
use crate::core::logic::{asteroid_count_for_wave, asteroid_speed_scale_for_wave, AsteroidSize};
use crate::core::state::{GameState, Lives};
use crate::entities::asteroid::{random_spawn_position, random_velocity, spawn_asteroid, Asteroid};
use crate::fx::sprites::SpriteAssets;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeId {
    AsteroidBelt,
    AlienFleet,
    SolarFlare,
}

pub const THEME_POOL: [ThemeId; 3] = [ThemeId::AsteroidBelt, ThemeId::AlienFleet, ThemeId::SolarFlare];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StagePhase {
    Waves,
    Boss,
}

#[derive(Resource)]
pub struct Progression {
    pub cycle: u32,
    pub stage_in_cycle: usize,
    pub order: Vec<ThemeId>,
    pub phase: StagePhase,
    pub wave_in_stage: u32,
}

impl Progression {
    pub fn current_theme(&self) -> ThemeId {
        self.order[self.stage_in_cycle]
    }
}

pub fn new_progression() -> Progression {
    debug_assert!(
        THEME_POOL.len() >= CYCLE_LEN,
        "CYCLE_LEN must not exceed THEME_POOL size (current_theme 인덱싱 안전 보장)"
    );
    Progression {
        cycle: 0,
        stage_in_cycle: 0,
        order: pick_cycle_order(&THEME_POOL, CYCLE_LEN),
        phase: StagePhase::Waves,
        wave_in_stage: 0,
    }
}

/// pool에서 중복 없이 count개를 무작위로 뽑는다(부분 Fisher-Yates).
pub fn pick_cycle_order(pool: &[ThemeId], count: usize) -> Vec<ThemeId> {
    use rand::RngExt;
    let mut rng = rand::rng();
    let mut v = pool.to_vec();
    let n = v.len();
    for i in 0..n.min(count) {
        let j = rng.random_range(i..n);
        v.swap(i, j);
    }
    v.truncate(count);
    v
}

/// 보스 격파 후 다음 스테이지로. 사이클 끝이면 재셔플 + cycle↑.
pub fn advance_stage(prog: &mut Progression) {
    prog.stage_in_cycle += 1;
    if prog.stage_in_cycle >= CYCLE_LEN {
        prog.cycle += 1;
        prog.stage_in_cycle = 0;
        prog.order = pick_cycle_order(&THEME_POOL, CYCLE_LEN);
    }
    prog.phase = StagePhase::Waves;
    prog.wave_in_stage = 0;
}

/// 기존 난이도 공식(asteroid_count_for_wave 등)에 넘길 '웨이브 번호'.
/// 사이클/스테이지/웨이브가 누적될수록 단조 증가.
pub fn stage_wave_number(prog: &Progression) -> u32 {
    let stages_done = prog.cycle * CYCLE_LEN as u32 + prog.stage_in_cycle as u32;
    stages_done * WAVES_PER_STAGE + prog.wave_in_stage + 1
}

pub struct ThemeParams {
    pub count_mul: f32,
    pub speed_mul: f32,
    pub ufo_interval_mul: f32,
}

pub fn theme_params(t: ThemeId) -> ThemeParams {
    match t {
        ThemeId::AsteroidBelt => ThemeParams { count_mul: 1.15, speed_mul: 1.0, ufo_interval_mul: 1.0 },
        ThemeId::AlienFleet => ThemeParams { count_mul: 1.0, speed_mul: 1.0, ufo_interval_mul: 0.8 },
        ThemeId::SolarFlare => ThemeParams { count_mul: 1.0, speed_mul: 1.2, ufo_interval_mul: 1.0 },
    }
}

pub fn themed_wave_count(theme: ThemeId, wave_no: u32) -> usize {
    let base = asteroid_count_for_wave(wave_no) as f32;
    (base * theme_params(theme).count_mul).round().max(1.0) as usize
}

pub fn themed_speed_scale(theme: ThemeId, wave_no: u32) -> f32 {
    asteroid_speed_scale_for_wave(wave_no) * theme_params(theme).speed_mul
}

pub fn theme_name(t: ThemeId) -> &'static str {
    match t {
        ThemeId::AsteroidBelt => "소행성대",
        ThemeId::AlienFleet => "외계 함대",
        ThemeId::SolarFlare => "화염지대",
    }
}

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            start_first_stage.after(crate::core::state::reset_game),
        )
        .add_systems(Update, stage_control.run_if(in_state(GameState::Playing)));
    }
}

/// 현재 스테이지 테마/난이도로 소행성 한 웨이브를 스폰한다.
pub fn spawn_first_wave_of_stage(commands: &mut Commands, assets: &SpriteAssets, prog: &Progression) {
    let theme = prog.current_theme();
    let wave_no = stage_wave_number(prog);
    let count = themed_wave_count(theme, wave_no);
    let scale = themed_speed_scale(theme, wave_no);
    for _ in 0..count {
        let base = random_velocity(AsteroidSize::Large);
        spawn_asteroid(commands, assets, AsteroidSize::Large, random_spawn_position(), base * scale);
    }
}

fn start_first_stage(mut commands: Commands, assets: Res<SpriteAssets>, prog: Res<Progression>) {
    spawn_first_wave_of_stage(&mut commands, &assets, &prog);
}

/// Waves 단계: 소행성 전멸 시 다음 웨이브 스폰 또는 보스 전환.
/// (Task 3 시점: 보스 대신 임시로 즉시 다음 스테이지. Task 4에서 보스 스폰으로 교체.)
fn stage_control(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut prog: ResMut<Progression>,
    mut lives: ResMut<Lives>,
    asteroids: Query<(), With<Asteroid>>,
) {
    if prog.phase != StagePhase::Waves {
        return;
    }
    if asteroids.iter().count() != 0 {
        return;
    }
    prog.wave_in_stage += 1;
    if prog.wave_in_stage < WAVES_PER_STAGE {
        // 다음 웨이브 진입 시 목숨을 기본치 이상으로 리필(기본 이상이면 유지).
        lives.0 = lives.0.max(STARTING_LIVES);
        spawn_first_wave_of_stage(&mut commands, &assets, &prog);
    } else {
        // 웨이브 전멸 상태에서 진입하므로 잔여 소행성은 없다. 보스 스폰.
        prog.phase = StagePhase::Boss;
        let kind = crate::entities::boss::boss_for_theme(prog.current_theme());
        crate::entities::boss::spawn_boss(&mut commands, &assets, kind, prog.cycle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themed_scaling_applies_multiplier() {
        let belt = themed_wave_count(ThemeId::AsteroidBelt, 1);
        let fleet = themed_wave_count(ThemeId::AlienFleet, 1);
        assert!(belt >= fleet); // belt count_mul 1.3
        assert!(themed_speed_scale(ThemeId::SolarFlare, 1) > themed_speed_scale(ThemeId::AlienFleet, 1));
    }

    #[test]
    fn cycle_order_is_distinct_subset() {
        for _ in 0..20 {
            let o = pick_cycle_order(&THEME_POOL, CYCLE_LEN);
            assert_eq!(o.len(), CYCLE_LEN);
            for (i, a) in o.iter().enumerate() {
                assert!(THEME_POOL.contains(a));
                assert!(!o[..i].contains(a), "중복 없음");
            }
        }
    }

    #[test]
    fn advance_within_cycle_then_new_cycle() {
        let mut p = new_progression();
        p.phase = StagePhase::Boss;
        p.wave_in_stage = WAVES_PER_STAGE;
        let c0 = p.cycle;
        advance_stage(&mut p); // 0 -> 1
        assert_eq!(p.stage_in_cycle, 1);
        assert_eq!(p.cycle, c0);
        assert_eq!(p.phase, StagePhase::Waves);
        assert_eq!(p.wave_in_stage, 0);
        advance_stage(&mut p); // 1 -> 2
        advance_stage(&mut p); // 2 -> 새 사이클
        assert_eq!(p.stage_in_cycle, 0);
        assert_eq!(p.cycle, c0 + 1);
        assert_eq!(p.order.len(), CYCLE_LEN);
    }

    #[test]
    fn wave_number_increases_with_progress() {
        let mut p = new_progression();
        let a = stage_wave_number(&p);
        p.wave_in_stage = 2;
        let b = stage_wave_number(&p);
        p.stage_in_cycle = 1;
        let c = stage_wave_number(&p);
        p.cycle = 1;
        let d = stage_wave_number(&p);
        assert!(a < b && b < c && c < d);
    }

    #[test]
    fn theme_params_differ() {
        assert!(theme_params(ThemeId::AsteroidBelt).count_mul > 1.0);
        assert!(theme_params(ThemeId::SolarFlare).speed_mul > 1.0);
        assert!(theme_params(ThemeId::AlienFleet).ufo_interval_mul < 1.0);
    }
}
