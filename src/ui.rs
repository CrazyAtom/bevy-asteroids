//! UI 루트: 관심사별 서브모듈(hud·boss_bar·banner·game_over)의 시스템을
//! `UiPlugin` 하나로 묶어 등록한다.

mod banner;
mod boss_bar;
mod game_over;
mod help;
mod hud;
pub(crate) mod menu;
mod pause;
mod scaling;
mod title;

use bevy::prelude::*;

use crate::core::state::{GameState, RunPhase};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<banner::LastStage>()
            .init_resource::<menu::MenuSelection>()
            .init_resource::<help::HelpOpen>()
            // 창 크기에 맞춰 UI(Px)를 비례 스케일하고, 카메라를 16:9 레터박스로 맞춰
            // 월드도 정확히 1280×720만 보이게 한다(둘이 같은 fit 배율이라 정렬됨).
            .add_systems(Update, (scaling::sync_ui_scale, scaling::sync_letterbox))
            // HELP 오버레이: HelpOpen에 맞춰 패널 스폰/디스폰 + 열려 있으면 아무 키로 닫기.
            .add_systems(Update, help::manage_help)
            .add_systems(Update, help::close_help.run_if(help::help_is_open))
            .add_systems(OnEnter(GameState::Title), title::spawn_title)
            .add_systems(OnExit(GameState::Title), title::despawn_title)
            .add_systems(
                Update,
                (
                    title::advance_title_intro,
                    title::drift_title_debris,
                    title::glow_title,
                    title::rotate_title_ship,
                    title::emit_title_trail,
                    title::fade_title_trail,
                    title::hide_title_while_help,
                )
                    .run_if(in_state(GameState::Title)),
            )
            .add_systems(
                Update,
                title::pulse_title_selection
                    .after(menu::highlight_menu)
                    .run_if(in_state(GameState::Title)),
            )
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
            .add_systems(Update, pause::pause_input.run_if(in_state(RunPhase::Running)))
            .add_systems(
                Update,
                pause::resume_on_esc
                    .run_if(in_state(RunPhase::Paused).and_then(not(help::help_is_open))),
            )
            .add_systems(OnEnter(RunPhase::Paused), pause::spawn_pause_menu)
            .add_systems(OnExit(RunPhase::Paused), pause::despawn_pause_menu)
            .add_systems(
                Update,
                // HELP 오버레이가 열려 있으면 하부 메뉴 입력을 억제(그동안 close_help가
                // 아무 키를 닫기로 소비). 여는 Enter 프레임엔 아직 help가 꺼져 있어
                // menu_activate가 정상 실행되어 HelpOpen을 켠다.
                (menu::menu_move, menu::menu_activate, menu::highlight_menu).run_if(
                    in_state(GameState::Title)
                        .or_else(in_state(GameState::GameOver))
                        .or_else(in_state(RunPhase::Paused))
                        .and_then(not(help::help_is_open)),
                ),
            );
    }
}
