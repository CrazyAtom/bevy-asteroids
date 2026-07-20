# 네이티브 앱 배포 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** macOS(`.app`, arm64)·Windows(zip, x64) 데스크톱 바이너리를 GitHub Actions로 빌드해 GitHub Releases에 자동 배포한다(미서명, 앱 아이콘 포함).

**Architecture:** 런타임 `current_exe()` 기준 에셋 경로 해결로 배포본이 어디서든 에셋을 찾게 하고, 태그 `v*` 푸시 시 `release.yml`이 test→create-release→build(mac/win) 잡으로 패키징·업로드한다. macOS 번들 조립은 재사용 가능한 셸 스크립트로 분리한다.

**Tech Stack:** Rust + Bevy 0.19, `winresource`(Windows 아이콘 임베드, build-dep), GitHub Actions, `iconutil`/`librsvg`(macOS 아이콘), `gh` CLI(Releases).

## Global Constraints

- 기존 동작 불변: 웹 배포(`deploy.yml`), 네이티브 개발(`cargo run` → 소스 트리 `assets/`), `cargo test`/`clippy -D warnings` 통과.
- 에셋은 `build.rs` 생성물 유지(gitignore). 배포 아티팩트에 런타임 에셋(`sprites/*.png`·`sounds/*.wav`·`music/*.wav`) 포함.
- 미서명 배포. 릴리스 업로드는 러너 내장 `gh` CLI만 사용(3rd-party 액션 금지).
- 타깃: macOS arm64(`.app`), Windows x64(zip). Linux·서명·유니버설·설치프로그램은 비범위.
- 커밋 메시지 한국어, conventional prefix(이 레포 관례), `Co-Authored-By` 미포함.

---

### Task 1: 에셋 경로 런타임 해결 (`src/main.rs`)

**Files:**
- Modify: `src/main.rs` (`asset_path()` 교체 + `resolve_asset_dir()` 신설 + 테스트 모듈)

**Interfaces:**
- Consumes: 없음
- Produces: `fn resolve_asset_dir(exe_dir: &std::path::Path) -> Option<std::path::PathBuf>` (순수·테스트 대상), `fn asset_path() -> String`(기존 시그니처 유지, AssetPlugin이 소비)

- [ ] **Step 1: 실패 테스트 작성** — `src/main.rs` 끝에 추가

```rust
#[cfg(all(test, not(target_arch = "wasm32")))]
mod asset_path_tests {
    use super::resolve_asset_dir;
    use std::fs;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("bevy_ast_assettest_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn prefers_macos_bundle_resources() {
        let root = temp_root("bundle");
        let macos = root.join("Contents/MacOS");
        let res_assets = root.join("Contents/Resources/assets");
        fs::create_dir_all(&macos).unwrap();
        fs::create_dir_all(&res_assets).unwrap();
        let got = resolve_asset_dir(&macos).expect("번들 Resources/assets를 찾아야 함");
        assert_eq!(got.canonicalize().unwrap(), res_assets.canonicalize().unwrap());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn falls_back_to_sibling_assets() {
        let root = temp_root("sibling");
        let exe_dir = root.join("bin");
        let sibling = exe_dir.join("assets");
        fs::create_dir_all(&sibling).unwrap();
        let got = resolve_asset_dir(&exe_dir).expect("실행파일 옆 assets를 찾아야 함");
        assert_eq!(got.canonicalize().unwrap(), sibling.canonicalize().unwrap());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn none_when_no_assets_present() {
        let root = temp_root("none");
        let exe_dir = root.join("bin");
        fs::create_dir_all(&exe_dir).unwrap();
        assert!(resolve_asset_dir(&exe_dir).is_none());
        fs::remove_dir_all(&root).ok();
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `cargo test --bin bevy-asteroids asset_path_tests`
Expected: FAIL — `cannot find function 'resolve_asset_dir'`

- [ ] **Step 3: 구현** — `src/main.rs`의 기존 `asset_path()`(현재 76–86행)를 아래로 교체

```rust
/// 실행파일 디렉터리 기준으로 배포 에셋 위치를 찾는다(순수 파일시스템 검사).
/// macOS `.app`(../Resources/assets) → 실행파일 옆(assets) 순으로 우선.
#[cfg(not(target_arch = "wasm32"))]
fn resolve_asset_dir(exe_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let bundle = exe_dir.join("../Resources/assets");
    if bundle.is_dir() {
        return Some(bundle);
    }
    let sibling = exe_dir.join("assets");
    if sibling.is_dir() {
        return Some(sibling);
    }
    None
}

