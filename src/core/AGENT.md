# AGENT.md — `src/core/`

게임 오브젝트와 무관한 **코어**: 상수, 순수 로직, 공유 컴포넌트, 전역 상태.

## 모듈

- **`config.rs`** — 모든 튜닝 상수(창 크기, 우주선 물리, 소행성/UFO, 파워업, 스테이지/보스, Z 레이어, 스프라이트 크기, 타이밍). **밸런스/튜닝은 전부 여기서.**
- **`logic.rs`** — 순수 함수 + `AsteroidSize`. **Bevy에 의존하지 않음**(std/rand만). 충돌 판정(`circles_overlap`, `segment_circle_hit`), 화면 순환(`wrap_position`), 난이도 공식(`asteroid_count_for_wave` 등), 감쇠(`decay_trauma`) 등. **단위 테스트 집중 대상.**
- **`components.rs`** — 여러 엔티티가 공유하는 컴포넌트: `Velocity`, `AngularVelocity`, `Collider{radius}`, `Wrapping`(화면 순환), `EdgeReflect`(경계 반사 대상 마커 — 얼음 스테이지에서만 반사, 그 외엔 순환).
- **`state.rs`** — `GameState`(Playing/GameOver), 리소스(`Score`/`Lives`/`HighScore`), `GameplayEntity` 마커, `GameStatePlugin`, `reset_game`(재시작 시 상태 초기화; `pub(crate)`).

## 규칙

- **순수 로직은 `logic.rs`에.** Bevy 타입을 import하지 마세요 — App 없이 단위 테스트 가능해야 합니다. 새 게임 규칙(판정식·수식)은 여기 추가하고 테스트를 씁니다.
- **튜닝 값은 `config.rs` 상수로.** 매직 넘버를 시스템 코드에 박지 마세요.
- **공유 컴포넌트만 `components.rs`에.** 특정 엔티티 전용 컴포넌트는 해당 `entities/` 모듈에 둡니다.
- `GameplayEntity`가 붙은 엔티티는 `OnExit(Playing)`에서 일괄 정리됩니다. 판 동안만 존재하는 엔티티엔 반드시 붙이세요(배경 등 영속 엔티티엔 붙이지 않음).
