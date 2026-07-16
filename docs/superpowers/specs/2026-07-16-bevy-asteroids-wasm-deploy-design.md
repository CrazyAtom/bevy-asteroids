# 웹(WASM) 배포 설계

> 미뤄뒀던 기능 재제안 중 사용자가 선택한 항목(잔여 2종 중 첫째). 목표: 링크 하나로 브라우저에서 플레이 가능한 웹 배포.

**목표:** 같은 코드베이스에서 `wasm32-unknown-unknown` 타깃을 추가로 빌드해 GitHub Pages에 배포하고, `main` push 시 GitHub Actions로 자동 갱신한다. **네이티브 데스크톱 빌드·동작은 완전히 보존한다.**

**핵심 원칙:** 웹은 "추가 타깃". 플랫폼이 갈리는 최소 지점(에셋 경로·최고점수 저장)만 `#[cfg]`로 분기하고, 나머지는 공유. 배포는 `trunk`가 번들링, GitHub Actions가 빌드→배포.

---

## 1. 배경 — 현재 구조와 웹 전환 시 걸림돌

- **build.rs가 에셋을 빌드 시 생성**: `assets/sprites/src/*.svg` → `resvg`로 `assets/sprites/*.png`, 코드 합성으로 `assets/sounds/*.wav`. **PNG/WAV는 `.gitignore`**(SVG 원본만 커밋). `build.rs`·`resvg`는 **호스트에서 도는 빌드 스크립트**라 wasm 타깃 빌드 중에도 정상 생성된다.
- **`AssetPlugin` 절대경로**(`src/main.rs`): `file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/assets")`. 웹엔 파일시스템이 없어 이 경로가 무의미 — 에셋은 HTTP로 상대경로 로딩해야 한다.
- **`HighScore` 파일 저장**: `bevy-persistent` + `dirs::config_dir()`로 파일에 JSON 저장. 웹엔 파일시스템이 없다.
- **`rand 0.10`**: 난수 소스로 `getrandom`을 씀. `getrandom 0.3+`는 `wasm32-unknown-unknown`에서 웹 백엔드 설정이 필요(브라우저 crypto).
- **윈도우**: `title` + `resolution (1280,720)`. 웹은 캔버스가 필요.

**검증된 사실(설치 소스):**
- `bevy-persistent 0.11`은 **웹 스토리지를 내장 지원**한다. `Storage` enum: `Filesystem{path}`(`cfg(not(wasm))`) + `LocalStorage{key}`/`SessionStorage{key}`(`cfg(wasm)`). 빌더 `.path(p)`는 wasm에서 `p`가 `"local"`/`"session"`으로 시작하면 그 접두어를 뗀 나머지를 스토리지 **key**로 쓰고, 아니면 **패닉**한다(`bevy-persistent-0.11.0/src/builder.rs:112-131`). wasm 의존성 `gloo-storage`가 자동으로 딸려온다.
- `dirs 6.0`은 wasm cfg 분기가 있어 컴파일은 되나 경로는 `None`을 반환한다(`dirs-6.0.0/src/lib.rs:28`). → 현재 `.unwrap_or_else(|| PathBuf::from("."))` 폴백은 wasm에서 `"."` 경로가 되어 위 패닉을 유발하므로 **경로 설정을 cfg 분기해야 한다.**
- lockfile에 `wasm-bindgen 0.2.126`이 이미 존재(bevy의 web-sys 경유). `trunk`가 매칭되는 `wasm-bindgen-cli`를 자동 조달한다.

---

## 2. 빌드 툴링 — `trunk`

`trunk`(Bevy 웹 표준)를 쓴다: wasm 빌드 + `wasm-bindgen` + `index.html` 생성 + 에셋 복사 + `wasm-opt` 최적화를 한 번에 처리.

- **`index.html`**(레포 루트): 최소 HTML + 캔버스 + trunk 지시.
  - `<link data-trunk rel="rust" data-bin="bevy-asteroids"/>`(wasm 빌드 대상).
  - `<link data-trunk rel="copy-dir" href="assets"/>`(생성된 `assets/`를 `dist/`로 복사).
  - `<canvas id="bevy-canvas"></canvas>` + 페이지를 꽉 채우는 최소 CSS.
