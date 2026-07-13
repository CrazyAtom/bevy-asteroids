use bevy::prelude::*;

use crate::core::config::{CYCLE_LEN, WAVES_PER_STAGE};

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
        ThemeId::AsteroidBelt => ThemeParams { count_mul: 1.3, speed_mul: 1.0, ufo_interval_mul: 1.0 },
        ThemeId::AlienFleet => ThemeParams { count_mul: 1.0, speed_mul: 1.0, ufo_interval_mul: 0.6 },
        ThemeId::SolarFlare => ThemeParams { count_mul: 1.0, speed_mul: 1.4, ufo_interval_mul: 1.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
