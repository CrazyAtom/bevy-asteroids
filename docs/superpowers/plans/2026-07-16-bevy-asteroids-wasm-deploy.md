# 웹(WASM) 배포 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 같은 코드베이스에서 `wasm32-unknown-unknown` 타깃을 추가로 빌드해 GitHub Pages에 배포하고, `main` push 시 GitHub Actions로 자동 갱신한다. 네이티브 데스크톱 빌드·동작은 완전히 보존한다.

**Architecture:** 웹은 "추가 타깃". 플랫폼이 갈리는 최소 지점(에셋 경로·최고점수 저장·난수 소스·캔버스)만 `#[cfg(target_arch="wasm32")]`/`cfg(target_family="wasm")`로 분기하고 나머지는 공유. `trunk`가 wasm 번들(+wasm-opt)을 만들고, GitHub Actions가 빌드→Pages 배포. 생성 에셋(gitignore된 PNG/WAV)은 CI의 `build.rs`가 재생성.

**Tech Stack:** Rust, Bevy 0.19, trunk, wasm-bindgen, bevy-persistent(웹=localStorage), GitHub Actions/Pages.

## Global Constraints

- **네이티브 보존**: 데스크톱 `cargo run`/`cargo build`/`cargo test`(123 통과)/`cargo clippy --all-targets -- -D warnings`(경고 0) 동작·결과 불변. 모든 웹 전용 코드는 `cfg` 뒤에.
- **데스크톱 브라우저 + 키보드 전용**. 모바일/터치는 범위 밖.
- 툴: `trunk`. 배포 서브경로: `trunk build --release --public-url /bevy-asteroids/`(Pages 프로젝트 사이트가 `https://crazyatom.github.io/bevy-asteroids/`).
- **bevy-persistent 웹 스토리지**: wasm에서 `.path(p)`는 `p`가 `"local"`/`"session"` 접두어로 시작해야 하며 아니면 **패닉**(`builder.rs:112-131`). → 최고점수 경로를 wasm에서 `"local/bevy-asteroids-highscore"`로.
- **getrandom 웹 백엔드**: `rand 0.10`의 난수 소스. wasm에서 별도 설정 필요(아래 Task 1에서 확정 — 빌드 에러가 정확히 안내).
- 불확실한 Bevy 0.19 API(캔버스 필드명 등)는 설치 소스로 검증.
- Git: 커밋/PR 한국어. 커밋 말미 `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`. push/PR은 **CrazyAtom** 계정(`gh auth switch --user CrazyAtom`).
- 이 기능은 인프라라 신규 단위 테스트 없음. 검증 = 빌드/컴파일 명령 + 사람 브라우저 확인.

---

## File Structure

| 파일 | 태스크 | 책임 |
|---|---|---|
| `src/main.rs` | 1 | `asset_path()`·HighScore 경로 cfg 분기, `Window` 캔버스 설정 |
| `Cargo.toml` | 1 | wasm 타깃 전용 `getrandom` 의존성 |
| `.cargo/config.toml` | 1 | wasm 타깃 rustflags(getrandom 백엔드, 필요 시) |
| `index.html` | 2 | trunk 진입점: 캔버스 + rust/copy-dir 지시 + 최소 CSS |
| `.gitignore` | 2 | `/dist` 무시 |
| `.github/workflows/deploy.yml` | 3 | wasm 빌드 → Pages 자동 배포 |

---

## Task 1: 코드 wasm 호환 — cfg 분기 + getrandom + 캔버스

**목표 산출물:** `cargo build --target wasm32-unknown-unknown`이 성공하고, 네이티브(`cargo test`/`cargo run`/`cargo clippy`)는 불변.

**Files:**
- Modify: `src/main.rs`
- Modify: `Cargo.toml`
- Create(필요 시): `.cargo/config.toml`

**Interfaces:**
- Produces: `fn asset_path() -> String`(cfg 분기), `setup_high_score`의 cfg 분기된 `path`, `Window`에 캔버스 설정. wasm 빌드가 통과하는 상태.

