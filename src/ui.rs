//! UI 루트: 관심사별 서브모듈(hud·boss_bar·banner·game_over)의 시스템을
//! `UiPlugin` 하나로 묶어 등록한다.

mod banner;
mod boss_bar;
mod game_over;
mod hud;
pub(crate) mod menu;
mod title;

use bevy::prelude::*;

use crate::core::state::{GameState, RunPhase};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<banner::LastStage>()
            .init_resource::<menu::MenuSelection>()
            .add_systems(OnEnter(GameState::Title), title::spawn_title)
            .add_systems(OnExit(GameState::Title), title::despawn_title)
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
                    hud::update_weapon_slots,
                    banner::announce_stage,
                    banner::announce_boss,
                    banner::wave_banner_lifetime,
                    boss_bar::update_boss_bar,
                )
                    .run_if(in_state(RunPhase::Running)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over::spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), game_over::despawn_game_over)
            .add_systems(Update, game_over::restart_input.run_if(in_state(GameState::GameOver)))
            .add_systems(
                Update,
                (menu::menu_move, menu::menu_activate, menu::highlight_menu).run_if(
                    in_state(GameState::Title)
                        .or_else(in_state(GameState::GameOver))
                        .or_else(in_state(RunPhase::Paused)),
                ),
            );
    }
}
