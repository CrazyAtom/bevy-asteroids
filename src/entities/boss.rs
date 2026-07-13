use bevy::prelude::*;

use crate::core::config::{BOSS_BASE_HEALTH, BOSS_HEALTH_PER_CYCLE};
use crate::systems::stage::ThemeId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    MotherRock,
    Mothership,
    BlazingCore,
}

pub fn boss_for_theme(t: ThemeId) -> BossKind {
    match t {
        ThemeId::AsteroidBelt => BossKind::MotherRock,
        ThemeId::AlienFleet => BossKind::Mothership,
        ThemeId::SolarFlare => BossKind::BlazingCore,
    }
}

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
    pub health: f32,
    pub max_health: f32,
}

/// 보스 종류별 기준 체력(모암이 가장 높음) × 사이클 증가.
pub fn boss_max_health(kind: BossKind, cycle: u32) -> f32 {
    let base = match kind {
        BossKind::MotherRock => BOSS_BASE_HEALTH * 1.5,
        BossKind::Mothership => BOSS_BASE_HEALTH,
        BossKind::BlazingCore => BOSS_BASE_HEALTH * 1.2,
    };
    base + cycle as f32 * BOSS_HEALTH_PER_CYCLE
}

/// 체력을 깎고, 0 이하이면 true(격파).
pub fn apply_boss_damage(boss: &mut Boss, dmg: f32) -> bool {
    boss.health -= dmg;
    boss.health <= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::stage::ThemeId;

    #[test]
    fn boss_kind_matches_theme() {
        assert_eq!(boss_for_theme(ThemeId::AsteroidBelt), BossKind::MotherRock);
        assert_eq!(boss_for_theme(ThemeId::AlienFleet), BossKind::Mothership);
        assert_eq!(boss_for_theme(ThemeId::SolarFlare), BossKind::BlazingCore);
    }

    #[test]
    fn health_scales_with_cycle() {
        let h0 = boss_max_health(BossKind::MotherRock, 0);
        let h2 = boss_max_health(BossKind::MotherRock, 2);
        assert!(h2 > h0);
    }

    #[test]
    fn damage_reduces_and_defeats() {
        let mut b = Boss { kind: BossKind::Mothership, health: 3.0, max_health: 10.0 };
        assert!(!apply_boss_damage(&mut b, 1.0)); // 2 남음
        assert!((b.health - 2.0).abs() < 1e-6);
        assert!(apply_boss_damage(&mut b, 5.0)); // 0 이하 → 격파
    }
}
