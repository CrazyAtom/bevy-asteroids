# Bevy Asteroids — 설계 문서

- **작성일**: 2026-07-10
- **목표**: Rust 게임 엔진 Bevy를 학습하기 위해 클래식 애스토로이드(우주선 슈터) 게임을 만든다.
- **환경**: Rust 1.96.1 / cargo 1.96.1, Bevy 최신 안정판, macOS arm64 (Apple Silicon)

## 1. 개요와 범위

원조 Asteroids(1979)를 재현하는 2D 게임. 다음을 **목표 기능 세트(클래식)** 로 포함한다.

- 우주선 회전/추진 조작 (관성: 추진을 멈춰도 속도 유지)
- 총알 발사
- 소행성 분열 (대 → 중 → 소, 소는 파괴 시 소멸)
- 화면 순환(wrap-around): 화면 밖으로 나가면 반대편에서 등장
- 우주선-소행성 충돌 시 목숨 감소, 목숨 3개
- 점수 UI (HUD)
- 게임오버 및 재시작

**범위 밖(향후 확장 후보):** 적 UFO, 하이퍼스페이스, 웨이브별 난이도 상승, 최고점수 저장, 사운드/이펙트.

## 2. 비주얼 스타일

**벡터 라인 그래픽.** 원조 게임처럼 삼각형 우주선, 다각형 소행성을 코드로 `Mesh`를 생성해 흰 선(`LineStrip`)으로만 그린다. 외부 이미지 에셋을 사용하지 않는다. 소행성은 원 둘레 꼭짓점의 반지름을 살짝 랜덤하게 흔들어 울퉁불퉁한 바위 느낌을 낸다.

## 3. 아키텍처 — 플러그인 단위 모듈화

Bevy의 관용적 방식대로 기능별 플러그인으로 코드를 나눈다. 각 파일은 하나의 명확한 책임만 갖는다.

```
src/
├── main.rs          # App 조립, 플러그인 등록, 윈도우 설정
├── game_state.rs    # GameState(States) + Score/Lives 리소스
├── player.rs        # 우주선: 컴포넌트, 스폰, 입력→회전/추진/발사
├── asteroid.rs      # 소행성: 스폰, 크기(대/중/소), 분열 로직
├── bullet.rs        # 총알: 발사, 수명(timer) 후 소멸
├── movement.rs      # 속도 적분(위치 이동) + 화면 순환(wrap)
├── collision.rs     # 총알↔소행성, 우주선↔소행성 충돌 판정
└── ui.rs            # 점수/목숨 HUD, 게임오버 화면
```

각 모듈은 대응하는 Bevy `Plugin`을 노출하고, `main.rs`에서 `add_plugins((...))`로 조립한다.

## 4. 데이터 모델

### Components (엔티티에 붙는 데이터)

- `Player` — 우주선 마커
- `Asteroid { size: AsteroidSize }` — `AsteroidSize` 는 `Large / Medium / Small` enum
- `Bullet { life: Timer }` — 일정 시간 뒤 자동 소멸
- `Velocity(Vec2)` — 선형 속도. 위치를 매 틱 이 값으로 적분
- `Collider { radius: f32 }` — 원형 충돌 판정용 반지름
- `Wrapping` — 화면 밖으로 나가면 반대편으로 순환시키는 대상 마커

### Resources (전역 상태)

- `Score(u32)`, `Lives(u32)`
- `GameState` (Bevy `States`): `Playing` ↔ `GameOver`

## 5. 시스템 & 데이터 흐름

물리 이동은 프레임률에 흔들리지 않도록 `FixedUpdate`(고정 시간간격)에, 입력·렌더·UI는 `Update`(매 프레임)에 배치한다.

| 시스템 | 스케줄 | 책임 |
|---|---|---|
| `player_input` | Update | 키보드 → 회전, 추진력을 Velocity에 가산, 발사 시 총알 스폰 |
| `apply_velocity` | FixedUpdate | `Transform.translation += Velocity * dt` |
| `wrap_around` | FixedUpdate | 화면 경계 넘으면 반대편으로 좌표 순환 |
| `bullet_lifetime` | Update | Timer 만료된 총알 despawn |
| `bullet_vs_asteroid` | Update | 겹치면 총알 소멸 + 소행성 분열/소멸 + 점수 증가 |
| `player_vs_asteroid` | Update | 겹치면 목숨 감소, 우주선 리스폰 또는 GameState::GameOver 전환 |
| `wave_control` | Update | 소행성이 0개가 되면 다음 웨이브 스폰 |
| `update_hud` | Update | 점수/목숨 텍스트 갱신 |

### 조작

- **← →**: 회전
- **↑**: 추진
- **Space**: 발사
- **R**: (게임오버 시) 재시작

### 핵심 메커니즘 설명

- **분열 로직**: 큰 소행성이 총알에 맞으면 `commands.entity(e).despawn()`으로 자신을 제거하고, 그 자리에서 한 단계 작은 소행성 2개를 서로 다른 방향 속도로 스폰한다. `Small`은 파괴 시 아무것도 스폰하지 않는다.
- **관성**: 추진키가 `Velocity`에 힘을 더하기만 하고, 떼어도 속도가 유지된다(마찰 0). 필요 시 매 틱 속도에 감쇠 계수(예: 0.99)를 곱해 감속을 넣을 수 있다. (기본값: 감속 없음, 원조 게임에 가깝게)

## 6. 렌더링 — 벡터 라인

- 우주선: 삼각형 3정점을 `LineStrip` 토폴로지 `Mesh`로 생성.
- 소행성: 원 둘레 N개 정점의 반지름을 랜덤하게 흔든 다각형 `Mesh`.
- 총알: 점 또는 짧은 선.
- 모두 흰색 단색 머티리얼로 선만 표시.

## 7. 테스트 전략

순수 계산 로직을 함수로 분리해 TDD로 먼저 잡고, 그 위에 시스템을 얹는다.

- `wrap_position(pos, bounds)` — 화면 순환 좌표 계산
- `next_asteroid_size(size)` — 분열 시 다음 크기 반환 (Large→Medium, Medium→Small, Small→None)
- `circles_overlap(a, ra, b, rb)` — 원-원 충돌 판정

시스템 수준 테스트는 최소 `App`을 만들어 엔티티를 삽입하고 `app.update()`를 돌린 뒤 컴포넌트/리소스 상태를 검증한다.

## 8. 개발 편의

- Bevy는 구현 시작 시 `cargo add bevy`로 최신 안정 버전을 확인해 고정한다. 버전마다 API가 다르므로 코드는 확정된 실제 버전에 맞춰 작성한다.
- 개발 중 재컴파일 속도를 위해 dev 프로파일에서 Bevy `dynamic_linking` 피처 사용을 고려한다.

## 9. 완료 기준 (Definition of Done)

- 우주선을 회전/추진/발사할 수 있다.
- 소행성이 대→중→소로 분열되고, 소는 파괴 시 사라진다.
- 모든 이동 엔티티가 화면을 순환한다.
- 소행성과 충돌하면 목숨이 줄고, 0이 되면 게임오버 화면이 뜬다.
- 게임오버에서 R로 재시작할 수 있다.
- 점수와 남은 목숨이 화면에 표시된다.
- 순수 로직 함수에 대한 단위 테스트가 통과한다.
