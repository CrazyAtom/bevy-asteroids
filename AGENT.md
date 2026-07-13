# AGENT.md — Bevy Asteroids

AI 코딩 에이전트를 위한 저장소 지침입니다. 코드 작업 전 이 문서를 먼저 읽고, 해당 폴더의 `src/<folder>/AGENT.md`도 필요 시 로드하세요.

## 프로젝트 개요

Rust + [Bevy](https://bevyengine.org) `0.19` 로 만든 2D 애스테로이드 슈터. **단일 크레이트**를 도메인별 폴더로 그룹핑하고, 기능은 Bevy `Plugin` 단위로 분리합니다. 그래픽·사운드는 외부 에셋 없이 빌드 시 자체 생성합니다.

## 기술 스택

- Rust edition 2021 / Bevy `0.19`(`wav` feature) / rand `0.10` / bevy-persistent `0.11` / serde / dirs
- `[build-dependencies] resvg`(SVG→PNG 래스터화)

## 아키텍처

```
src/
├── main.rs      # App 조립: 플러그인 등록, 윈도우/카메라, AssetPlugin 경로
├── ui.rs        # HUD·배너·보스 체력 바·게임오버
├── core/        # 게임 무관 코어 (config·logic·components·state)
├── entities/    # 게임 오브젝트 (asteroid·boss·bullet·player·powerup·ufo)
├── systems/     # 엔티티 교차 시스템 (collision·movement·stage)
└── fx/          # 연출·에셋 (sprites·animation·audio·background·effects·shake)
```

각 폴더의 세부 규칙은 `src/<folder>/AGENT.md`를 참고하세요.

## 필수 명령

```bash
cargo build
cargo test                               # 전부 통과해야 함
cargo clippy --all-targets -- -D warnings  # 경고 0 유지
```

- 실기 확인이 필요하면 창이 뜨는 GUI 앱이므로, 짧게 실행 후 종료해 startup 패닉/로그만 확인하세요(렌더 결과는 사람이 확인).

## 핵심 규칙

### Bevy 0.19 API

- **불확실한 API는 반드시 설치된 크레이트 소스로 검증**하세요(`~/.cargo/registry/src/.../bevy-*`). 버전에 따라 시그니처가 자주 바뀝니다.
- 이 프로젝트는 **Events→Messages** 명칭을 씁니다: `Message`/`MessageReader`/`MessageWriter`/`add_message`.
- UI 이미지는 `ImageNode`, 존재 여부 쿼리는 `Has<T>`, 단일 엔티티는 `query.single()`(Result 반환).
- 리소스가 스케줄 시작 전에 필요하면 **플러그인 `build()`에서 즉시 `insert_resource`** 하세요(지연 명령은 같은 프레임 소비 시 아직 반영 안 됨 — `SpriteAssets`가 그 예).

### 테스트

- **순수 로직/함수만 테스트**합니다(충돌 판정·진행 전이·난이도·보스 체력 등). 렌더링·연출은 테스트하지 않습니다.
- 시스템 테스트는 `bevy::ecs::system::RunSystemOnce::run_system_once` 패턴을 씁니다.
- 스폰 함수가 `&SpriteAssets`를 요구하면 테스트에선 `crate::fx::sprites::dummy_sprite_assets()`(`#[cfg(test)]`)를 삽입/전달하세요.

### 아트/사운드 파이프라인 (자체 완결)

- 스프라이트: `assets/sprites/src/*.svg`(커밋) → `build.rs`가 `resvg`로 `assets/sprites/*.png` 생성 → **PNG는 `.gitignore`**.
- 사운드: `build.rs`가 코드로 WAV 합성 → `assets/sounds/*.wav` → **WAV는 `.gitignore`**.
- 스타일: 플랫 채색 + 두꺼운 어두운 외곽선 + 하이라이트 1톤(카툰). `build.rs`는 `cargo:rerun-if-changed`로 원본 변경 시에만 재생성.

### 튜닝/밸런스

- 모든 튜닝 값(속도·크기·난이도·타이밍·색·Z레이어)은 `src/core/config.rs` 상수에 모읍니다. 밸런스 조정은 여기서.

### Git

- 커밋 메시지·PR 본문은 **한국어**로 작성합니다.
- 커밋 말미에 `Co-Authored-By: Claude ...` 라인을 넣습니다.
- main에서 직접 작업하지 말고 **브랜치 먼저**. PR/푸시는 이 저장소 소유 계정(**CrazyAtom**)으로 합니다(`gh auth switch --user CrazyAtom`).

## 개발 워크플로

큰 기능: 브레인스토밍 → 설계(`docs/superpowers/specs/`) → 구현 계획(`docs/superpowers/plans/`) → 태스크별 구현 → 리뷰. 작은 튜닝/버그픽스: 가벼운 브랜치 + 즉시 검증(build/test/clippy/실행) 루프.

## 디버그 편의

- `F1`(디버그 빌드 한정, `cfg(debug_assertions)`): HUD 확인용으로 모든 파워업·목숨·특수충전 부여. 부여 실드가 무적을 줘 멈춰서 확인 가능.
