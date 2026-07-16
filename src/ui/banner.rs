//! 중앙 배너: 스테이지 시작("STAGE c-s 테마명")과 보스 등장("! BOSS !") 알림.

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::core::state::GameplayEntity;
use crate::entities::boss::Boss;
use crate::systems::stage::{theme_name, Progression};
use crate::ui::scaling::spawn_stage;

/// 웨이브가 오를 때 화면 중앙에 잠깐 떴다 사라지는 "WAVE N" 배너.
#[derive(Component)]
pub(super) struct WaveBanner {
    pub(super) life: Timer,
}

#[derive(Resource, Default)]
pub(super) struct LastStage(Option<(u32, usize)>);

/// 재시작 시 스테이지 배너 중복 억제 상태를 초기화한다(Local이 아니라 리소스라 리셋 가능).
pub(super) fn reset_last_stage(mut last: ResMut<LastStage>) {
    last.0 = None;
}

/// 스테이지(사이클·순번)가 바뀌면 "STAGE c-s 테마명" 배너를 잠깐 띄운다.
pub(super) fn announce_stage(
    mut commands: Commands,
    prog: Res<Progression>,
    mut last: ResMut<LastStage>,
    existing: Query<Entity, With<WaveBanner>>,
) {
    let key = (prog.cycle, prog.stage_in_cycle);
    if last.0 == Some(key) {
        return;
    }
    last.0 = Some(key);
    // 이전 배너가 남아 있으면 제거해 중첩 방지
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let stage = spawn_stage(
        &mut commands,
        (WaveBanner { life: Timer::from_seconds(1.8, TimerMode::Once) }, GameplayEntity),
    );
    commands.entity(stage).with_children(|s| {
        s.spawn((
            Text::new(format!(
                "STAGE {}-{}  {}",
                prog.cycle + 1,
                prog.stage_in_cycle + 1,
                theme_name(prog.current_theme())
            )),
            TextFont { font_size: FontSize::Px(44.0), ..default() },
            TextColor(Color::srgb(0.4, 0.9, 1.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(32.0),
                ..default()
            },
        ));
    });
}

/// 수명이 다한 웨이브 배너를 제거한다.
pub(super) fn wave_banner_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut WaveBanner)>,
) {
    for (entity, mut banner) in &mut query {
        banner.life.tick(time.delta());
        if banner.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// 보스가 등장한 프레임에 "⚠ BOSS" 배너를 잠깐 띄운다.
pub(super) fn announce_boss(
    mut commands: Commands,
    bosses: Query<(), Added<Boss>>,
    existing: Query<Entity, With<WaveBanner>>,
) {
    if bosses.is_empty() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let stage = spawn_stage(
        &mut commands,
        (WaveBanner { life: Timer::from_seconds(1.5, TimerMode::Once) }, GameplayEntity),
    );
    commands.entity(stage).with_children(|s| {
        s.spawn((
            Text::new("! BOSS !"),
            TextFont { font_size: FontSize::Px(52.0), ..default() },
            TextColor(Color::srgb(1.0, 0.35, 0.35)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(28.0),
                left: Val::Percent(40.0),
                ..default()
            },
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn wave_banner_expires() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        let mut timer = Timer::from_seconds(1.5, TimerMode::Once);
        timer.tick(std::time::Duration::from_secs_f32(2.0)); // 이미 만료
        let e = app.world_mut().spawn(WaveBanner { life: timer }).id();
        app.world_mut().run_system_once(wave_banner_lifetime).unwrap();
        assert!(app.world().get_entity(e).is_err());
    }
}
