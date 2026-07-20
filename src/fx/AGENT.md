# AGENT.md — `src/fx/`

**연출·에셋**: 스프라이트 핸들, 애니메이션, 효과음, 배경 음악(BGM), 배경, 파티클, 화면 흔들림.

## 모듈

- **`sprites.rs`** — `SpriteAssets` 리소스(모든 `Handle<Image>`) + `sprite_size_for`(콜라이더 반경→표시 크기). `SpritesPlugin`이 **플러그인 `build()`에서 즉시 `insert_resource`**(스케줄 시작 전 보장). 테스트용 `dummy_sprite_assets()`(`#[cfg(test)]`).
- **`animation.rs`** — `FrameAnimation`(프레임 이미지 스왑) + `next_frame_index`(순수) + `spawn_explosion_anim`. 마지막 프레임 뒤 despawn.
- **`audio.rs`** — `Sfx` enum + `SfxEvent`(Message) + `play_sfx`(Bevy `AudioPlayer`). 효과음은 이벤트로 요청.
- **`music.rs`** — 배경 음악(BGM). `Track` enum(Title + 테마 6곡) + `Track::for_theme(ThemeId)`, `desired_track(GameState, Option<&Progression>)`(순수, Title→Title·그 외→테마 트랙)으로 원하는 곡을 정하고 `sync_music`이 `CurrentMusic`과 비교해 교체 재생(`AudioPlayer` + `PlaybackSettings::LOOP`, 온디맨드 로드). 음소거는 `MusicEnabled(bool)`를 **`Persistent`로 지속**(`load_music_enabled`, `#[cfg]` 경로 분기 — `state`의 최고점수 저장 패턴 재사용)하고 `toggle_music_key`(**`M`키**)로 토글. WAV는 build.rs가 `assets/music/*.wav`로 절차 생성. 메뉴의 MUSIC 토글은 `ui/menu.rs`가 이 리소스를 뒤집는다.
- **`background.rs`** — 테마별 배경 스프라이트 교체(`update_theme_background`, 상태 비의존 비교).
- **`fog.rs`** — 시야 제한 안개(전자기폭풍 테마). `FogState` 리소스(기본 off, stage의 `sync_fog_state`가 켬), 오버레이가 우주선 추적 + `storm_vision_scale`로 시야창 진동. 가시성 시스템은 상태 무관 상시 실행(게임오버 잔상 방지 패턴).
- **`effects.rs`** — 파티클(`spawn_explosion` = 스파크 파티클 + 애니 폭발), 수명/페이드.
- **`shake.rs`** — trauma 기반 카메라 오프셋. `ShakeEvent`로 트리거, 시간 감쇠.

## 규칙

- **에셋은 플러그인 `build()`에서 로드**하세요(Startup 시스템의 지연 명령은 같은 프레임 소비자보다 늦어 패닉 유발 — `SpriteAssets`가 그 사례). `AssetServer`는 `DefaultPlugins`에서 이미 삽입되어 있습니다.
- 효과음/흔들림은 시스템에서 직접 처리하지 말고 **`SfxEvent`/`ShakeEvent` 메시지**로 요청하세요.
- Z 레이어(배경 < 물체 < 실드 < 빔 < 파티클 < 폭발)는 `core::config`의 `Z_*` 상수를 씁니다.
- 새 스프라이트: `assets/sprites/src/`에 SVG 추가 → `SpriteAssets` 필드 + `build_sprite_assets`/`dummy_sprite_assets`에 로드 추가. PNG는 `build.rs`가 생성(gitignore).
