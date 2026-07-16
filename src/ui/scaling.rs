//! 기준 해상도(1280×720)로 디자인된 UI를 현재 뷰포트에 비례 스케일한다.
//!
//! 카메라 투영(AutoMin)은 '월드'만 스케일하고 Bevy UI는 그걸 따르지 않는다 —
//! UI는 창의 논리 픽셀로 레이아웃된다. 웹에선 창 크기에 따라 논리 해상도가
//! 유동하므로, 고정 `Val::Px`(폰트·HUD)로 짜인 UI는 창을 줄이면 그대로인 채
//! 뷰포트만 작아져 화면 밖으로 넘친다. `UiScale`(Px 값 전체에 곱해지는 배율)을
//! 창 비율만큼 맞춰, UI를 월드의 AutoMin 스케일링과 일치시킨다.

use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::core::config::{WINDOW_HEIGHT, WINDOW_WIDTH};

/// 논리 뷰포트 크기에 대한 UI 스케일 배율. 기준 1280×720 박스가 뷰포트 안에
/// 들어가도록(contain) 두 축 비율 중 더 작은 값을 쓴다 — 월드 카메라의
/// `ScalingMode::AutoMin`과 동일한 fit이라 UI와 월드가 같은 비율로 스케일된다.
pub fn ui_scale_for(width: f32, height: f32) -> f32 {
    (width / WINDOW_WIDTH).min(height / WINDOW_HEIGHT)
}

/// 주 창의 논리 크기를 읽어 `UiScale`을 갱신한다. 값이 실제로 바뀔 때만 써서
/// 불필요한 변경 감지(레이아웃 재계산)를 피한다. 네이티브 기본 창(1280×720)에선
/// 배율이 1.0이라 사실상 no-op이고, 창을 줄여도 UI가 잘리지 않게 지켜준다.
pub fn sync_ui_scale(windows: Query<&Window, With<PrimaryWindow>>, mut ui_scale: ResMut<UiScale>) {
    let Ok(window) = windows.single() else { return };
    let target = ui_scale_for(window.width(), window.height());
    // 창이 0/최소화 등 비정상 크기면 배율이 0·비유한이 되어 레이아웃을 깨므로 무시.
    if target.is_finite() && target > 0.0 && (ui_scale.0 - target).abs() > f32::EPSILON {
        ui_scale.0 = target;
    }
}

