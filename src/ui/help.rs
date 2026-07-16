//! 조작키 안내 + 아이템 아이콘 범례를 보여주는 HELP 오버레이.
//!
//! 타이틀/일시정지 메뉴의 HELP 항목이 `HelpOpen`을 켜면 어둠 오버레이 + 1280×720
//! 가상 스테이지 위 패널을 띄운다(스케일·레터박스 자동). 아무 키로 닫고 원래
//! 메뉴로 복귀한다. 새 GameState 대신 리소스+오버레이라, 진입 화면(Title/Paused)에
//! 무관하게 하부 상태를 그대로 둔 채 위에 겹쳐 복귀 경로 추적이 필요 없다.

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::entities::special_weapon::{weapon_icon, SpecialWeaponKind};
use crate::fx::sprites::SpriteAssets;
use crate::ui::scaling::spawn_stage;

/// HELP 오버레이 표시 여부. 메뉴의 ShowHelp가 켜고, 아무 키가 끈다.
#[derive(Resource, Default)]
pub struct HelpOpen(pub bool);

/// HELP 오버레이 최상위 엔티티 마커(어둠 오버레이 + 스테이지 컨테이너에 부착).
#[derive(Component)]
pub(super) struct HelpPanel;

/// `HelpOpen`이 켜져 있으면 true — 하부 메뉴 입력을 억제하는 run 조건.
pub(super) fn help_is_open(help: Res<HelpOpen>) -> bool {
    help.0
}

/// `HelpOpen` 상태에 맞춰 패널을 스폰/디스폰한다(상시 실행, 저비용).
/// `SpriteAssets`는 SpritesPlugin이 빌드 시점에 삽입해 첫 프레임 전부터 존재가
/// 보장되므로 그냥 Res로 받는다(만에 하나 없으면 조용한 잠금 대신 즉시 패닉으로
/// 로딩 순서 회귀를 크게 드러낸다).
pub(super) fn manage_help(
    help: Res<HelpOpen>,
    panel: Query<Entity, With<HelpPanel>>,
    mut commands: Commands,
    assets: Res<SpriteAssets>,
) {
    let exists = !panel.is_empty();
    if help.0 && !exists {
        spawn_help_panel(&mut commands, &assets);
    } else if !help.0 && exists {
        for entity in &panel {
            commands.entity(entity).despawn();
        }
    }
}

/// 패널이 떠 있는 동안 아무 키나 누르면 닫는다. 패널이 실제로 존재하는(=여는
/// Enter가 소비된 다음) 프레임부터만 반응해, 연 입력이 즉시 닫는 것을 막는다.
pub(super) fn close_help(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut help: ResMut<HelpOpen>,
    panel: Query<(), With<HelpPanel>>,
) {
    if panel.is_empty() {
        return;
    }
    if keys.get_just_pressed().next().is_some() {
        help.0 = false;
        // 닫는 키를 소비한다. 이걸 안 하면 같은 프레임에 하부 menu_activate나
        // resume_on_esc가 not(help_is_open) 게이트를 통과해 그 키를 재해석할 수 있다
        // (Enter로 HELP 재오픈, ESC로 게임 재개). clear로 실행 순서와 무관하게 안전:
        // close_help가 먼저면 소비, 하부가 먼저면 아직 help가 열려 게이트 off.
        keys.clear();
    }
}