/// 에셋 루트 경로. 배포본은 실행 위치 기준, 개발(cargo run)은 소스 트리 fallback,
/// wasm은 상대경로(HTTP 로딩).
fn asset_path() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .and_then(|dir| resolve_asset_dir(&dir))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string())
    }
    #[cfg(target_arch = "wasm32")]
    {
        "assets".to_string()
    }
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `cargo test --bin bevy-asteroids asset_path_tests`
Expected: PASS (3 tests)

- [ ] **Step 5: 회귀 확인 (개발 실행 불변)**

Run: `cargo run` (짧게 실행 후 종료 — startup 패닉 없이 타이틀/에셋 로딩되는지 육안)
Expected: 소스 트리 `assets/`로 정상 로딩(현재 동작과 동일). `cargo clippy --all-targets -- -D warnings` 경고 0.

- [ ] **Step 6: 커밋**

```bash
git add src/main.rs
git commit -m "feat: 네이티브 에셋 경로를 실행 위치 기준으로 해결"
```

---

### Task 2: 앱 아이콘 에셋 (`assets/icon/app-icon.svg` + `.ico`)

**Files:**
- Create: `assets/icon/app-icon.svg` (커밋, 진짜 원본)
- Create: `assets/icon/app-icon.ico` (커밋, SVG에서 사전 생성한 파생 산출물 — Windows 임베드용)

**Interfaces:**
- Produces: 아이콘 원본 SVG(Task 4 macOS `.icns` 생성 입력), `.ico`(Task 3 Windows 임베드 입력)

- [ ] **Step 1: 아이콘 SVG 작성** — `assets/icon/app-icon.svg`

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#1b2240"/>
      <stop offset="1" stop-color="#0a0d1c"/>
    </linearGradient>
  </defs>
  <!-- 라운드스퀘어 우주 배경 -->
  <rect x="0" y="0" width="1024" height="1024" rx="220" fill="url(#bg)"/>
  <!-- 별 -->
  <g fill="#cfd8ff">
    <circle cx="220" cy="230" r="9"/><circle cx="800" cy="200" r="7"/>
    <circle cx="860" cy="470" r="6"/><circle cx="180" cy="640" r="7"/>
    <circle cx="330" cy="820" r="6"/><circle cx="720" cy="800" r="8"/>
  </g>
  <!-- 소행성 -->
  <path d="M250 720 l50 -30 55 20 20 55 -35 45 -60 5 -40 -45z"
        fill="#7c6f63" stroke="#2a2320" stroke-width="14" stroke-linejoin="round"/>
  <!-- 플레이어 우주선(위 방향, 카툰: 플랫 채색 + 두꺼운 외곽선 + 하이라이트) -->
  <g transform="translate(512 470) rotate(0)">
    <path d="M0 -190 L120 150 L0 90 L-120 150 Z"
          fill="#4ea1ff" stroke="#10233f" stroke-width="26" stroke-linejoin="round"/>
    <path d="M0 -190 L44 60 L0 30 L-44 60 Z" fill="#bfe0ff"/>
    <!-- 추진 화염 -->
    <path d="M-46 120 L0 250 L46 120 L0 165 Z" fill="#ffb43b" stroke="#c8460f" stroke-width="14" stroke-linejoin="round"/>
  </g>
</svg>
```

- [ ] **Step 2: SVG 육안 검토** — 렌더해서 확인(레포의 `resvg` 대신 시스템 도구 사용)

Run(mac): `qlmanage -t -s 512 -o /tmp assets/icon/app-icon.svg 2>/dev/null; open /tmp/app-icon.svg.png` (또는 브라우저로 SVG 열기)
Expected: 어두운 라운드스퀘어에 파란 우주선 + 화염 + 소행성 + 별이 보이고, 512px에서 형태가 뚜렷. 마음에 안 들면 SVG 수정 후 재확인(반복).

- [ ] **Step 3: `.ico` 생성** — ImageMagick으로 멀티사이즈 아이콘 생성

```bash
# 도구 없으면: brew install imagemagick
magick assets/icon/app-icon.svg -background none \
  -define icon:auto-resize=16,32,48,64,128,256 assets/icon/app-icon.ico