/// 화면 전체를 덮는 투명 컨테이너 + 중앙 정렬된 1280×720 "가상 스테이지"를
/// 스폰하고, 스테이지(자식) 엔티티를 반환한다. 반환 엔티티의 자식으로 UI를 올리면
/// `UiScale`이 스테이지(Px 크기)를 통째로 스케일하므로, 내부의 Px(폰트)든
/// Percent(위치·간격)든 모두 같은 비율로 움직인다(월드의 `AutoMin`과 대칭).
///
/// `root_bundle`(마커·`GlobalZIndex`·`GameplayEntity` 등)은 최상위 컨테이너에
/// 붙는다. Bevy 0.19의 `Children`는 `linked_spawn`이라, 기존 despawn 쿼리
/// (`With<Marker>`)가 컨테이너를 지우면 스테이지·자식까지 재귀로 함께 사라진다 —
/// 그래서 자식에는 화면 마커를 중복으로 달지 않는다.
pub fn spawn_stage(commands: &mut Commands, root_bundle: impl Bundle) -> Entity {
    let outer = commands
        .spawn((
            root_bundle,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();
    commands
        .spawn((
            ChildOf(outer),
            Node { width: Val::Px(WINDOW_WIDTH), height: Val::Px(WINDOW_HEIGHT), ..default() },
        ))
        .id()
}

/// 물리 뷰포트 안에 16:9(1280:720) 최대 사각형을 중앙 배치한 (위치, 크기)를 물리
/// 픽셀로 돌려준다. 창이 16:9보다 넓으면 좌우, 높으면 상하에 여백(레터박스)이 생긴다.
pub fn letterbox_rect(phys_w: u32, phys_h: u32) -> (UVec2, UVec2) {
    // 0 크기(최소화 등)에서도 호출부에 의존하지 않고 안전하게: 최소 1px로 취급
    // (clamp(1, 0)는 min>max라 패닉하므로 여기서 미리 막는다).
    let phys_w = phys_w.max(1);
    let phys_h = phys_h.max(1);
    let target = WINDOW_WIDTH / WINDOW_HEIGHT; // 16:9
    let (w, h) = (phys_w as f32, phys_h as f32);
    let (vw, vh) = if w / h > target {
        ((h * target).round(), h) // 창이 더 넓음 → 높이에 맞추고 좌우 여백
    } else {
        (w, (w / target).round()) // 창이 더 높거나 같음 → 폭에 맞추고 상하 여백
    };
    let vw = (vw as u32).clamp(1, phys_w);
    let vh = (vh as u32).clamp(1, phys_h);
    let pos = UVec2::new((phys_w - vw) / 2, (phys_h - vh) / 2);
    (pos, UVec2::new(vw, vh))
}

/// 카메라 뷰포트를 16:9 레터박스 사각형으로 맞춰, 월드가 UI 스테이지처럼 정확히
/// 1280×720만 보이게 한다(그 밖은 카메라 clear로 검은 여백). 이게 없으면
/// `AutoMin`이 16:9가 아닌 창에서 720 너머(배경 없는 영역)까지 비춰 총알이 화면
/// 밖으로 새 보인다. 뷰포트가 16:9라 UI의 physical_viewport_size도 16:9가 되어
/// `sync_ui_scale`의 배율과 정확히 맞아떨어져 월드·UI가 함께 정렬된다.
pub fn sync_letterbox(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Camera2d>>,
) {
    let Ok(window) = windows.single() else { return };
    let (pw, ph) = (window.physical_width(), window.physical_height());
    if pw == 0 || ph == 0 {
        return;
    }
    let (pos, size) = letterbox_rect(pw, ph);
    for mut cam in &mut cameras {
        // 값이 바뀔 때만 갱신해 불필요한 변경(렌더 그래프 재구성)을 피한다.
        let changed = cam
            .viewport
            .as_ref()
            .map(|v| v.physical_position != pos || v.physical_size != size)
            .unwrap_or(true);
        if changed {
            cam.viewport = Some(Viewport { physical_position: pos, physical_size: size, ..default() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_one_at_reference_resolution() {
        // 기준 해상도에선 배율 1.0 — 네이티브 기본 창에서 no-op임을 보장.
        assert!((ui_scale_for(1280.0, 720.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_halves_when_viewport_halves() {
        // 뷰포트가 절반이면 UI(Px)도 절반으로 줄어 화면 밖으로 넘치지 않는다.
        assert!((ui_scale_for(640.0, 360.0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_doubles_on_larger_viewport() {
        assert!((ui_scale_for(2560.0, 1440.0) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_uses_shorter_axis_to_contain() {
        // 폭은 여유(1.0)지만 높이가 빠듯(0.5)한 창 → contain 위해 더 작은 0.5 선택.
        assert!((ui_scale_for(1280.0, 360.0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn letterbox_fills_exactly_at_16_9() {
        // 정확히 16:9면 여백 없이 전체를 채운다.
        let (pos, size) = letterbox_rect(2560, 1440);
        assert_eq!(pos, UVec2::ZERO);
        assert_eq!(size, UVec2::new(2560, 1440));
    }

    #[test]
    fn letterbox_pillarboxes_wide_window() {
        // 16:9보다 넓은 창 → 높이를 채우고 좌우 여백(pos.x>0, pos.y==0).
        let (pos, size) = letterbox_rect(1600, 720);
        assert_eq!(size, UVec2::new(1280, 720));
        assert_eq!(pos, UVec2::new(160, 0));
    }

    #[test]
    fn letterbox_letterboxes_tall_window() {
        // 16:9보다 높은 창 → 폭을 채우고 상하 여백(pos.y>0, pos.x==0).
        let (pos, size) = letterbox_rect(1280, 1000);
        assert_eq!(size, UVec2::new(1280, 720));
        assert_eq!(pos, UVec2::new(0, 140));
    }

    #[test]
    fn letterbox_handles_zero_without_panic() {
        // 0 크기(최소화)에서도 패닉 없이 유효한(≥1) 뷰포트를 돌려준다.
        let (_pos, size) = letterbox_rect(0, 0);
        assert!(size.x >= 1 && size.y >= 1);
    }
}
