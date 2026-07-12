use bevy::prelude::*;

use crate::core::config::{EXPLOSION_FRAME_SECS, Z_EXPLOSION};
use crate::fx::sprites::SpriteAssets;

/// 프레임을 순서대로 교체하는 스프라이트 애니메이션(개별 프레임 이미지 스왑).
#[derive(Component)]
pub struct FrameAnimation {
    pub frames: Vec<Handle<Image>>,
    pub timer: Timer,
    pub index: usize,
}

/// 다음 프레임 인덱스. last를 넘어서면 None(=despawn 신호).
pub fn next_frame_index(current: usize, last: usize) -> Option<usize> {
    if current >= last {
        None
    } else {
        Some(current + 1)
    }
}

/// 지정 위치에 폭발 프레임 애니메이션 엔티티를 스폰한다.
pub fn spawn_explosion_anim(commands: &mut Commands, assets: &SpriteAssets, position: Vec2) {
    let frames = assets.explosion_frames.clone();
    let first = frames[0].clone();
    commands.spawn((
        FrameAnimation {
            frames,
            timer: Timer::from_seconds(EXPLOSION_FRAME_SECS, TimerMode::Repeating),
            index: 0,
        },
        Sprite {
            image: first,
            custom_size: Some(Vec2::splat(64.0)),
            ..default()
        },
        Transform::from_translation(position.extend(Z_EXPLOSION)),
        crate::core::state::GameplayEntity,
    ));
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            advance_animation.run_if(in_state(crate::core::state::GameState::Playing)),
        );
    }
}

/// 프레임 타이머가 끝날 때마다 다음 프레임으로 넘기고, 마지막 프레임 뒤엔 엔티티를 제거한다.
fn advance_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut FrameAnimation, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut q {
        anim.timer.tick(time.delta());
        if !anim.timer.is_finished() {
            continue;
        }
        // 프레임이 비어 있으면(불변식 위반) 조용히 제거해 언더플로 패닉을 방지.
        let Some(last) = anim.frames.len().checked_sub(1) else {
            commands.entity(entity).despawn();
            continue;
        };
        match next_frame_index(anim.index, last) {
            Some(next) => {
                anim.index = next;
                sprite.image = anim.frames[next].clone();
            }
            None => {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_advances_then_ends() {
        assert_eq!(next_frame_index(0, 5), Some(1));
        assert_eq!(next_frame_index(4, 5), Some(5));
        assert_eq!(next_frame_index(5, 5), None); // 마지막 → despawn
    }
}
