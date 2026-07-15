//! 상단 HUD: 점수/최고점/스테이지 텍스트, 목숨 하트, 특수무기 충전, 활성 파워업 배지.

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_persistent::prelude::*;

use crate::core::state::{GameplayEntity, HighScore, Lives, Score};
use crate::entities::player::{Player, RapidFire, Shield, Spread};
use crate::entities::special_weapon::SpecialWeapon;
use crate::fx::sprites::SpriteAssets;
use crate::systems::stage::Progression;

#[derive(Component)]
pub(super) struct Hud;

/// 목숨 하트 아이콘(인덱스). index < lives 이면 표시.
#[derive(Component)]
pub(super) struct LifeIcon(usize);

/// 특수무기 충전 수 텍스트.
#[derive(Component)]
pub(super) struct SpChargeText;

/// 활성 파워업 배지 아이콘(0=실드,1=연사,2=확산). 활성 시만 표시.
#[derive(Component)]
pub(super) struct ModIcon(u8);

const MAX_HEART_ICONS: usize = 8;
const HUD_ICON_PX: f32 = 22.0;

pub(super) fn spawn_hud(mut commands: Commands, assets: Res<SpriteAssets>) {
    let icon = || Node {
        width: Val::Px(HUD_ICON_PX),
        height: Val::Px(HUD_ICON_PX),
        ..default()
    };
    let row = || Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(3.0),
        ..default()
    };
    commands
        .spawn((
            GameplayEntity,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                ..default()
            },
        ))
        .with_children(|root| {
            // Score / High / Stage 텍스트
            root.spawn((
                Hud,
                Text::new("Score: 0   High: 0   Stage 1-1"),
                TextFont { font_size: FontSize::Px(22.0), ..default() },
                TextColor(Color::WHITE),
            ));
            // 목숨: 하트 나열
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(1.0),
                ..default()
            })
            .with_children(|hearts| {
                for i in 0..MAX_HEART_ICONS {
                    hearts.spawn((LifeIcon(i), ImageNode::new(assets.powerup[3].clone()), icon()));
                }
            });
            // 특수무기: 아이콘 + 충전 수
            root.spawn(row()).with_children(|sp| {
                sp.spawn((ImageNode::new(assets.powerup[4].clone()), icon()));
                sp.spawn((
                    SpChargeText,
                    Text::new("0"),
                    TextFont { font_size: FontSize::Px(22.0), ..default() },
                    TextColor(Color::WHITE),
                ));
            });
            // 활성 파워업 배지(실드/연사/확산; 활성 시만 표시)
            root.spawn(row()).with_children(|mods| {
                for i in 0..3u8 {
                    mods.spawn((
                        ModIcon(i),
                        ImageNode::new(assets.powerup[i as usize].clone()),
                        icon(),
                        Visibility::Hidden,
                    ));
                }
            });
        });
}

pub(super) fn update_hud(
    score: Res<Score>,
    high: Res<Persistent<HighScore>>,
    prog: Res<Progression>,
    player: Query<&SpecialWeapon, With<Player>>,
    mut hud: Query<&mut Text, (With<Hud>, Without<SpChargeText>)>,
    mut sp: Query<&mut Text, (With<SpChargeText>, Without<Hud>)>,
) {
    for mut text in &mut hud {
        text.0 = format!(
            "Score: {}   High: {}   Stage {}-{}",
            score.0, high.0, prog.cycle + 1, prog.stage_in_cycle + 1
        );
    }
    let charges = player.single().map(|w| w.charges).unwrap_or(0);
    for mut text in &mut sp {
        text.0 = format!("{charges}");
    }
}

/// 목숨 수만큼 하트 아이콘을 표시한다.
pub(super) fn update_life_icons(lives: Res<Lives>, mut q: Query<(&LifeIcon, &mut Visibility)>) {
    for (icon, mut vis) in &mut q {
        *vis = if (icon.0 as u32) < lives.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 활성 파워업 배지 아이콘을 표시/숨김한다(실드/연사/확산).
#[allow(clippy::type_complexity)]
pub(super) fn update_mod_icons(
    player: Query<(Has<Shield>, Has<RapidFire>, Has<Spread>), With<Player>>,
    mut icons: Query<(&ModIcon, &mut Visibility)>,
) {
    let (sh, rf, sp) = player.single().unwrap_or((false, false, false));
    for (icon, mut vis) in &mut icons {
        let on = match icon.0 {
            0 => sh,
            1 => rf,
            _ => sp,
        };
        *vis = if on { Visibility::Visible } else { Visibility::Hidden };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn hud_shows_score_lives_wave() {
        let mut app = App::new();
        app.insert_resource(Score(150));
        app.insert_resource(Lives(2));
        app.insert_resource(crate::systems::stage::new_progression());
        let hs = Persistent::<HighScore>::builder()
            .name("test high score")
            .format(StorageFormat::Json)
            .path(std::env::temp_dir().join("bevy-asteroids-test").join("highscore.json"))
            .default(HighScore(0))
            .build()
            .unwrap();
        app.insert_resource(hs);
        let e = app.world_mut().spawn((Hud, Text::new("초기값"))).id();
        app.world_mut().run_system_once(update_hud).unwrap();
        let text = app.world().entity(e).get::<Text>().unwrap();
        assert!(text.0.contains("150"));
        assert!(text.0.contains("Stage 1-1"));
        assert!(text.0.contains("High: 0"));
    }
}