/// 파워업(4) + 특수무기(5) 아이콘·이름 범례. HUD와 동일한 스프라이트를 쓴다.
fn item_legend(assets: &SpriteAssets) -> [(Handle<Image>, &'static str); 9] {
    [
        (assets.powerup[0].clone(), "SHIELD"),
        (assets.powerup[1].clone(), "RAPID FIRE"),
        (assets.powerup[2].clone(), "SPREAD"),
        (assets.powerup[3].clone(), "EXTRA LIFE"),
        (weapon_icon(SpecialWeaponKind::LaserBeam, assets), "LASER BEAM"),
        (weapon_icon(SpecialWeaponKind::ScatterNova, assets), "SCATTER NOVA"),
        (weapon_icon(SpecialWeaponKind::HomingMissile, assets), "HOMING MISSILE"),
        (weapon_icon(SpecialWeaponKind::Shockwave, assets), "SHOCKWAVE"),
        (weapon_icon(SpecialWeaponKind::ShieldBurst, assets), "SHIELD BURST"),
    ]
}

/// 조작 안내(키 → 동작). 기본 폰트에 화살표 글리프가 없어 ASCII로 표기한다.
const CONTROLS: [(&str, &str); 6] = [
    ("MOVE", "LEFT / RIGHT  rotate,   UP  thrust,   DOWN  brake"),
    ("FIRE", "SPACE"),
    ("SPECIAL", "X    (fires queued weapon)"),
    ("HYPERSPACE", "H    (emergency warp)"),
    ("PAUSE", "ESC"),
    ("MENU", "UP / DOWN  move,   ENTER  select"),
];

fn spawn_help_panel(commands: &mut Commands, assets: &SpriteAssets) {
    let heading = Color::srgb(0.45, 0.85, 1.0);
    let key = Color::srgb(0.95, 0.92, 0.6);
    let body = Color::srgb(0.78, 0.78, 0.78);
    let dim = Color::srgb(0.5, 0.5, 0.5);

    // 어둠 오버레이(전체 창) — 하부 화면을 가린다. 모든 UI 위(z=10).
    commands.spawn((
        HelpPanel,
        GlobalZIndex(10),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.97)),
    ));

    // 내용은 1280×720 스테이지 위(z=11). 제목·힌트는 가운데, 본문은 고정폭
    // 좌측정렬로 열을 맞춘다(표처럼 읽히게).
    let stage = spawn_stage(commands, (HelpPanel, GlobalZIndex(11)));
    commands.entity(stage).with_children(|s| {
        s.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new("HOW TO PLAY"),
                TextFont { font_size: FontSize::Px(32.0), ..default() },
                TextColor(Color::WHITE),
            ));

            // 고정폭 좌측정렬 본문 패널.
            col.spawn(Node {
                width: Val::Px(680.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(14.0),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn((
                    Text::new("CONTROLS"),
                    TextFont { font_size: FontSize::Px(17.0), ..default() },
                    TextColor(heading),
                ));
                // [키 150px 고정폭 셀] [동작] 2열 정렬.
                for (k, action) in CONTROLS {
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(k),
                                TextFont { font_size: FontSize::Px(15.0), ..default() },
                                TextColor(key),
                                Node { width: Val::Px(150.0), ..default() },
                            ));
                            row.spawn((
                                Text::new(action),
                                TextFont { font_size: FontSize::Px(15.0), ..default() },
                                TextColor(body),
                            ));
                        });
                }

                panel.spawn((
                    Text::new("ITEMS"),
                    TextFont { font_size: FontSize::Px(17.0), ..default() },
                    TextColor(heading),
                ));
                // 아이콘+이름을 216px 고정폭 셀로 3개씩 한 줄 → 3열 그리드처럼 정렬.
                let legend = item_legend(assets);
                for chunk in legend.chunks(3) {
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for (icon, name) in chunk {
                                row.spawn(Node {
                                    width: Val::Px(216.0),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(8.0),
                                    ..default()
                                })
                                .with_children(|cell| {
                                    cell.spawn((
                                        ImageNode::new(icon.clone()),
                                        Node {
                                            width: Val::Px(22.0),
                                            height: Val::Px(22.0),
                                            ..default()
                                        },
                                    ));
                                    cell.spawn((
                                        Text::new(*name),
                                        TextFont { font_size: FontSize::Px(13.0), ..default() },
                                        TextColor(body),
                                    ));
                                });
                            }
                        });
                }
            });

            col.spawn((
                Text::new("PRESS ANY KEY TO RETURN"),
                TextFont { font_size: FontSize::Px(13.0), ..default() },
                TextColor(dim),
            ));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::sprites::dummy_sprite_assets;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn manage_help_spawns_then_despawns_panel() {
        let mut app = App::new();
        app.insert_resource(dummy_sprite_assets());
        app.insert_resource(HelpOpen(true));

        app.world_mut().run_system_once(manage_help).unwrap();
        let open = app.world_mut().query::<&HelpPanel>().iter(app.world()).count();
        assert!(open >= 1, "HelpOpen이면 패널(HelpPanel)이 스폰돼야 한다");

        app.world_mut().resource_mut::<HelpOpen>().0 = false;
        app.world_mut().run_system_once(manage_help).unwrap();
        let closed = app.world_mut().query::<&HelpPanel>().iter(app.world()).count();
        assert_eq!(closed, 0, "닫히면 패널이 모두 제거돼야 한다");
    }

    #[test]
    fn close_help_closes_and_consumes_key() {
        let mut app = App::new();
        app.insert_resource(HelpOpen(true));
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::Enter);
        app.insert_resource(input);
        app.world_mut().spawn(HelpPanel); // 패널이 떠 있는 상태

        app.world_mut().run_system_once(close_help).unwrap();

        assert!(!app.world().resource::<HelpOpen>().0, "아무 키로 닫혀야 한다");
        assert!(
            !app.world().resource::<ButtonInput<KeyCode>>().just_pressed(KeyCode::Enter),
            "닫는 키는 소비돼 하부 메뉴/재개로 새지 않아야 한다"
        );
    }

    #[test]
    fn close_help_ignores_input_before_panel_spawns() {
        let mut app = App::new();
        app.insert_resource(HelpOpen(true));
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::Enter); // 여는 Enter가 아직 남아 있는 상황
        app.insert_resource(input);
        // HelpPanel 없음 → 패널이 뜨기 전 프레임엔 닫히지 않아야 한다.
        app.world_mut().run_system_once(close_help).unwrap();
        assert!(app.world().resource::<HelpOpen>().0, "패널 스폰 전엔 닫히지 않아야 한다");
    }
}
