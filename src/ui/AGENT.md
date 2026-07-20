# AGENT.md — `src/ui/`

**UI·화면·메뉴**. HUD·오버레이·타이틀/일시정지/게임오버 화면과 이들이 공유하는 메뉴 엔진, 그리고 창 크기 대응 스케일링. 모든 시스템은 `UiPlugin`(`ui.rs`) 하나로 등록된다.

## 모듈

- **`hud.rs`** — 인게임 상단 HUD: 점수/최고점/스테이지 텍스트, 목숨 하트, 특수무기 충전 슬롯, 활성 파워업 배지. `RunPhase::Running`에서만 갱신.
- **`boss_bar.rs`** — 화면 상단 보스 체력 바. 보스가 존재할 때만 표시/갱신.
- **`banner.rs`** — 중앙 배너 알림: 스테이지 시작(`STAGE c-s 테마명`)과 보스 등장(`! BOSS !`). `LastStage` 리소스로 중복 알림 억제.
- **`title.rs`** — 타이틀(아케이드 어트랙트 모드). 함선이 타이틀 글자를 좌→우로 훑는 인트로(추진 잔상) → HDR+Bloom 발광 타이틀 + 정박 회전 함선 + 메뉴. 타이틀 문구는 월드 공간 `Text2d`라 스프라이트처럼 Bloom을 탄다.
- **`pause.rs`** — 일시정지: `ESC` 토글, 어둠 오버레이, `RESUME`/`RESTART`/`QUIT TO TITLE` 메뉴. `RunPhase::Paused`가 실제 정지 상태.
- **`game_over.rs`** — 게임오버 화면 + 메뉴(`RESTART`/`QUIT TO TITLE`).
- **`help.rs`** — 조작키 안내 + 아이템 아이콘 범례 HELP 오버레이. 타이틀/일시정지 메뉴의 HELP 항목이 `HelpOpen`을 켜면 어둠 오버레이 + 패널을 띄우고, **아무 키로 닫는다**. 열려 있는 동안 하부 메뉴 입력은 억제된다.
- **`menu.rs`** (`pub(crate)`) — 타이틀·일시정지·게임오버가 공유하는 **방향키 메뉴 엔진**: `MenuAction`(START/RESUME/RESTART/HELP/ToggleMusic/QUIT 등), `MenuItem`/`MenuSelection`, 이동(`menu_move`)·확정(`menu_activate`)·강조(`highlight_menu`). 음악 토글 항목은 `MusicMenuItem` 마커 + `update_music_menu_label`(현재 ON/OFF 반영, `highlight_menu` **이후** 실행).
- **`scaling.rs`** — 기준 해상도 **1280×720**로 디자인한 UI(Px)를 현재 뷰포트에 비례 스케일(`UiScale`)하고, 카메라를 16:9 **레터박스**로 맞춰 월드도 정확히 1280×720만 보이게 한다(`ui_scale_for`·`letterbox_rect`는 순수 함수 — 단위 테스트 대상). 웹에서 창 크기가 제각각이어도 UI와 월드가 같은 배율로 정렬된다.

## 규칙

- **모든 UI 시스템은 `ui.rs`의 `UiPlugin::build`에서 등록**한다. 화면별 스폰/디스폰은 `OnEnter`/`OnExit(상태)`에, 갱신은 `Update` + `run_if(상태)`에 건다.
- **상태 축을 정확히 구분**한다: `GameState{Title, Playing, GameOver}`(화면 전환) vs `RunPhase{Running, Paused}`(Playing 하위 진행/정지). 인게임 HUD·보스바는 `RunPhase::Running`에서만 갱신해 일시정지 중 정지시킨다.
- **메뉴를 새로 쓰지 말고 `menu.rs` 엔진을 재사용**한다. 새 항목은 `MenuAction` 변형 + `menu_activate` arm + 해당 화면에 `MenuItem` 스폰으로 추가한다. 동적 라벨(음악 ON/OFF처럼)은 `highlight_menu`가 정적 라벨로 덮어쓴 **뒤** 실행되는 별도 시스템으로 갱신한다.
- **HELP가 열려 있으면 하부 메뉴 입력을 억제**한다(`run_if(... .and_then(not(help_is_open)))`). 여는 프레임엔 아직 help가 꺼져 있어 `menu_activate`가 정상 실행되어 `HelpOpen`을 켠다 — 이 1프레임 순서 의존을 깨지 말 것.
- **좌표·크기는 1280×720 기준**으로 잡는다. `scaling`이 뷰포트 대응을 전담하므로 개별 UI가 창 크기를 직접 읽지 않는다.
- 웹 빌드에서 의미 없는 항목(`QUIT` 등)은 `#[cfg(not(target_family = "wasm"))]`로 분기한다.