```
Expected: `assets/icon/app-icon.ico` 생성(수 KB~수십 KB). `file assets/icon/app-icon.ico` → "MS Windows icon resource".
(ImageMagick 미가용 시 대안: `brew install icoutils librsvg` 후 `rsvg-convert`로 각 크기 PNG 생성 → `icotool -c -o app-icon.ico *.png`.)

- [ ] **Step 4: `.gitignore` 확인** — `assets/icon/`가 기존 `assets` 무시 규칙에 안 걸리는지 확인

Run: `git check-ignore assets/icon/app-icon.svg assets/icon/app-icon.ico || echo "추적 가능"`
Expected: "추적 가능"(무시되지 않음). 만약 무시되면 `.gitignore`에 `!assets/icon/` 예외 추가.

- [ ] **Step 5: 커밋**

```bash
git add assets/icon/app-icon.svg assets/icon/app-icon.ico
git commit -m "feat: 앱 아이콘(우주선) SVG + Windows용 ICO 추가"
```

---

### Task 3: Windows 아이콘 임베드 (`Cargo.toml` + `build.rs`)

**Files:**
- Modify: `Cargo.toml` (Windows 한정 build-dependency)
- Modify: `build.rs` (Windows 한정 아이콘 임베드 분기 — `main()` 최상단)

**Interfaces:**
- Consumes: `assets/icon/app-icon.ico` (Task 2)
- Produces: Windows exe에 임베드된 아이콘(런타임 코드 영향 없음)

- [ ] **Step 1: Cargo.toml에 build-dependency 추가** — `[build-dependencies]` 블록 아래에 추가

```toml
# Windows exe에 앱 아이콘을 임베드(build.rs). Windows 타깃 빌드에서만 사용.
[target.'cfg(windows)'.build-dependencies]
winresource = "0.1"
```

- [ ] **Step 2: build.rs에 임베드 분기 추가** — `build.rs`의 `fn main()` **첫 줄**에 삽입(기존 에셋 생성 로직 앞)

```rust
    // Windows: exe에 앱 아이콘 리소스를 임베드(탐색기·작업표시줄 표시용).
    // 호스트가 Windows일 때만(=windows 러너 빌드) 실행되고, 그 외 타깃엔 무영향.
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon/app-icon.ico");
        res.compile().expect("Windows 아이콘 리소스 컴파일 실패");
    }
```

- [ ] **Step 3: 비-Windows 빌드 회귀 확인 (개발기 mac)**

Run: `cargo build && cargo test`
Expected: 성공. `cfg(windows)`가 false라 winresource는 컴파일/사용되지 않음(빌드 시간·바이너리 불변). `cargo clippy --all-targets -- -D warnings` 경고 0.

- [ ] **Step 4: 커밋**

```bash
git add Cargo.toml Cargo.lock build.rs
git commit -m "feat: Windows exe에 앱 아이콘 임베드(winresource)"
```

> Windows 임베드 실제 표시는 개발기(mac)에서 검증 불가 — Task 5 CI 산출물(Windows zip)로 확인한다.

---

### Task 4: macOS `.app` 패키징 스크립트 (`scripts/package-macos.sh`)

**Files:**
- Create: `scripts/package-macos.sh` (CI·로컬 공용)

**Interfaces:**
- Consumes: 릴리스 바이너리(`target/release/bevy-asteroids`), 생성된 `assets/`, `assets/icon/app-icon.svg` (Task 1·2), 버전 문자열(인자)
- Produces: `dist-native/BevyAsteroids.app`, `dist-native/BevyAsteroids-macos-arm64.zip`

- [ ] **Step 1: 스크립트 작성** — `scripts/package-macos.sh`

```bash
#!/usr/bin/env bash
# macOS .app 번들 조립 + zip. 사용법: scripts/package-macos.sh <version>
# 요구 도구: iconutil(내장), rsvg-convert(brew install librsvg).
set -euo pipefail

VERSION="${1:-0.0.0}"
BIN="target/release/bevy-asteroids"
OUT="dist-native"
APP="$OUT/BevyAsteroids.app"
RES="$APP/Contents/Resources"
MACOS="$APP/Contents/MacOS"

[ -f "$BIN" ] || { echo "바이너리 없음: $BIN (먼저 cargo build --release)"; exit 1; }
command -v rsvg-convert >/dev/null || { echo "rsvg-convert 필요: brew install librsvg"; exit 1; }

rm -rf "$OUT"
mkdir -p "$MACOS" "$RES/assets"

