# 🚀 Bevy Asteroids

Rust 게임 엔진 [Bevy](https://bevyengine.org) `0.19` 로 만든 클래식 **애스토로이드(우주선 슈터)** 게임입니다. 원조 Asteroids(1979)의 벡터 라인 그래픽 스타일을 코드로 재현했으며, 외부 이미지 에셋 없이 순수 코드로만 렌더링합니다.

![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/Bevy-0.19-232326?logo=bevy&logoColor=white)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)
![Status](https://img.shields.io/badge/status-WIP-yellow)

> Bevy의 ECS(Entity-Component-System)를 학습하기 위한 프로젝트입니다.

---

## 🎮 게임 화면

```
                    ▲
                   ╱ ╲            ◇ ◇
                  ╱   ╲         ◇     ◇
                 ╱_ ▽ _╲        ◇  ○  ◇      · ← 총알
                                  ◇   ◇
   Score: 340   Lives: 3          ◇ ◇
```

흰색 벡터 라인으로 그려진 삼각형 우주선과 울퉁불퉁한 다각형 소행성이 검은 우주 공간을 떠다닙니다.
*(실제 스크린샷은 추후 추가 예정)*

---

## ✨ 특징

- **관성 기반 조종** — 추진을 멈춰도 우주선이 관성으로 미끄러집니다.
- **소행성 분열** — 큰 소행성을 쏘면 중간 2개로, 중간은 작은 2개로 쪼개지고, 작은 것은 파괴됩니다.
- **화면 순환(wrap-around)** — 화면 밖으로 나간 물체는 반대편에서 다시 등장합니다.
- **목숨 & 점수** — 목숨 3개, 크기별 점수, 실시간 HUD 표시.
- **웨이브** — 모든 소행성을 부수면 새 웨이브가 등장합니다.
- **게임오버 & 재시작** — 목숨이 0이 되면 게임오버, `R` 로 재시작.
- **벡터 라인 렌더링** — Bevy `Gizmos` 즉시 모드로 흰색 외곽선만 그립니다(에셋 0개).

---

## 🕹️ 조작

| 키 | 동작 |
|---|---|
| `←` `→` | 우주선 회전 |
| `↑` | 추진(가속) |
| `Space` | 총알 발사 |
| `R` | 재시작 (게임오버 시) |

---

## 🛠️ 기술 스택

- **언어**: Rust (edition 2021)
- **게임 엔진**: [Bevy](https://bevyengine.org) `0.19` (ECS, 2D 렌더링, 상태 관리)
- **난수**: [rand](https://crates.io/crates/rand) `0.10` (소행성 형태/속도 생성)

---

## 📦 사전 준비

[Rust 툴체인](https://rustup.rs)이 필요합니다.

```bash
# rustup 설치 (미설치 시)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

> macOS(Apple Silicon) 환경에서 개발/검증되었습니다. Bevy가 지원하는 Linux·Windows에서도 동작합니다.

---

## ▶️ 빌드 & 실행

```bash
# 실행 (첫 빌드는 Bevy 의존성 컴파일로 수 분 소요)
cargo run

# 릴리스 빌드로 실행 (성능 최적화)
cargo run --release
```

**개발 중 재컴파일 속도를 높이려면** Bevy의 동적 링크 기능을 사용하세요:

```bash
cargo run --features bevy/dynamic_linking
```

---

## 🧪 테스트

순수 게임 로직(화면 순환, 충돌 판정, 소행성 분열)과 주요 ECS 시스템에 단위 테스트가 있습니다.

```bash
cargo test
```

---

## 🗂️ 프로젝트 구조

기능별 Bevy `Plugin` 단위로 모듈을 분리했습니다.

```
src/
├── main.rs          # App 조립, 윈도우/카메라 설정, 플러그인 등록
├── config.rs        # 게임플레이 상수 (창 크기, 속도, 목숨 등)
├── logic.rs         # 순수 함수 + AsteroidSize (Bevy 무관, 단위 테스트 대상)
├── components.rs    # 공유 컴포넌트: Velocity, Collider, Wrapping
├── state.rs         # GameState(Playing/GameOver), Score/Lives 리소스
├── movement.rs      # 속도 적분 + 화면 순환 (FixedUpdate)
├── player.rs        # 우주선: 스폰, 입력(회전/추진), 렌더
├── bullet.rs        # 총알: 발사, 수명, 렌더
├── asteroid.rs      # 소행성: 웨이브 스폰, 다각형 생성, 분열, 렌더
├── collision.rs     # 충돌: 총알↔소행성, 우주선↔소행성
└── ui.rs            # HUD, 게임오버 화면, 재시작
```

---

## 📐 설계 문서

- [게임 설계 문서](docs/superpowers/specs/2026-07-10-bevy-asteroids-design.md)
- [구현 계획](docs/superpowers/plans/2026-07-10-bevy-asteroids.md)

---

## 📄 라이선스

개인 학습용 프로젝트입니다.
