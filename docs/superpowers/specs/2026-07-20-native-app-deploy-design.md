# 네이티브 앱 배포 설계 (macOS + Windows)

> 데스크톱 바이너리를 GitHub Releases로 자동 배포한다. 웹(WASM) 배포·네이티브 개발(`cargo run`) 동작은 불변.

## 목표

`bevy-asteroids`를 데스크톱에서 다운로드·실행 가능하게 배포한다.

- **타깃**: macOS(`.app` 번들, arm64) + Windows(zip, x64)
- **서명**: 미서명(Gatekeeper/SmartScreen 우회를 README에 안내). CI는 나중에 secrets만 넣으면 서명 추가 가능하도록 설계.
- **자동화**: GitHub Actions — 태그 `v*` 푸시 시 3-잡(test→build×2) → Releases 업로드.
- **앱 아이콘**: 게임 카툰 스타일의 새 전용 아이콘.

## 비범위 (후속 과제)

- 코드 서명·공증(Apple Developer / Windows Authenticode)
- Linux 배포, macOS x64/유니버설 바이너리
- 설치 프로그램(dmg/msi/NSIS), 자동 업데이트

## 전역 제약

- **기존 동작 불변**: 웹 배포(`deploy.yml`), 네이티브 개발 실행(`cargo run` → 소스 트리 `assets/` 사용)은 그대로.
- **에셋은 `build.rs` 생성물**(PNG/WAV/음악, gitignore) — CI 빌드 시 생성되며 배포 아티팩트에 함께 포장.
- **공급망 최소화**: 릴리스 업로드는 러너 내장 `gh` CLI 사용(3rd-party 액션 지양).

---

## 설계

### 1. 에셋 경로 런타임 해결 (`src/main.rs`)

현재 `asset_path()`는 `CARGO_MANIFEST_DIR/assets`(개발 PC 절대경로)를 컴파일 타임에 박아 배포 시 깨진다. 실행 위치 기준으로 해결하되 개발 실행은 보존한다.

```rust
#[cfg(not(target_arch = "wasm32"))]
fn asset_path() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // macOS .app: Contents/MacOS/<bin> → ../Resources/assets
            let bundle = dir.join("../Resources/assets");
            if bundle.is_dir() { return bundle.to_string_lossy().into_owned(); }
            // 일반 zip: 실행파일 옆 assets/
            let sibling = dir.join("assets");
            if sibling.is_dir() { return sibling.to_string_lossy().into_owned(); }
        }
    }
    // 개발(cargo run) fallback: 소스 트리 assets (기존 동작 보존)
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string()
}
```

- **우선순위**: `.app` Resources → exe 옆 → 소스 트리. `is_dir()` 존재 검사로 분기해 같은 바이너리가 번들/zip/개발 어디서든 맞는 경로를 쓴다.
- **wasm 분기 불변**(`"assets"` 상대경로).

### 2. 앱 아이콘

게임 카툰 스타일(우주선 + 소행성, 어두운 라운드스퀘어 우주 배경)의 새 전용 아이콘. "SVG 원본 커밋, 래스터는 파생" 관례를 따른다.

- **소스**: `assets/icon/app-icon.svg` (커밋). 구현 시 제작 후 육안 검토·조정.
- **macOS**: CI(mac 러너)에서 SVG → 다중 해상도 PNG(16·32·64·128·256·512 + @2x) → `iconutil -c icns` → `AppIcon.icns`를 `Contents/Resources/`에 배치, Info.plist `CFBundleIconFile=AppIcon`. 래스터화는 `librsvg`(`brew install librsvg`의 `rsvg-convert`) 또는 `sips`.
- **Windows**: exe에 아이콘을 **임베드**해야 표시됨 → `build.rs`(Windows 타깃 한정)에서 `winresource`로 `.ico` 임베드.
  - `.ico`는 **사전 생성 커밋**(`assets/icon/app-icon.ico`) — Windows 러너에 SVG→ICO 변환기가 없어 CI 실패 지점을 안 늘리기 위함. SVG가 진짜 원본, `.ico`는 파생 산출물.

**새 의존성**: `winresource`(`[target.'cfg(windows)'.build-dependencies]`로 한정, 버전 핀).

### 3. 패키징

**macOS `.app` 번들** (CI 스크립트로 조립 — 번들은 특정 구조의 디렉터리라 전용 툴 불필요):

```text
BevyAsteroids.app/
  Contents/
    Info.plist
    MacOS/bevy-asteroids       # 릴리스 바이너리(aarch64-apple-darwin)
    Resources/
      AppIcon.icns
      assets/                  # sprites/ · sounds/ · music/
→ zip → BevyAsteroids-macos-arm64.zip
```

Info.plist 필드: `CFBundleName`, `CFBundleDisplayName`, `CFBundleIdentifier`(`io.github.crazyatom.bevy-asteroids`), `CFBundleExecutable`(bevy-asteroids), `CFBundlePackageType`(APPL), `CFBundleShortVersionString`(태그에서), `CFBundleVersion`, `CFBundleIconFile`(AppIcon), `LSMinimumSystemVersion`, `NSHighResolutionCapable`(true).