# 1) 바이너리
cp "$BIN" "$MACOS/bevy-asteroids"
chmod +x "$MACOS/bevy-asteroids"

# 2) 런타임 에셋(sprites/sounds/music) — SVG 원본(sprites/src)은 제외
cp -R assets/sprites "$RES/assets/sprites"
rm -rf "$RES/assets/sprites/src"
cp -R assets/sounds "$RES/assets/sounds"
cp -R assets/music "$RES/assets/music"

# 3) 아이콘: SVG → iconset PNG들 → .icns
ICONSET="$OUT/AppIcon.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  rsvg-convert -w "$s"   -h "$s"   assets/icon/app-icon.svg -o "$ICONSET/icon_${s}x${s}.png"
  d=$((s*2))
  rsvg-convert -w "$d"   -h "$d"   assets/icon/app-icon.svg -o "$ICONSET/icon_${s}x${s}@2x.png"
done
iconutil -c icns "$ICONSET" -o "$RES/AppIcon.icns"
rm -rf "$ICONSET"

# 4) Info.plist
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Bevy Asteroids</string>
  <key>CFBundleDisplayName</key><string>Bevy Asteroids</string>
  <key>CFBundleIdentifier</key><string>io.github.crazyatom.bevy-asteroids</string>
  <key>CFBundleExecutable</key><string>bevy-asteroids</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# 5) zip
( cd "$OUT" && ditto -c -k --keepParent "BevyAsteroids.app" "BevyAsteroids-macos-arm64.zip" )
echo "완료: $OUT/BevyAsteroids-macos-arm64.zip"
```

- [ ] **Step 2: 실행 권한 부여**

Run: `chmod +x scripts/package-macos.sh`

- [ ] **Step 3: 로컬 빌드 + 패키징**

Run: `cargo build --release && scripts/package-macos.sh 0.0.0-local`
Expected: `dist-native/BevyAsteroids.app` + `.zip` 생성. (librsvg 없으면 안내대로 `brew install librsvg` 후 재실행.)

- [ ] **Step 4: 배포본 에셋 로딩 검증 (blocker 수정 실증)**

Run: `open dist-native/BevyAsteroids.app`
Expected: 앱이 실행되고 **스프라이트·사운드·음악이 정상 로딩**(소스 트리 경로가 아니라 번들 `Resources/assets`를 씀). 아이콘이 Dock/Finder에 표시. 첫 실행은 미서명이라 **우클릭 → 열기** 필요할 수 있음.

- [ ] **Step 5: `.gitignore`에 산출물 추가** — `dist-native/`가 커밋되지 않도록

`.gitignore`에 한 줄 추가:
```
/dist-native/
```

- [ ] **Step 6: 커밋**

```bash
git add scripts/package-macos.sh .gitignore
git commit -m "feat: macOS .app 번들 패키징 스크립트"
```

---

### Task 5: 릴리스 워크플로 (`.github/workflows/release.yml`)

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `scripts/package-macos.sh` (Task 4), 태그 `v*`
- Produces: 태그별 GitHub Release + `BevyAsteroids-macos-arm64.zip`·`bevy-asteroids-windows-x64.zip`

- [ ] **Step 1: 워크플로 작성** — `.github/workflows/release.yml`

```yaml
name: Release native builds

on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      tag:
        description: '릴리스 태그(예: v0.1.0) — 이미 존재해야 함'
        required: true

permissions:
  contents: write

