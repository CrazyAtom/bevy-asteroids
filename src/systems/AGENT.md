# AGENT.md — `src/systems/`

엔티티 **교차 시스템**. 한 오브젝트에 속하지 않고 여러 오브젝트를 다루는 로직.

## 모듈

- **`movement.rs`** — `FixedUpdate`에서 `apply_velocity`(속도 적분) · `apply_spin`(회전) · `wrap_around`(화면 순환). `Velocity`/`AngularVelocity`/`Wrapping`을 가진 모든 엔티티에 작동.
- **`collision.rs`** — 충돌·피해. `bullet_vs_asteroid`(분열·드롭·점수), `bullet_vs_ufo`, `beam_vs_targets`(특수무기 빔), `player_damage`(소행성/UFO/보스/적총알 피격 → 목숨·리스폰·게임오버; **실드 시 무피해**). 판정은 `core::logic`의 `circles_overlap`/`segment_circle_hit`.
- **`stage.rs`** — **진행 오케스트레이션**. `Progression{cycle, stage_in_cycle, order, phase, wave_in_stage}` 리소스 + 순수 함수(`pick_cycle_order`·`advance_stage`·`stage_wave_number`·`theme_params`·`themed_wave_count/speed`). `StagePlugin`이 테마 웨이브를 스폰하고 웨이브 N회 후 보스로 전환.

## 규칙

- `collision`/`stage`는 `Update`(+`run_if(Playing)`), `movement`는 `FixedUpdate`.
- **난이도는 `stage_wave_number(&prog)`를 `core::logic`의 웨이브 공식에 넣고 `theme_params` 배율을 곱해** 산출합니다. 난이도 수치를 직접 박지 마세요.
- 보스 격파/스테이지 전환 등 **상태 전이는 순수 함수(`advance_stage` 등)로 분리**해 테스트합니다. 시스템은 그 함수를 호출하는 얇은 껍질로.
- 보스전 진입/진행 중엔 일반 웨이브·타이머 UFO를 스폰하지 않습니다(`phase == Waves` 게이트).
- `Progression`은 `state.rs::reset_game`에서 초기화됩니다(재시작 시 재셔플).