- **`Trunk.toml`**(선택): 빌드 옵션(release, dist 경로 등) 고정.
- **로컬 개발**: `trunk serve`로 로컬 웹서버 + 브라우저 핫리로드 확인. 네이티브는 기존대로 `cargo run`.

대안(비채택): 수동 `wasm-bindgen-cli`(투명하나 조각 관리 부담), `wasm-pack`(라이브러리/npm용).

---

## 3. 코드 변경 (cfg 분기 — 네이티브 동작 보존)

### 3.1 `AssetPlugin` 경로 (`src/main.rs`)
빌더 체인 중간을 `cfg`하기보다 **`file_path` 값 자체를 `cfg` 분기**해 `.set(...)`은 그대로 둔다.
```rust
fn asset_path() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    { concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string() } // 네이티브: 절대경로(IDE 실행 대응)
    #[cfg(target_arch = "wasm32")]
    { "assets".to_string() }                                       // wasm: 상대경로(HTTP 로딩)
}
// ...
.set(AssetPlugin { file_path: asset_path(), ..default() })
```

### 3.2 최고점수 경로 (`src/main.rs` `setup_high_score`)
```rust
#[cfg(not(target_arch = "wasm32"))]
let path = dirs::config_dir()
    .map(|d| d.join("bevy-asteroids"))
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join("highscore.json");
#[cfg(target_arch = "wasm32")]
let path = std::path::PathBuf::from("local/bevy-asteroids-highscore");
// 이후 .path(path) → 네이티브=Filesystem, wasm=LocalStorage{key:"bevy-asteroids-highscore"}
```
`dirs`는 wasm에서 불필요하므로 `[target.'cfg(not(target_family="wasm"))'.dependencies]`로 옮겨 네이티브 전용으로 격리하는 것도 고려(선택).

### 3.3 `getrandom` 웹 백엔드 (`Cargo.toml` + 빌드 설정)
`rand 0.10`이 쓰는 `getrandom`의 웹 백엔드를 활성화한다. `getrandom 0.3` 방식은 `wasm_js` feature + `--cfg getrandom_backend="wasm_js"`(또는 `.cargo/config.toml`의 `[target.wasm32-unknown-unknown] rustflags`), `getrandom 0.2` 방식은 `js` feature. **정확한 버전·설정은 구현 계획에서 `cargo build --target wasm32-unknown-unknown`/`trunk build` 에러 메시지로 확정**한다(에러가 필요한 feature/flag를 정확히 안내함). 네이티브 빌드엔 영향 없도록 wasm 타깃 한정으로 넣는다.

### 3.4 캔버스/반응형 (`src/main.rs` `Window`)
```rust
Window {
    title: "Bevy Asteroids".into(),
    canvas: Some("#bevy-canvas".into()),   // wasm: index.html의 캔버스에 부착. 네이티브에선 무시됨
    fit_canvas_to_parent: true,            // 페이지에 맞춤(0.19 필드명은 소스로 확인)
    resolution: (1280, 720).into(),        // 초기 크기
    ..default()
}
```
`canvas`/`fit_canvas_to_parent`는 네이티브에서 무시되므로 무조건 설정 가능(0.19 정확한 필드명은 구현 시 검증).

---

## 4. CI/CD — GitHub Actions → Pages

`.github/workflows/deploy.yml`:
- 트리거: `push` to `main`(+ 수동 `workflow_dispatch`).
- 단계: checkout → Rust toolchain + `wasm32-unknown-unknown` 타깃 → `trunk` 설치(바이너리 다운로드) → `trunk build --release --public-url /bevy-asteroids/` → `actions/upload-pages-artifact`(dist) → `actions/deploy-pages`.
- **`--public-url /bevy-asteroids/`**: GitHub Pages 프로젝트 사이트는 `https://crazyatom.github.io/bevy-asteroids/` 서브경로로 서빙되므로, wasm/JS/에셋 URL이 이 접두어로 해석되도록 지정.
- Pages 설정: 저장소 Settings에서 소스를 "GitHub Actions"로. 권한: 워크플로에 `pages: write`, `id-token: write`.
- **에셋 재생성**: CI의 `cargo build`(trunk 경유)가 `build.rs`를 돌려 gitignore된 PNG/WAV를 매번 생성 → trunk가 dist로 복사. 별도 커밋 불필요.