env:
  TAG: ${{ github.event.inputs.tag || github.ref_name }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test

  create-release:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Release 생성(없으면)
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh release view "$TAG" --repo "$GITHUB_REPOSITORY" \
            || gh release create "$TAG" --repo "$GITHUB_REPOSITORY" \
                 --title "$TAG" --generate-notes

  build-macos:
    needs: create-release
    runs-on: macos-latest   # Apple Silicon(arm64)
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: librsvg 설치(아이콘 래스터화)
        run: brew install librsvg
      - name: 빌드(build.rs가 에셋 생성)
        run: cargo build --release
      - name: .app 패키징
        run: |
          chmod +x scripts/package-macos.sh
          scripts/package-macos.sh "${TAG#v}"
      - name: Release 업로드
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh release upload "$TAG" dist-native/BevyAsteroids-macos-arm64.zip --clobber --repo "$GITHUB_REPOSITORY"

  build-windows:
    needs: create-release
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: 빌드(build.rs가 에셋 생성 + 아이콘 임베드)
        run: cargo build --release
      - name: 패키징(zip)
        shell: pwsh
        run: |
          $dir = "bevy-asteroids-windows-x64"
          New-Item -ItemType Directory -Force -Path $dir | Out-Null
          Copy-Item target\release\bevy-asteroids.exe "$dir\"
          Copy-Item -Recurse assets\sprites "$dir\assets\sprites"
          Remove-Item -Recurse -Force "$dir\assets\sprites\src" -ErrorAction SilentlyContinue
          Copy-Item -Recurse assets\sounds "$dir\assets\sounds"
          Copy-Item -Recurse assets\music  "$dir\assets\music"
          Compress-Archive -Path "$dir\*" -DestinationPath "$dir.zip" -Force
      - name: Release 업로드
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: gh release upload "$env:TAG" bevy-asteroids-windows-x64.zip --clobber --repo "$env:GITHUB_REPOSITORY"
```

- [ ] **Step 2: YAML 유효성 확인**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('OK')"`
Expected: `OK`

- [ ] **Step 3: 커밋 + 태그 드라이런**

```bash
git add .github/workflows/release.yml
git commit -m "ci: 네이티브 릴리스 워크플로(mac .app + win zip)"
```

- [ ] **Step 4: 실제 릴리스 검증 (사용자와 함께)**

`main` 병합 후 테스트 태그로 워크플로를 돌린다:
```bash
git tag v0.1.0 && git push origin v0.1.0     # 또는 Actions에서 workflow_dispatch
```
Expected: Actions에서 test→create-release→build-macos·build-windows 성공, Release에 zip 2개 첨부. 각 zip 다운로드 → 압축 해제 → 실행 → 게임·사운드·아이콘 정상(사람 검증, 미서명 우회 안내대로).

---

### Task 6: README 다운로드/설치 섹션

**Files:**
- Modify: `README.md` (웹 플레이 안내 뒤 "다운로드" 섹션 추가)

- [ ] **Step 1: 섹션 추가** — `README.md`의 웹 플레이 인용구(`> 🎮 웹에서 바로 플레이 ...`) 바로 아래에 삽입

```markdown

## ⬇️ 다운로드 (데스크톱)

[Releases](https://github.com/CrazyAtom/bevy-asteroids/releases)에서 OS별 빌드를 받으세요. **미서명 빌드**라 첫 실행 시 아래 안내가 필요합니다.

- **macOS (Apple Silicon)**: `BevyAsteroids-macos-arm64.zip` 압축 해제 → `BevyAsteroids.app`을 응용 프로그램으로 이동 → **우클릭 → 열기**(최초 1회, Gatekeeper). 안 열리면 터미널에서 `xattr -dr com.apple.quarantine BevyAsteroids.app`.
- **Windows (x64)**: `bevy-asteroids-windows-x64.zip` 압축 해제 → `bevy-asteroids.exe` 실행 → SmartScreen 경고 시 **추가 정보 → 실행**.
```

- [ ] **Step 2: 렌더 확인**

Run: (IDE 마크다운 미리보기 또는) `python3 -c "print(open('README.md').read()[:400])"`
Expected: 링크·목록 문법 정상, lint 경고 0(MD040/MD060 등 없음).

- [ ] **Step 3: 커밋**

```bash
git add README.md
git commit -m "docs: 데스크톱 다운로드/설치 안내 추가"
```

---

## Self-Review

- **스펙 커버리지**: §1 경로→T1, §2 아이콘→T2(+T3 Win 임베드, T4 mac icns), §3 패키징→T4(mac)·T5(win), §4 CI→T5, §5 README→T6. 누락 없음. ✓
- **타입 일관성**: `resolve_asset_dir(&Path)->Option<PathBuf>`·`asset_path()->String`이 T1 정의와 소비처(AssetPlugin) 일치. 스크립트 산출물명(`BevyAsteroids-macos-arm64.zip`·`bevy-asteroids-windows-x64.zip`)이 T4·T5 업로드와 일치. ✓
- **플레이스홀더 스캔**: 모든 코드/스크립트/YAML/SVG 실제 내용 포함, TBD 없음. ✓
- **위험**: Windows 아이콘 임베드·CI는 개발기 검증 불가 → T5 실제 태그 드라이런으로 확인(사람). `.ico` 생성 도구(ImageMagick) 부재 시 대안(icoutils) 명시. ✓
```
