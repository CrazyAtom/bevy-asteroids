# 🚀 Bevy Asteroids

Rust 게임 엔진 [Bevy](https://bevyengine.org) `0.19` 로 만든 **애스테로이드(우주선 슈터)** 게임입니다. 클래식 Asteroids에서 출발해 파워업·특수무기·적 UFO·**스테이지/테마/보스**·**배경 음악**·타이틀/일시정지 메뉴까지 갖췄으며, 모든 그래픽·사운드·음악은 **외부 다운로드 에셋 없이** 자체 제작 SVG와 코드에서 빌드 시 생성합니다. 데스크톱과 **웹(WebAssembly)** 양쪽에서 동작합니다.

![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/Bevy-0.19-232326?logo=bevy&logoColor=white)
![Platform](https://img.shields.io/badge/platform-Web%20%7C%20macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue)

> 🎮 **웹에서 바로 플레이**: <https://crazyatom.github.io/bevy-asteroids/> (`main` 푸시 시 GitHub Actions가 WASM으로 빌드해 GitHub Pages에 자동 배포)

---

## ✨ 특징

- **관성 기반 조종** — 추진을 멈춰도 관성으로 미끄러지고, 마찰/브레이크로 감속합니다.
- **소행성 분열** — 큰 소행성 → 중간 2개 → 작은 2개로 쪼개집니다(크기별 속도·점수).
- **화면 순환(wrap-around)** — 화면 밖으로 나간 물체는 반대편에서 다시 등장합니다(얼음 테마에서는 벽 반사).
- **적 UFO** — 2종(대형=무작위 사격, 소형=조준 사격), 웨이브에 따라 등장 빈도 상승.
- **파워업 5종** — 실드 · 연사 · 확산탄(3→6→9 누적) · 추가 목숨 · 특수무기 충전. 가까이 가면 **자석**으로 끌려옵니다.
- **특수무기 5종** — 레이저빔 · 스캐터노바(전방위 산탄) · 유도미사일 · 쇼크웨이브(충격파 링) · 실드버스트. **카트라이더식 FIFO 큐**로 획득 순서대로 쌓이고 `X`로 맨 앞부터 발동합니다.
- **하이퍼스페이스** — 무작위 위치 순간이동(`H`).
- **스테이지 · 테마 · 보스** — 무작위 순환하는 **6개 테마 스테이지**(ASTEROID BELT · ALIEN FLEET · SOLAR FLARE · FROZEN FIELD · EM STORM · BLACK HOLE), 각 테마마다 전용 **보스**(체력 바). 사이클마다 난이도가 상승합니다.
- **테마별 환경 트위스트** — 얼음 지대(미끄러운 조종 + 벽 반사) · 전자기폭풍(안개로 시야 제한) · 블랙홀(중력장으로 빨려듦).
- **배경 음악(BGM)** — 절차 생성 칩튠 7곡(타이틀 + 테마 6곡). `M` 키나 메뉴로 음소거를 토글하며 설정은 저장됩니다.
- **타이틀 · 일시정지 · HELP** — 아케이드 어트랙트 타이틀 화면, `ESC` 일시정지 메뉴, 조작키·아이템 범례를 보여주는 HELP 오버레이.
- **연출** — 카툰 스프라이트, 폭발 애니메이션, 파티클, 화면 흔들림, 성운 배경, 효과음.
- **최고 점수 저장** — 로컬에 영속화(웹은 브라우저 저장소).
- **반응형 화면** — 1280×720 기준으로 디자인하고 창 크기에 맞춰 비례 스케일 + 16:9 레터박스.

---

## 🕹️ 조작

| 키 | 동작 |
| --- | --- |
| `←` `→` | 우주선 회전 (탭=미세 조준, 홀드=가속) |
| `↑` | 추진(가속) |
| `↓` | 브레이크/후진 |
| `Space` | 총알 발사 |
| `X` | 특수무기 발동 (큐 맨 앞부터) |
| `H` | 하이퍼스페이스(순간이동) |
| `Enter` | 메뉴 선택 확정 |
| `ESC` | 일시정지 / 메뉴 |
| `M` | 배경 음악 켜기/끄기 |
| `R` | 재시작 (게임오버 시) |
| `F1` | **디버그 전용** — HUD 확인용으로 파워업·목숨·충전 부여 (디버그 빌드만) |

> 타이틀·일시정지 메뉴에서 **HELP**를 열면 전체 조작키와 아이템 아이콘 범례를 볼 수 있습니다.

---

## 🛠️ 기술 스택

- **언어**: Rust (edition 2021)
- **게임 엔진**: [Bevy](https://bevyengine.org) `0.19` — ECS, 2D 스프라이트 렌더링, 상태 관리, 오디오
- **난수**: [rand](https://crates.io/crates/rand) `0.10` (웹은 `getrandom` `wasm_js` 백엔드로 브라우저 crypto 사용)
- **영속화**: [bevy-persistent](https://crates.io/crates/bevy-persistent) `0.11` (최고 점수·음악 설정 JSON)
- **빌드 타임 래스터화**: [resvg](https://crates.io/crates/resvg) (SVG → PNG, `build.rs` 전용)
- **웹 빌드**: [trunk](https://trunkrs.dev) (WASM 번들 + `wasm-opt` 크기 최적화)

---

## 📦 사전 준비

[Rust 툴체인](https://rustup.rs)이 필요합니다.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

> macOS(Apple Silicon)에서 개발/검증되었습니다. Bevy가 지원하는 Linux·Windows에서도 동작합니다.

---

## ▶️ 빌드 & 실행

```bash
# 실행 (첫 빌드는 Bevy 의존성 컴파일로 수 분 소요)
cargo run

# 릴리스 빌드로 실행 (성능 최적화, F1 디버그 기능 없음)
cargo run --release
```

**개발 중 재컴파일 속도를 높이려면** Bevy 동적 링크:

```bash
cargo run --features bevy/dynamic_linking
```

### 웹(WASM)으로 실행

```bash
# 사전 준비: wasm 타깃 + trunk
rustup target add wasm32-unknown-unknown
cargo install trunk

# 로컬 개발 서버 (http://localhost:8080)
trunk serve

# 정적 배포물 빌드 (dist/)
trunk build --release
```

> `main` 브랜치에 푸시하면 [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml)이 WASM으로 빌드해 GitHub Pages(<https://crazyatom.github.io/bevy-asteroids/>)에 자동 배포합니다.

### 에셋 생성 (자동)

그래픽·사운드·음악은 `build.rs`가 **빌드 시 생성**합니다 — 별도 다운로드가 필요 없습니다.

- **스프라이트**: `assets/sprites/src/*.svg`(커밋된 원본) → `resvg`로 `assets/sprites/*.png` 생성
- **효과음**: 코드로 합성한 WAV → `assets/sounds/*.wav`
- **배경 음악**: 다성 칩튠 시퀀서가 합성한 루프 WAV → `assets/music/*.wav`

생성된 PNG·WAV는 `.gitignore` 되며, 원본(SVG·합성 코드)만 커밋됩니다. `cargo:rerun-if-changed`로 원본이 바뀔 때만 재생성됩니다.

---

## 🧪 테스트

순수 게임 로직(충돌 판정·소행성 분열·난이도·진행 전이·보스 체력·특수무기 큐 등)과 주요 스폰 시스템에 단위 테스트가 있습니다. 렌더링/연출은 관례상 테스트하지 않습니다.

```bash
cargo test
cargo clippy --all-targets -- -D warnings   # 경고 0 유지
```

---

## 🗂️ 프로젝트 구조

단일 크레이트를 **도메인별 폴더**로 그룹핑하고, 기능은 Bevy `Plugin` 단위로 분리합니다.

```text
src/
├── main.rs               # App 조립: 플러그인 등록, 윈도우/카메라, 에셋 경로
├── ui.rs + ui/           # UiPlugin 루트 + 화면/HUD/메뉴
│                         #   hud · boss_bar · banner · title · pause · game_over · help · menu · scaling
├── core/                 # 게임 오브젝트와 무관한 코어
│   ├── config.rs + config/   # 튜닝 상수(재수출) — ship·combat·world·render·stage·boss
│   ├── logic.rs + logic/     # 순수 함수(Bevy App 무관) — geometry·difficulty·env
│   ├── components.rs         # 공유 컴포넌트: Velocity·AngularVelocity·Collider·Wrapping·EdgeReflect·GravityBody
│   └── state.rs              # GameState(Title/Playing/GameOver) + RunPhase(Running/Paused), 리소스, reset_game
├── entities/             # 게임 오브젝트 (각 모듈 = 컴포넌트 + 스폰 + 플러그인 + 시스템)
│   ├── player.rs             # 우주선: 입력·추진·화염·실드·하이퍼스페이스
│   ├── special_weapon.rs + special_weapon/  # 특수무기 5종 FIFO 큐 + 서브모듈(beam·nova·missile·shockwave·shield_burst)
│   ├── asteroid.rs           # 소행성 스폰/분열
│   ├── bullet.rs             # 총알(아군/적)
│   ├── ufo.rs                # 적 UFO
│   ├── powerup.rs            # 파워업 5종 + 자석
│   ├── black_hole.rs         # 블랙홀 중력장(블랙홀 테마)
│   ├── boss.rs + boss/       # 보스 공통(6종 디스패치) + 전용 로직(golem·tesla·singularity)
├── systems/              # 엔티티 교차 시스템
│   ├── movement.rs           # 속도 적분·회전·화면 순환/벽 반사 (FixedUpdate)
│   ├── collision.rs          # 충돌·피격·보스 전투
│   └── stage.rs              # 진행(Progression)·테마/보스 오케스트레이션·환경 동기화
└── fx/                   # 연출·에셋
    ├── sprites.rs            # SpriteAssets(모든 스프라이트 핸들) + 크기 매핑
    ├── animation.rs          # 프레임 애니메이션(폭발)
    ├── audio.rs              # 효과음 재생
    ├── music.rs              # 배경 음악(BGM): 상태/테마별 트랙 재생·음소거 지속
    ├── background.rs         # 테마별 배경
    ├── fog.rs                # 시야 제한 안개(전자기폭풍 테마)
    ├── effects.rs            # 파티클
    └── shake.rs              # 화면 흔들림
```

> 구조 규칙: **폴더 = 기능, 파일 = 변형/관심사.** 한 기능 안에서 변형(보스 종류, 특수무기, 도메인 상수)이 늘어 파일이 비대해지면 서브모듈로 나누고, 루트 파일은 공유 인프라·디스패치·재수출만 담당합니다(예: `entities/boss/`, `entities/special_weapon/`, `core/config/`).

빌드/에셋:

```text
build.rs                    # 빌드 시 효과음·배경음악 WAV + 스프라이트 PNG(SVG→resvg) 생성
assets/sprites/src/*.svg    # 자체 제작 SVG (커밋). PNG는 빌드 생성(gitignore)
index.html                  # trunk 웹 빌드 진입점(캔버스 16:9 레터박스)
.github/workflows/deploy.yml # main 푸시 시 WASM 빌드 → GitHub Pages 배포
```

> AI 에이전트용 폴더별 지침은 각 폴더의 `AGENT.md`와 루트 [`AGENT.md`](AGENT.md)를 참고하세요.

---

## 📐 설계 문서

기능별 브레인스토밍 → 설계(spec) → 구현 계획(plan) 순으로 문서화되어 있습니다.

- 설계: [`docs/superpowers/specs/`](docs/superpowers/specs/)
- 계획: [`docs/superpowers/plans/`](docs/superpowers/plans/)

---

## 📄 라이선스

[MIT License](LICENSE) © 2026 Kang Hyun Bu
