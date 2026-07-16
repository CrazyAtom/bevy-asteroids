//! 시야 제한 안개(전자기폭풍 테마). 방사형 그라데이션 오버레이가 우주선을 따라다니며
//! 원형 시야창을 만든다. on/off는 `FogState`가 제어하고(테마에 무관), 진행 테마에 따라
//! `systems::stage::sync_fog_state`가 켠다. 시야창 크기는 폭풍 피크에 맞춰 진동한다.

use bevy::prelude::*;

use crate::core::config::{FOG_BASE_SIZE, Z_FOG};
use crate::core::logic::storm_vision_scale;
use crate::core::state::RunPhase;
use crate::entities::player::Player;
use crate::fx::sprites::SpriteAssets;

/// 안개 활성 여부. 기본 off(안개 없음). 진행 테마가 EM Storm일 때만 켜진다.
#[derive(Resource, Default)]
pub struct FogState {
    pub active: bool,
}

#[derive(Component)]
struct FogOverlay;

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FogState>()
            .add_systems(Startup, spawn_fog)
            .add_systems(Update, update_fog.run_if(not(in_state(RunPhase::Paused))));
    }
}

fn spawn_fog(mut commands: Commands, assets: Res<SpriteAssets>) {
    commands.spawn((
        FogOverlay,
        Sprite {
            image: assets.fog.clone(),
            custom_size: Some(Vec2::splat(FOG_BASE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_FOG),
        Visibility::Hidden,
    ));
}

/// 활성 시 오버레이를 우주선에 맞추고 시야창 크기를 폭풍 피크로 진동시킨다. 비활성 시 숨김.
fn update_fog(
    time: Res<Time>,
    fog_state: Res<FogState>,
    players: Query<&Transform, (With<Player>, Without<FogOverlay>)>,
    mut q: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<FogOverlay>>,
) {
    let Ok((mut tf, mut sprite, mut vis)) = q.single_mut() else { return };
    if !fog_state.active {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Visible;
    if let Ok(ship) = players.single() {
        tf.translation.x = ship.translation.x;
        tf.translation.y = ship.translation.y;
    }
    let scale = storm_vision_scale(time.elapsed_secs());
    sprite.custom_size = Some(Vec2::splat(FOG_BASE_SIZE * scale));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_state_defaults_off() {
        assert!(!FogState::default().active);
    }
}
