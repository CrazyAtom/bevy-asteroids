# 배경 음악(BGM) 설계

**작성일:** 2026-07-17
**상태:** 설계 승인됨 → 계획 단계로

## 목표(Goal)

절차적으로 생성한 **칩튠 배경 음악**을 추가한다. 타이틀 1곡 + 게임플레이 테마별 6곡(총 7곡), 심리스 루프. **음소거 토글**(M 키 + 메뉴 항목)을 제공하고 설정은 세션 간 지속한다. 기존 효과음(SFX)과 게임 로직은 변하지 않는다.

## 아키텍처(Architecture)

세 부분으로 나뉜다. (1) `build.rs`가 절차적으로 7개 루프 WAV를 생성, (2) 신규 `fx/music.rs`가 상태·테마에 맞춰 트랙을 재생/교체, (3) 지속되는 `MusicEnabled` 설정이 M 키·메뉴로 토글된다. 게임의 모든 에셋이 build.rs 절차 생성이라는 기존 방침을 그대로 따른다.

## 전역 제약(Global Constraints)

- **엔진:** Bevy 0.19, 내장 `bevy_audio`(`AudioPlayer` + `PlaybackSettings`). 지속은 `bevy-persistent`(최고점수와 동일 패턴).
- **음원:** 100% 절차 생성(`build.rs`). 외부 바이너리 음원·라이선스 없음.
- **포맷:** 16bit PCM WAV, **22050 Hz 모노**(기존 `SR` 상수 재사용). `assets/music/*.wav`.
- **플랫폼 패리티:** 네이티브·wasm 동일 동작. 지속 경로는 `#[cfg]` 분기(네이티브=`dirs::config_dir()` 파일, wasm=`local/...` localStorage) — `load_high_score` 패턴 재사용.
- **온디맨드 로드:** 현재 필요한 트랙만 `asset_server.load` — 7곡을 부팅 시 전부 받지 않는다(wasm 초기 다운로드 최소화).
- **불변식:** SFX 재생·게임플레이 로직·기존 테스트는 영향 없음. 음소거는 **BGM에만** 적용, SFX는 항상 재생.

## 1. 음악 생성 — `build.rs` 확장

기존 `synth(dur, freq_fn, amp, noise)`(단일 톤)을 **다성 칩튠 시퀀서**로 확장한다. SFX 생성 코드는 그대로 두고 음악 생성 함수를 추가한다.

### 음악 이론 원시요소
- **음정→주파수:** `note_hz(semitone: i32) -> f32 = BASE_HZ * 2f32.powf(semitone as f32 / 12.0)`. 반음 오프셋 기준. 스케일은 반음 오프셋 배열로 표현(예: 장조 `[0,2,4,5,7,9,11]`, 단조 `[0,2,3,5,7,8,10]`).
- **템포:** 트랙별 BPM → `beat_secs = 60.0 / bpm`. 노트는 비트 그리드에 배치.
- **파형:**
  - `pulse(phase, duty)` — 리드·아르페지오(칩튠 특유 사각/펄스파, duty로 음색 변화).
  - `triangle(phase)` — 베이스(둥근 저음).
- **엔벨로프:** 노트마다 빠른 어택 + 감쇠로 **경계에서 0에 수렴**(클릭·루프 이음새 잡음 방지). 예: 선형 AD, release 구간이 다음 노트 전에 0.

### 트랙 구조
한 트랙 = 3개 보이스를 마디 단위로 믹싱한 심리스 루프.
- **Bass:** 저옥타브 삼각파, 코드 루트를 비트마다.
- **Arpeggio:** 중음역 펄스파, 코드 구성음을 빠르게 분산.
- **Lead:** 멜로디(펄스파), 테마별 짧은 프레이즈. (성긴 테마는 생략 가능.)
각 보이스를 샘플 버퍼에 렌더 → 합산 → i16 범위로 정규화(클리핑 방지). **총 길이 = 정수 마디**라 루프 끝→처음이 자연스럽다.

### 7곡 무드 파라미터(하드코딩 패턴)
스케일·BPM·파형·패턴을 달리해 확연히 구분. (구체 노트 배열은 계획 단계에서 확정.)

| 트랙 | 파일 | 스케일 | BPM(대략) | 무드 |
|---|---|---|---|---|
| Title | `title.wav` | 장조 | 100 | 은은한 어트랙트, 성긴 아르페지오 |
| AsteroidBelt | `belt.wav` | 장조 | 120 | 경쾌·구동감 |
| AlienFleet | `fleet.wav` | 단조 | 130 | 긴장·행진 |
| SolarFlare | `flare.wav` | 장조(빠름) | 150 | 격렬·고에너지 |
| FrozenField | `ice.wav` | 단조 | 80 | 차갑고 성김(베이스 최소, 넓은 음정) |
| EmStorm | `storm.wav` | 반음계/불안정 | 140 | 불안정·디소넌트 |
| BlackHole | `void.wav` | 단조 | 70 | 어둡고 느림, 깊은 저음 드론 |

**길이:** 트랙당 약 12~16초 루프(모노 22kHz면 ≈0.5~0.7MB). 온디맨드라 실제 다운로드는 타이틀곡 + 현재 테마곡뿐.

## 2. 재생 — 신규 `fx/music.rs`

`MusicPlugin`이 상태·테마에 맞춰 올바른 루프 트랙을 재생한다.

### 트랙 식별·에셋
- `enum Track { Title, Belt, Fleet, Flare, Ice, Storm, Void }` + `Track::file() -> &'static str`(경로).
- `Track::for_theme(ThemeId) -> Track` — 테마→트랙 매핑.

### 상태
- 리소스 `CurrentMusic { track: Option<Track>, entity: Option<Entity> }` — 현재 재생 중인 트랙과 오디오 엔티티.