- [ ] **Step 1: wasm 타깃 추가 + 최초 빌드로 RED 확인**

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown 2>&1 | tail -30
```
Expected: 실패. 가장 먼저 `getrandom`(난수) 관련 에러가 뜬다(예: "the wasm32-unknown-unknown target is not supported by default ... getrandom" 또는 `getrandom_backend` 안내). 이 에러 메시지가 **정확한 해결책을 안내**한다 — 다음 스텝에서 그에 맞춘다.

- [ ] **Step 2: getrandom 웹 백엔드 설정**

`Cargo.toml` 하단에 wasm 타깃 전용 의존성 추가(에러가 `getrandom 0.3` 계열을 가리키는 일반적 경우):

```toml
# 웹(wasm)에서만 난수 소스로 브라우저 crypto를 쓰도록 getrandom 백엔드 활성. 네이티브 무영향.
[target.'cfg(target_family = "wasm")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```

그리고 `.cargo/config.toml` 생성(getrandom 0.3이 요구하는 cfg 플래그):

```toml
[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```

주의: 실제 `getrandom` 버전/방식은 Step 1 에러가 정본이다. 만약 에러가 `getrandom 0.2`(`js` feature)나 다른 버전을 가리키면 그에 맞춰 버전/feature/flag를 조정하라(예: `0.2`면 `features=["js"]`, config 플래그 불필요). `cargo tree -i getrandom --target wasm32-unknown-unknown`으로 어느 버전이 난수 소스인지 확인 가능.

- [ ] **Step 3: 재빌드로 getrandom 해결 확인, 다음 에러(에셋/HighScore) 관찰**

```bash
cargo build --target wasm32-unknown-unknown 2>&1 | tail -30
```
Expected: getrandom 에러 해소. 이제 컴파일이 진행되며 코드 레벨 문제는 없을 수 있으나(런타임 패닉은 별개), 빌드 자체는 통과할 가능성이 높다. `env!("CARGO_MANIFEST_DIR")`는 컴파일 타임 상수라 wasm 빌드도 컴파일은 됨(런타임에만 무의미). HighScore 경로 패닉은 **런타임**이라 빌드는 통과. 빌드가 통과하면 Step 4~6은 런타임 정합성(웹에서 실제로 동작)을 위한 필수 cfg 분기다.

- [ ] **Step 4: AssetPlugin 경로 cfg 분기** — `src/main.rs`

`main()` 위에 헬퍼 추가:

```rust
/// 에셋 루트 경로. 네이티브는 절대경로(IDE 실행 대응), wasm은 상대경로(HTTP 로딩).
fn asset_path() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string()
    }
    #[cfg(target_arch = "wasm32")]
    {
        "assets".to_string()
    }
}
```

그리고 기존 `AssetPlugin` 설정을 다음으로 교체:

```rust
                .set(AssetPlugin {
                    file_path: asset_path(),
                    ..default()
                }),
```

- [ ] **Step 5: HighScore 경로 cfg 분기** — `src/main.rs` `setup_high_score`

기존 `dir`/`path` 계산을 다음으로 교체(네이티브=파일, wasm=localStorage 키):

```rust
fn setup_high_score(mut commands: Commands) {
    #[cfg(not(target_arch = "wasm32"))]
    let path = dirs::config_dir()
        .map(|d| d.join("bevy-asteroids"))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("highscore.json");
    #[cfg(target_arch = "wasm32")]
    let path = std::path::PathBuf::from("local/bevy-asteroids-highscore");

    commands.insert_resource(
        Persistent::<core::state::HighScore>::builder()
            .name("high score")
            .format(StorageFormat::Json)
            .path(path)
            .default(core::state::HighScore(0))
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .expect("최고점수 리소스 초기화 실패"),
    );
}
```

(주의: 현재 `setup_high_score`가 App 빌드 시점 동기 삽입/Startup 중 어디서 호출되는지 확인해, 기존 호출 방식은 유지하고 내부 `path` 계산만 교체할 것. `dirs`는 wasm에서 `None`을 반환하므로 위 cfg로 wasm 경로가 `"local/..."`이 되어 bevy-persistent 패닉을 피한다.)

- [ ] **Step 6: Window 캔버스 설정** — `src/main.rs` `WindowPlugin`

`Window` 필드에 캔버스 셀렉터 추가(네이티브에선 무시됨). 정확한 필드명은 설치 소스로 검증:

```bash
grep -rn "pub canvas\|fit_canvas_to_parent\|pub struct Window" ~/.cargo/registry/src/*/bevy_window-0.19.0/src/window.rs | head
```
확인 후(0.19 필드명 기준) 예:

```rust
                    primary_window: Some(Window {
                        title: "Bevy Asteroids".into(),
                        resolution: (1280, 720).into(),
                        canvas: Some("#bevy-canvas".into()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
```

만약 `fit_canvas_to_parent`가 0.19에서 다른 이름(예: 별도 컴포넌트)이면 소스에 맞춰 조정하거나 생략하고 `canvas`만 설정한다(반응형은 index.html CSS로도 처리 가능).

- [ ] **Step 7: wasm 빌드 성공 + 네이티브 회귀 없음 검증**

```bash
cargo build --target wasm32-unknown-unknown 2>&1 | tail -5   # 성공
cargo test 2>&1 | grep "test result"                         # 123 passed 유지
cargo clippy --all-targets -- -D warnings 2>&1 | tail -2     # 경고 0
cargo run  # (사람) 데스크톱 창이 지금과 동일하게 뜨고 플레이되는지 짧게 확인
```
Expected: wasm 빌드 성공, `cargo test` 123 passed, clippy 0, 네이티브 실행 불변.

- [ ] **Step 8: 커밋**

```bash
git add src/main.rs Cargo.toml .cargo/config.toml Cargo.lock
git commit -m "feat: wasm 타깃 컴파일 지원(cfg 분기 + getrandom 웹 백엔드)

에셋 경로(asset_path)·최고점수 경로(웹=localStorage)·캔버스 설정을 wasm 한정
cfg로 분기해 네이티브 동작을 보존하며 wasm32 빌드가 통과하게 한다.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 2: trunk 번들 — index.html + 에셋 복사

**목표 산출물:** `trunk build`가 `dist/`에 완전한 웹 번들(index.html + .wasm + .js + assets/)을 만든다. `trunk serve`로 로컬 브라우저 플레이 확인(사람).

**Files:**
- Create: `index.html`(레포 루트)
- Modify: `.gitignore`(`/dist`)

**Interfaces:**
- Consumes: Task 1의 wasm 호환 코드, `Window.canvas = "#bevy-canvas"`.
- Produces: `dist/`(배포 산출물). Task 3의 CI가 이 `trunk build`를 그대로 실행.

- [ ] **Step 1: trunk 설치 확인**

```bash
trunk --version || cargo install --locked trunk
```
Expected: trunk 버전 출력(없으면 설치).

- [ ] **Step 2: index.html 생성**(레포 루트)

`Window.canvas`가 `#bevy-canvas`를 참조하므로 동일 id의 캔버스를 둔다.

```html
<!DOCTYPE html>
<html lang="ko">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>Bevy Asteroids</title>
    <style>
        html, body { margin: 0; padding: 0; height: 100%; overflow: hidden; background: #000; }
        canvas { display: block; width: 100vw; height: 100vh; }
    </style>
    <!-- 이 바이너리를 wasm으로 빌드. wasm-opt로 크기 축소(-Os). -->
    <link data-trunk rel="rust" data-bin="bevy-asteroids" data-wasm-opt="s"/>
    <!-- 빌드 시 생성된 assets/ 를 dist/로 복사(HTTP 로딩 대상). -->
    <link data-trunk rel="copy-dir" href="assets"/>
</head>
<body>
    <canvas id="bevy-canvas"></canvas>
</body>
</html>
```

- [ ] **Step 3: .gitignore에 dist 추가**

`.gitignore`에 한 줄 추가:

```
# trunk 빌드 산출물
/dist
```

- [ ] **Step 4: trunk 빌드로 번들 생성 검증**

```bash
trunk build --release --public-url /bevy-asteroids/ 2>&1 | tail -20
ls -la dist/ dist/assets/sprites/*.png dist/assets/sounds/*.wav 2>&1 | head
```
Expected: 빌드 성공. `dist/`에 `index.html`, `*_bg.wasm`(또는 유사), `*.js`, `assets/`(sprites/*.png, sounds/*.wav 포함)가 존재. 에셋이 없으면 `build.rs`가 안 돈 것 — `cargo build`를 먼저 돌려 `assets/`를 채운 뒤 재시도(trunk가 cargo build를 부르므로 보통 자동 생성됨).

- [ ] **Step 5: (사람) 로컬 브라우저 실행 확인 — 체크포인트**

```bash
trunk serve --release --public-url /
```
브라우저에서 `http://localhost:8080` 접속 → **타이틀 화면·게임플레이·사운드(시작 키 이후)·최고점수 localStorage 유지(플레이→새로고침→유지)** 확인. 콘솔 에러 없는지 확인.
> 이 스텝은 사람만 가능(서브에이전트는 브라우저 불가). 구현 서브에이전트는 `trunk build` 성공까지 검증하고, 이 브라우저 확인은 사람 체크포인트로 남긴다.

- [ ] **Step 6: 커밋**

```bash
git add index.html .gitignore
git commit -m "feat: trunk 웹 번들 진입점(index.html) + dist gitignore

캔버스(#bevy-canvas) + rust/copy-dir 지시로 wasm과 생성 에셋을 dist로 번들.
trunk build로 배포 산출물 생성, trunk serve로 로컬 브라우저 확인.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Task 3: GitHub Actions → Pages 자동 배포

**목표 산출물:** `.github/workflows/deploy.yml`. `main` push 시 CI가 wasm 빌드→Pages 배포. (실제 배포는 브랜치 머지 + Pages 설정 후 동작 — 사람 체크포인트.)

**Files:**
- Create: `.github/workflows/deploy.yml`

**Interfaces:**
- Consumes: Task 2의 `trunk build --release --public-url /bevy-asteroids/`.

- [ ] **Step 1: 워크플로 작성** — `.github/workflows/deploy.yml`

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Rust 툴체인 + wasm 타깃
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - name: trunk 설치
        uses: taiki-e/install-action@v2
        with:
          tool: trunk
      - name: 빌드(에셋은 build.rs가 재생성)
        run: trunk build --release --public-url /bevy-asteroids/
      - name: Pages 아티팩트 업로드
        uses: actions/upload-pages-artifact@v3
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Pages 배포
        id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: YAML 유효성 검증**

```bash
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/deploy.yml')); print('YAML OK')"
```
Expected: `YAML OK`. (문법만 검증. 실제 실행은 머지 후 GitHub에서.)

- [ ] **Step 3: 커밋**

```bash
git add .github/workflows/deploy.yml
git commit -m "ci: GitHub Actions로 wasm 빌드→GitHub Pages 자동 배포

main push 시 wasm32 타깃 + trunk로 빌드(--public-url /bevy-asteroids/)해
Pages에 배포. 에셋은 CI의 build.rs가 재생성.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

- [ ] **Step 4: (사람) Pages 설정 + 실제 배포 확인 — 체크포인트**

> 서브에이전트가 아닌 사람이 수행:
> 1. GitHub 저장소 **Settings → Pages → Source를 "GitHub Actions"** 로 설정(1회).
> 2. 브랜치를 `main`에 머지(또는 PR 머지) → `deploy.yml`가 트리거됨.
> 3. Actions 탭에서 `Deploy to GitHub Pages` 성공 확인.
> 4. `https://crazyatom.github.io/bevy-asteroids/` 접속 → 플레이·에셋 로딩·최고점수 유지 확인.
> 5. 에셋 404 시 `--public-url` 경로 재확인(가장 흔한 실수).

---

## 최종 검증(사람 체크포인트 요약)

- 네이티브: `cargo run`으로 데스크톱 동작 불변, `cargo test` 123 passed, clippy 0.
- 로컬 웹: `trunk serve`로 브라우저 플레이 + 최고점수 localStorage 유지.
- 배포: Actions 성공 + 실제 Pages 링크에서 플레이.

---

## Self-Review 결과(작성자 점검)

- **스펙 커버리지:** §2 트렁크→T2, §3.1 AssetPlugin→T1S4, §3.2 HighScore→T1S5, §3.3 getrandom→T1S2, §3.4 캔버스→T1S6/T2S2, §4 CI/CD→T3, §5 크기(wasm-opt via trunk `data-wasm-opt`)→T2S2(네이티브 프로필 불변으로 §5의 "opt-level 검토"는 wasm-opt로 대체—네이티브 릴리즈 성능 보존), §6 제약→Global Constraints, §7 테스트→각 태스크 검증+사람 체크포인트, §8 파일구조→File Structure, §9 위험(getrandom·public-url·캔버스명)→T1S2/T1S6/T3S4. 누락 없음.
- **플레이스홀더:** getrandom 정확 버전/캔버스 필드명은 "빌드 에러/소스로 확정"으로 명시적 해결 경로 제시(빈칸 아님). 사람 체크포인트는 서브에이전트 불가 작업의 정당한 위임.
- **타입/명칭 일관성:** `asset_path()`·`setup_high_score`·`#bevy-canvas`(index.html ↔ Window.canvas)·`--public-url /bevy-asteroids/`(T2 검증 ↔ T3 CI)·`dist`(copy 대상 ↔ upload path) 일관.