---

## 5. 바이너리 크기 최적화

- 릴리즈 프로필: wasm 크기를 위해 `opt-level = "s"`(또는 `"z"`), `lto = true`, `codegen-units = 1`, `panic = "abort"` 검토. 단 기존 `[profile.dev.package."*"] opt-level=3`(네이티브 개발 속도)은 유지.
- `trunk build --release`가 `wasm-opt`를 자동 적용(추가 축소).
- 목표: 과도한 최적화로 빌드가 크게 느려지지 않는 선에서 합리적 크기.

---

## 6. 제약 (범위 경계)

- **네이티브 보존**: 데스크톱 `cargo run`/`cargo build`/`cargo test` 동작·결과는 지금과 동일. 모든 웹 전용 코드는 `cfg(target_arch="wasm32")`(또는 `target_family="wasm"`) 뒤에.
- **데스크톱 브라우저 + 키보드 전용**: 기존 게임이 키보드 전용이므로 웹도 동일. 모바일/터치 조작은 범위 밖(별도 기능).
- **오디오**: 브라우저 자동재생 정책상 사용자 입력 후 재생. 타이틀에서 시작 키를 누르는 흐름이 이를 자연히 충족(추가 처리 불필요, 필요 시 계획에서 확인).
- **범위 밖**: 커스텀 도메인, 로딩 스피너/프로그레스 UI, 모바일 터치, WebGPU 강제(WebGL2 기본), 서버·리더보드.

---

## 7. 테스트/검증

- **순수 로직 테스트**: 기존대로 네이티브 `cargo test`(123개). 웹 배포는 로직을 바꾸지 않으므로 테스트 스위트 불변.
- **wasm 컴파일 검증**: `cargo build --target wasm32-unknown-unknown`(또는 `trunk build`)로 cfg 분기·의존성이 컴파일되는지 확인.
- **로컬 실행 검증**: `trunk serve`로 브라우저에서 실제 플레이(타이틀·게임플레이·최고점수 localStorage 유지·사운드) — 사람 확인.
- **배포 검증**: Actions 배포 성공 + 실제 Pages 링크에서 플레이·에셋 로딩·최고점수 유지 확인(사람).
- **네이티브 회귀**: `cargo run`으로 데스크톱 동작 불변 확인.

---

## 8. 파일 구조 (신규/수정)

| 파일 | 변경 | 책임 |
|---|---|---|
| `index.html` | 신규 | trunk 진입점: 캔버스 + rust/copy-dir 지시 + 최소 CSS |
| `Trunk.toml` | 신규(선택) | trunk 빌드 옵션 |
| `.github/workflows/deploy.yml` | 신규 | wasm 빌드 → Pages 자동 배포 |
| `.cargo/config.toml` | 신규(필요 시) | wasm 타깃 rustflags(getrandom 백엔드 등) |
| `Cargo.toml` | 수정 | wasm 타깃 전용 의존성(getrandom 등), 릴리즈 프로필 |
| `src/main.rs` | 수정 | AssetPlugin·HighScore 경로 cfg 분기, 캔버스 설정 |
| `.gitignore` | 수정(필요 시) | `/dist` 무시 |

---

## 9. 위험/알아둘 점

- **getrandom 설정**이 첫 wasm 빌드의 가장 흔한 장애물 — 에러 메시지가 정확한 feature/flag를 안내하므로 계획에서 그에 맞춰 확정.
- **`--public-url` 누락 시** 배포 사이트에서 wasm/에셋 404 — 서브경로 배포의 대표 실수.
- **캔버스 필드명**(`fit_canvas_to_parent` 등)은 0.19에서 이름이 다를 수 있어 소스 검증.
- **패닉 훅**: wasm에서 패닉 시 콘솔에 스택이 안 뜰 수 있어 `console_error_panic_hook` 도입 검토(선택, 디버깅 편의).
