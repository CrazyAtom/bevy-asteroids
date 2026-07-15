//! UI 루트: 관심사별 서브모듈(hud·boss_bar·banner·game_over)의 시스템을
//! `UiPlugin` 하나로 묶어 등록한다.

mod banner;
mod boss_bar;
mod game_over;
mod hud;

use bevy::prelude::*;

use crate::core::state::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<banner::LastStage>()
            .add_systems(
                OnEnter(GameState::Playing),
                (hud::spawn_hud, boss_bar::spawn_boss_bar, banner::reset_last_stage),
            )
            .add_systems(
                Update,
                (
                    hud::update_hud,
                    hud::update_life_icons,
                    hud::update_mod_icons,
                    banner::announce_stage,
                    banner::announce_boss,
                    banner::wave_banner_lifetime,
                    boss_bar::update_boss_bar,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over::spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), game_over::despawn_game_over)
            .add_systems(Update, game_over::restart_input.run_if(in_state(GameState::GameOver)));
    }
}
