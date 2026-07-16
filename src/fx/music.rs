//! 배경 음악(BGM): 상태·테마에 맞춰 절차 생성 루프 트랙을 재생·교체한다.
//! 트랙 WAV는 build.rs가 `assets/music/*.wav`로 생성한다(다성 칩튠).

use bevy::prelude::*;

use crate::core::state::GameState;
use crate::systems::stage::{Progression, ThemeId};

/// 재생 가능한 BGM 트랙. 타이틀 1곡 + 테마 6곡.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Title,
    Belt,
    Fleet,
    Flare,
    Ice,
    Storm,
    Void,
}

impl Track {
    fn file(self) -> &'static str {
        match self {
            Track::Title => "music/title.wav",
            Track::Belt => "music/belt.wav",
            Track::Fleet => "music/fleet.wav",
            Track::Flare => "music/flare.wav",
            Track::Ice => "music/ice.wav",
            Track::Storm => "music/storm.wav",
            Track::Void => "music/void.wav",
        }
    }

    /// 진행 테마 → 게임플레이 트랙 매핑.
    fn for_theme(theme: ThemeId) -> Track {
        match theme {
            ThemeId::AsteroidBelt => Track::Belt,
            ThemeId::AlienFleet => Track::Fleet,
            ThemeId::SolarFlare => Track::Flare,
            ThemeId::FrozenField => Track::Ice,
            ThemeId::EmStorm => Track::Storm,
            ThemeId::BlackHole => Track::Void,
        }
    }
}

/// 현재 상태·테마에서 재생해야 할 트랙(순수 함수). Title→Title, 그 외→현재 테마곡.
/// 게임오버·재시작에서도 직전 테마곡을 유지하다 타이틀 복귀 시 타이틀곡으로 전환.
/// Progression은 게임플레이 중에만 존재하므로 없으면(타이틀 등) 안전하게 Title.
fn desired_track(state: GameState, prog: Option<&Progression>) -> Track {
    match state {
        GameState::Title => Track::Title,
        _ => prog.map(|p| Track::for_theme(p.current_theme())).unwrap_or(Track::Title),
    }
}

/// 현재 재생 중인 트랙과 오디오 엔티티.
#[derive(Resource, Default)]
struct CurrentMusic {
    track: Option<Track>,
    entity: Option<Entity>,
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentMusic>().add_systems(Update, sync_music);
    }
}

/// 원하는 트랙이 바뀌면 하드 컷으로 교체한다(기존 despawn → LOOP 재생 스폰).
/// 값이 같으면 매 프레임 아무것도 안 한다.
fn sync_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
    prog: Option<Res<Progression>>,
    mut current: ResMut<CurrentMusic>,
) {
    let desired = desired_track(*state.get(), prog.as_deref());
    if current.track == Some(desired) {
        return;
    }
    if let Some(entity) = current.entity.take() {
        commands.entity(entity).despawn();
    }
    let entity = commands
        .spawn((AudioPlayer::new(asset_server.load(desired.file())), PlaybackSettings::LOOP))
        .id();
    current.track = Some(desired);
    current.entity = Some(entity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::stage::new_progression;

    #[test]
    fn track_maps_each_theme() {
        assert_eq!(Track::for_theme(ThemeId::AsteroidBelt), Track::Belt);
        assert_eq!(Track::for_theme(ThemeId::AlienFleet), Track::Fleet);
        assert_eq!(Track::for_theme(ThemeId::SolarFlare), Track::Flare);
        assert_eq!(Track::for_theme(ThemeId::FrozenField), Track::Ice);
        assert_eq!(Track::for_theme(ThemeId::EmStorm), Track::Storm);
        assert_eq!(Track::for_theme(ThemeId::BlackHole), Track::Void);
    }

    #[test]
    fn title_state_always_plays_title() {
        let prog = new_progression();
        assert_eq!(desired_track(GameState::Title, Some(&prog)), Track::Title);
        assert_eq!(desired_track(GameState::Title, None), Track::Title);
    }

    #[test]
    fn gameplay_states_play_current_theme_track() {
        let prog = new_progression(); // 첫 테마 = AsteroidBelt
        let want = Track::for_theme(prog.current_theme());
        for state in [GameState::Playing, GameState::GameOver, GameState::Restarting] {
            assert_eq!(desired_track(state, Some(&prog)), want);
        }
    }

    #[test]
    fn missing_progression_falls_back_to_title() {
        // 게임플레이 상태인데 Progression이 아직 없으면(비정상) 안전하게 Title.
        assert_eq!(desired_track(GameState::Playing, None), Track::Title);
    }
}
