# AGENT.md — `src/fx/`

**연출·에셋**: 스프라이트 핸들, 애니메이션, 사운드, 배경, 파티클, 화면 흔들림.

## 모듈

- **`sprites.rs`** — `SpriteAssets` 리소스(모든 `Handle<Image>`) + `sprite_size_for`(콜라이더 반경→표시 크기). `SpritesPlugin`이 **플러그인 `build()`에서 즉시 `insert_resource`**(스케줄 시작 전 보장). 테스트용 `dummy_sprite_assets()`(`#[cfg(test)]`).
- **`animation.rs`** — `FrameAnimation`(프레임 이미지 스왑) + `next_frame_index`(순수) + `spawn_explosion_anim`. 마지막 프레임 뒤 despawn.
- **`audio.rs`** — `Sfx` enum + `SfxEvent`(Message) + `play_sfx`(Bevy `AudioPlayer`). 효과음은 이벤트로 요청.
- **`background.rs`** — 테마별 배경 스프라이트 교체(`update_theme_background`, 상태 비의존 비교).
- **`effects.rs`** — 파티클(`spawn_explosion` = 스파크 파티클 + 애니 폭발), 수명/페이드.
- **`shake.rs`** — trauma 기반 카메라 오프셋. `ShakeEvent`로 트리거, 시간 감쇠.

## 규칙

- **에셋은 플러그인 `build()`에서 로드**하세요(Startup 시스템의 지연 명령은 같은 프레임 소비자보다 늦어 패닉 유발 — `SpriteAssets`가 그 사례). `AssetServer`는 `DefaultPlugins`에서 이미 삽입되어 있습니다.
- 효과음/흔들림은 시스템에서 직접 처리하지 말고 **`SfxEvent`/`ShakeEvent` 메시지**로 요청하세요.
- Z 레이어(배경 < 물체 < 실드 < 빔 < 파티클 < 폭발)는 `core::config`의 `Z_*` 상수를 씁니다.
- 새 스프라이트: `assets/sprites/src/`에 SVG 추가 → `SpriteAssets` 필드 + `build_sprite_assets`/`dummy_sprite_assets`에 로드 추가. PNG는 `build.rs`가 생성(gitignore).
