//! 보스 체력 바(화면 상단, 보스 존재 시에만 표시).

use bevy::prelude::*;

use crate::core::state::GameplayEntity;
use crate::entities::boss::Boss;
use crate::ui::scaling::spawn_stage;

#[derive(Component)]
pub(super) struct BossHealthBar;

#[derive(Component)]
pub(super) struct BossHealthFill;

pub(super) fn spawn_boss_bar(mut commands: Commands) {
    // 보스 체력 바(기본 숨김; 보스 존재 시 표시). 1280×720 스테이지 위에 올려
    // Percent 위치·폭이 ui_scale로 다른 UI와 같은 비율로 스케일되게 한다.
    let stage = spawn_stage(&mut commands, (GameplayEntity,));
    commands.entity(stage).with_children(|s| {
        s.spawn((
            BossHealthBar,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(46.0),
                left: Val::Percent(25.0),
                width: Val::Percent(50.0),
                height: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.8)),
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((
                BossHealthFill,
                Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(Color::srgb(1.0, 0.28, 0.28)),
            ));
        });
    });
}

/// 보스가 있으면 체력 바를 표시하고 폭을 체력 비율로 갱신, 없으면 숨긴다.
pub(super) fn update_boss_bar(
    bosses: Query<&Boss>,
    mut bar: Query<&mut Visibility, With<BossHealthBar>>,
    mut fill: Query<&mut Node, With<BossHealthFill>>,
) {
    let boss = bosses.iter().next();
    for mut vis in &mut bar {
        *vis = if boss.is_some() { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Some(boss) = boss {
        let frac = (boss.health / boss.max_health).clamp(0.0, 1.0);
        for mut node in &mut fill {
            node.width = Val::Percent(frac * 100.0);
        }
    }
}