### 시스템
- `desired_track(state, theme) -> Track`: `Title`→`Title`; **그 외 모든 상태**(Playing/Paused/GameOver/Restarting)→`for_theme(current_theme)`. 즉 게임오버 화면에서도 직전 테마곡이 계속 흐르다가, 타이틀 복귀 시에만 타이틀곡으로 전환된다(결정적·상태 하나당 트랙 하나).
- `sync_music`(Update): 원하는 트랙과 `CurrentMusic.track`이 다르면 **하드 컷 교체** — 기존 엔티티 despawn 후 `AudioPlayer(load(track.file()))` + `PlaybackSettings::LOOP`(음량은 `MusicEnabled`) 스폰, `CurrentMusic` 갱신. 테마 전환은 스테이지 배너 타이밍이라 하드 컷이 자연스럽다.
- 음소거 반영: `MusicEnabled`가 false면 재생하지 않는다(엔티티 없음). true로 바뀌면 현재 트랙을 (재)시작.

### 루프
`PlaybackSettings::LOOP`로 반복. 심리스 루프는 생성 측(정수 마디 + 엔벨로프)이 보장.

## 3. 음소거 — 지속 설정

### 상태
- `#[derive(Resource, Serialize, Deserialize)] struct MusicEnabled(pub bool)` — 기본 `true`.
- `Persistent<MusicEnabled>`로 지속. 경로는 `load_high_score`와 동일 `#[cfg]` 분기(네이티브 설정파일 / wasm localStorage). 빌드 시점 삽입(최고점수처럼 초기 전이 전에 존재 보장).

### 토글 경로
- **M 키:** `toggle_music_key`(Update, 상시) — `KeyCode::KeyM` just_pressed 시 `MusicEnabled` 반전 + `persist()`. (M은 현재 미사용 키.)
- **메뉴 항목:** 타이틀·일시정지 메뉴에 `MenuAction::ToggleMusic` 항목. 활성 시 `MusicEnabled` 반전 + persist(상태 전이 없음). 라벨은 현재 상태 반영: `MUSIC: ON` / `MUSIC: OFF`.
  - 동적 라벨: 해당 항목에 `MusicMenuItem` 마커. `update_music_menu_label` 시스템이 `highlight_menu` 이후 실행되어 그 항목 텍스트를 `[> ]MUSIC: ON/OFF`로 덮어쓴다(선택 시 `> ` 프리픽스 포함).
- **HELP:** 조작 안내 CONTROLS에 `M  music on/off` 추가.

## 데이터 흐름

```
build.rs(빌드시)  ──생성──▶  assets/music/*.wav
GameState/RunPhase + Progression.current_theme()
        │
        ▼
   desired_track ──▶ sync_music ──(교체)──▶ AudioPlayer(LOOP)   (MusicEnabled=true일 때만)
                                   ▲
MusicEnabled(persist) ◀── M키 / 메뉴 ToggleMusic
```

## 오류 처리·엣지 케이스

- **트랙 파일 없음:** `asset_server.load`는 핸들을 즉시 반환(비동기). wasm에서 로드 지연 시 그 트랙은 준비될 때까지 무음 — 크래시 없음. `AssetMetaCheck::Never`(wasm)로 .meta 404 방지(기존).
- **부팅 시 음소거 상태:** `MusicEnabled` 빌드시 삽입이라 첫 프레임부터 존재(패닉 없음).
- **테마 전환 중 교체:** `sync_music`은 `CurrentMusic.track != desired`일 때만 교체 → 매 프레임 재생성 없음.
- **음소거 토글 중복 경로:** M키·메뉴 둘 다 같은 `MusicEnabled`를 반전. 같은 프레임 동시 입력이라도 최종값은 결정적(각 just_pressed는 한 번). 메뉴 `ToggleMusic`는 상태 전이가 없어 HELP 오버레이의 입력 억제와 무관.
- **일시정지 중 음악:** `Paused`도 `Playing` 하위라 게임플레이 트랙 유지(일시정지해도 음악은 계속 — 일반적 관행). 원하면 후속으로 볼륨 다운 가능(범위 밖).

## 테스트 전략

**단위 테스트 가능(순수 로직):**
- `note_hz(semitone)` 값 검증(예: 0→BASE, 12→2×BASE).
- 시퀀서 샘플 생성의 **결정성·길이**(같은 패턴→같은 샘플 수, 정수 마디 길이) + **경계 진폭이 0 근처**(루프 이음새 클릭 없음)를 수치로 검증.
- `Track::for_theme(theme)` 매핑 전수.
- `desired_track(state, theme)` 상태별 선택 로직.
- 음소거 토글 반전 로직(true↔false).

**단위 테스트 불가(수동/청취):** 실제 음악 퀄리티·무드 구분·루프 자연스러움 — 사용자 청취 확인.

## 범위 밖(후속)

- 트랙 간 **크로스페이드**(현재 하드 컷).
- **보스 전용** 곡.
- 일시정지 시 **볼륨 덕킹**.
- 음량 단계(현재 ON/OFF만).

## 파일 구조

- **Create:** `src/fx/music.rs`(MusicPlugin·Track·CurrentMusic·sync_music·MusicEnabled·토글·메뉴 라벨).
- **Modify:** `build.rs`(음악 시퀀서 + 7 트랙 생성), `src/main.rs`(MusicPlugin 등록 + MusicEnabled 빌드시 삽입), `src/ui/menu.rs`(`MenuAction::ToggleMusic`), `src/ui/title.rs`·`src/ui/pause.rs`(MUSIC 메뉴 항목), `src/ui/help.rs`(CONTROLS에 M 추가), `src/fx.rs`(mod music).