**Windows** (폴더 → zip):

```text
bevy-asteroids-windows-x64/
  bevy-asteroids.exe           # 아이콘 임베드됨
  assets/                      # sprites/ · sounds/ · music/
→ zip → bevy-asteroids-windows-x64.zip
```

- `assets/`는 런타임 에셋(sprites `*.png` · sounds `*.wav` · music `*.wav`)만 포함. SVG 원본(`sprites/src`)은 배포 불필요.

### 4. GitHub Actions (`.github/workflows/release.yml`, `deploy.yml`과 별개)

- **트리거**: `push: tags: ['v*']` + `workflow_dispatch`
- **권한**: `contents: write`(Releases 생성/업로드)
- **잡 구성**(순차 의존으로 Release 생성 레이스 제거):
  1. `test` (ubuntu-latest): `cargo test` — 깨진 릴리스 방지 게이트.
  2. `create-release` (`needs: test`, ubuntu-latest): 태그에 대한 Release를 **1회** 생성(`gh release create <tag> --generate-notes` 또는 빈 Release). 이미 있으면 스킵(idempotent).
  3. `build-macos` (`needs: create-release`, macos-latest = Apple Silicon/arm64): `cargo build --release`(build.rs가 에셋 생성) → `.icns` 생성 → `.app` 조립 + assets/icns 복사 → zip → `gh release upload <tag>`.
  4. `build-windows` (`needs: create-release`, windows-latest): `cargo build --release`(build.rs가 에셋 생성 + 아이콘 임베드) → `exe`+`assets` 폴더 → zip → `gh release upload <tag>`.
- 빌드 두 잡은 `create-release` 완료 후 병렬 실행되며, 각자 자기 zip만 업로드하므로 Release 자체를 동시에 만들지 않는다.

### 5. README 다운로드/설치 섹션

- **다운로드**: Releases 링크
- **macOS**(미서명): 압축 해제 → `.app`을 응용 프로그램으로 → 첫 실행 **우클릭 → 열기**(또는 `xattr -dr com.apple.quarantine BevyAsteroids.app`)
- **Windows**(미서명): 압축 해제 → `.exe` 실행 → SmartScreen 뜨면 **추가 정보 → 실행**

---

## 파일 변경 목록

| 파일 | 변경 |
|------|------|
| `src/main.rs` | `asset_path()` 런타임 해결(접근 A) |
| `build.rs` | Windows 한정 `winresource` 아이콘 임베드 분기 |
| `Cargo.toml` | `winresource` build-dep(cfg windows), (선택) 번들 메타 |
| `assets/icon/app-icon.svg` | **신규**(아이콘 원본) |
| `assets/icon/app-icon.ico` | **신규**(Windows 임베드용 사전 생성) |
| `.github/workflows/release.yml` | **신규**(릴리스 워크플로) |
| `README.md` | 다운로드/설치 섹션 |
| `.gitignore` | CI 생성 아이콘 파생물(icns/iconset)·기존 assets 규칙 확인 |

## 검증

- **로컬(개발자)**: `cargo build --release` 후 `.app` 조립 스크립트를 로컬 실행 → 바이너리를 **소스 트리 밖 다른 위치**로 옮겨 실행해 에셋이 정상 로딩되는지 확인(경로 blocker 수정 검증). Windows 임베드 아이콘은 크로스 검증 어려워 CI 산출물로 확인.
- **CI**: 테스트 태그(`v0.0.0-test` 등) 푸시 → 두 zip이 Release에 생성되는지 확인.
- **최종(사람)**: mac/Windows에서 각 zip 다운로드 → 압축 해제 → 실행 → 게임·사운드 정상 동작 육안/청취 확인.

## 위험

- **에셋 경로 fallback 오작동** → 배포본이 개발 경로 참조. 완화: `is_dir()` 우선순위(번들→옆→소스).
- **Windows 아이콘 임베드 실패로 빌드 붕괴**. 완화: `cfg(windows)` 한정, 로컬/CI 검증.
- **`macos-latest` 러너 아키텍처 변동**(현재 arm64). x64 사용자 미지원은 비범위로 명시.
- **`winresource` 공급망**. 완화: 널리 쓰이는 크레이트, 버전 핀, Windows 빌드에서만 사용.
- **Release 동시 생성 레이스**(두 빌드 잡). 완화: 생성 지점 단일화.

## 자기검토

- **스펙 커버리지**: 경로(§1)·아이콘(§2)·패키징(§3)·CI(§4)·README(§5) 모두 태스크화 가능. ✓
- **기존 동작 보존**: `cargo run`(소스 fallback)·웹 배포(wasm 분기·deploy.yml) 불변 명시. ✓
- **YAGNI**: 서명·Linux·유니버설·설치프로그램은 비범위로 분리. ✓
