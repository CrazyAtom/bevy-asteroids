# Bevy Asteroids — Phase 4 설계 문서: 카툰 스프라이트 렌더링

- **작성일**: 2026-07-12
- **목표**: 현재의 Gizmos 벡터 라인(즉시 모드) 렌더링을, **자체 제작 카툰 스프라이트**로 전면 전환한다. 게임플레이 로직·물리·충돌·상태는 그대로 두고 "그리는 방식"만 바꾼다. 전환 후 렌더링에 Gizmos는 더 이상 쓰이지 않는다.
- **환경**: Rust 1.96 / Bevy `0.19` (기본 `bevy_sprite` 포함) / rand `0.10` / bevy-persistent `0.11`, macOS arm64
- **선행**: Phase 1·2·3 (main에 병합 완료). 사운드 에셋 경로 고정(`AssetPlugin.file_path = <project>/assets`) 이미 적용됨 → 스프라이트도 동일 경로에서 로드.

---

## 1. 배경 — 현재 렌더링

화면의 모든 것이 **Gizmos 즉시 모드**로 그려진다. 스프라이트·텍스처·메시가 전혀 없고, 매 프레임 `draw_*` 시스템이 선/원을 다시 발행한다.

현재 그리기 시스템 (9개): `draw_player`(우주선+화염), `draw_shield`, `tick_and_draw_beam`(특수무기 빔), `draw_asteroids`, `draw_ufos`, `draw_bullets`/`draw_enemy_bullets`, `draw_powerups`, `draw_stars`, `draw_particles`.

---

## 2. 결정 사항 (브레인스토밍 합의)

1. **렌더링 방식** = 스프라이트(실제 그림 에셋). 3D 셀셰이딩·절차적 채움은 배제.
2. **아트 소스** = **자체 제작 SVG → PNG(build.rs 자동 생성)**, 저장소 자체 완결. 외부 팩·다운로드·라이선스 의존 없음. 스타일은 브레인스토밍 목업의 **"두꺼운 어두운 외곽선 + 플랫 컬러 카툰"**. SVG를 진실의 원본으로 커밋, PNG는 빌드 시 생성(gitignore).
3. **전환 범위** = **모든 시각 요소**:
   - 우주선 · **추진/브레이크 화염** · 소행성(대/중/소) · 적 UFO(2종) · 총알(아군/적) · 파워업(5종) · 배경
   - 특수무기 **빔**(스프라이트 빔)
   - **파편 파티클**(텍스처 파티클, 물리 유지) + **애니메이션 폭발**(개별 프레임 이미지 스왑)
   - **실드**(스프라이트 버블/링)
4. **절차적 유지** = **없음.** 모든 `draw_*`가 스프라이트로 대체되거나 제거된다. 화염도 스프라이트로 전환.
5. **게임플레이 불변** = 이동·발사·충돌·분열·웨이브·점수·상태 전이는 변경 없음. 기존 게임플레이 테스트(현재 56개) 전부 유지·통과.

---

## 3. 아키텍처 전환 — 즉시 모드 → 스프라이트 엔티티

### 3.1 핵심 원리

- **스폰 시 `Sprite` 부착 → 렌더러가 자동 그리기.** `draw_*`를 매 프레임 호출하는 대신, 엔티티 스폰 시점에 `Sprite`(이미지 핸들 + 표시 크기)를 붙인다. Bevy 렌더러가 `Transform`(위치·회전·크기)만으로 매 프레임 그린다.
- **로직 불변, 그리기만 교체.** 회전/스핀은 이미 `Transform`으로 처리되므로 이동·회전 시스템은 그대로. 삭제되는 것은 그리기 시스템뿐.
- **렌더 전용 데이터 제거.** `AsteroidShape`(폴리곤 점)와 우주선 점 배열은 렌더 전용 → 제거. 충돌은 `Collider{radius}`만 사용하므로 게임플레이 영향 없음(코드 단순화).

### 3.2 에셋 핸들 리소스 — `SpriteAssets`

시작 시(`Startup`) 모든 PNG를 한 번 로드해 `Handle<Image>`를 보관하는 리소스를 둔다. 각 스폰 시스템은 여기서 핸들을 꺼내 쓴다(반복 로드 방지).

```
#[derive(Resource)]
struct SpriteAssets {
    ship, flame,
    meteor_large, meteor_medium, meteor_small,
    ufo_large, ufo_small, bullet, enemy_bullet,
    powerup: [Handle<Image>; 5], // PowerupKind 순서
    beam, spark, shield, background,
    explosion_frames: Vec<Handle<Image>>, // 폭발 프레임 순서
}
```

### 3.3 신규 서브시스템 — 애니메이션 (`fx::animation`)

폭발은 여러 프레임이 순서대로 재생되는 애니메이션이다. **개별 프레임 이미지를 프레임마다 교체**하는 방식으로 구현한다(스프라이트 시트/아틀라스 불필요 — 합성 도구 없이 rsvg-convert만으로 프레임 PNG 생성 가능).

- `#[derive(Component)] struct FrameAnimation { frames: Vec<Handle<Image>>, timer: Timer /*Repeating*/, index: usize }`
- 폭발 스폰: `Sprite::from_image(frames[0])` + `FrameAnimation{ frames: assets.explosion_frames.clone(), .. }` + `Transform`(폭발 위치, z=EXPLOSION).
- `advance_animation` 시스템: 타이머 tick → `index += 1`, `sprite.image = frames[index]`; **마지막 프레임 도달 시 엔티티 despawn**.
- 프레임 진행 판정은 순수 함수로 분리해 테스트: `next_frame_index(current, last) -> Option<usize>`(마지막이면 `None`=despawn 신호).

### 3.4 텍스처 파티클 (`fx::effects` 수정)

- 기존 파티클 물리(폭발/피격 시 N개가 무작위 속도로 튀어나가 수명 후 소멸)는 **그대로 유지**.
- 스폰 시 원 대신 작은 스파크 스프라이트(`spark`)를 붙인다.
- 페이드: `fade_particles` 시스템이 `Particle.life.fraction_remaining()`을 `Sprite.color`의 알파(및/또는 스케일)에 반영. 기존 `draw_particles`는 제거.

### 3.5 스프라이트 빔 (`player.rs` 수정)

- 특수무기 발동 시 빔 엔티티에 빔 스프라이트를 `custom_size = (BEAM_WIDTH, 사거리)`로 붙이고, 원점 기준 정면 방향으로 회전·배치. 수명 타이머로 despawn.
- 빔 **판정 로직**(대상 히트·중복 방지 HashSet)은 변경 없음. `tick_and_draw_beam`의 Gizmos 그리기만 스프라이트로 대체.

### 3.6 화염 스프라이트 (`player.rs` 수정)

- 우주선 스폰 시 **화염 스프라이트를 자식 엔티티**로 달고 기본 `Visibility::Hidden`.
- `update_flame` 시스템: `EngineState`(추진/브레이크)를 읽어 화염 자식의 가시성을 토글하고 위치를 조정(추진=후미, 브레이크=전방). 플리커는 화염 프레임 2장을 빠른 타이머로 번갈아 교체(또는 스케일/알파 펄스).
- 화염은 우주선 본체보다 살짝 뒤(로컬 z 낮게) 배치해 "뒤로 뿜어져 나오는" 인상을 준다. 색은 카툰 팔레트(주황/노랑).

### 3.7 레이어(Z) 계획

Gizmos는 층 개념이 없지만 스프라이트는 `Transform.translation.z`로 앞뒤가 정해진다. 상수로 명시:

| 레이어 | z | 대상 |
|---|---|---|
| 배경 | −100 | 배경 스프라이트, 별 |
| 화염 | −1 | 우주선 화염(본체 뒤) |
| 물체 | 0 | 우주선·소행성·UFO·총알·파워업 |
| 실드 | 5 | 우주선 실드 버블 |
| 빔 | 10 | 특수무기 빔 |
| 파티클 | 12 | 파편 스파크 |
| 폭발 | 15 | 애니메이션 폭발 |

(HUD·게임오버는 기존 Bevy UI `Node`라 항상 최상단, 영향 없음.)

---

## 4. 에셋 계획 — 자체 제작 SVG → PNG

### 4.1 아트 파이프라인 (build.rs 자동 생성 — 사운드 WAV와 동일 패턴)

```
assets/sprites/
├── src/*.svg      # 진실의 원본(손으로 그린 SVG). 커밋.
└── *.png          # build.rs가 빌드 시 생성. gitignore(커밋 안 함).
build.rs           # 기존 WAV 생성에 더해 SVG→PNG 래스터화 추가
```

- **`cargo build` 시 자동 재생성.** build.rs가 순수 Rust 래스터라이저(`resvg`/`usvg`/`tiny-skia`, `[build-dependencies]`)로 `src/*.svg`를 지정 픽셀 크기의 PNG로 굽는다. `cargo:rerun-if-changed=assets/sprites/src`로 **SVG가 바뀔 때만** 다시 굽어 평소 빌드는 느려지지 않는다.
- 생성 WAV와 동일하게 PNG는 `.gitignore`, SVG 소스만 커밋 → 저장소 자체 완결(외부 도구·다운로드 0, 순수 Rust라 이식성 좋음).
- 스타일 가이드(모든 스프라이트 공통): **플랫 채색 + 두꺼운 어두운 외곽선 + 하이라이트 1톤·섀도 1톤.** 브레인스토밍 목업과 동일.

### 4.2 스프라이트 목록 (~22장)

| 요소 | 파일 | 색/비고 |
|---|---|---|
| 우주선 | `ship.png` | 청록 로켓 |
| 화염 | `flame.png` (+플리커용 2프레임) | 주황/노랑, 추진/브레이크 공용 |
| 소행성 대/중/소 | `meteor_large/medium/small.png` | 갈색 돌덩이(크레이터) |
| 적 UFO 대/소 | `ufo_large.png` / `ufo_small.png` | 초록/노랑 접시 |
| 총알 / 적 총알 | `bullet.png` / `enemy_bullet.png` | 청록 / 빨강 볼트 |
| 파워업 5종 | `powerup_shield/rapid/spread/life/special.png` | 종류별 배지 |
| 빔 | `beam.png` | 빛나는 세로 빔 |
| 파편 | `spark.png` | 소형 스파크 |
| 실드 | `shield.png` | 반투명 버블 링 |
| 배경 | `background.png` | 성운+별, 1280×720 |
| 폭발 | `explosion_0.png` … `explosion_N.png` | 확장하는 버스트 5~6프레임 |

### 4.3 크기 매핑 (순수 함수)

`sprite_size_for(radius: f32) -> Vec2` — 스프라이트 표시 크기를 기존 콜라이더 반경에 맞춰 산출해 **충돌과 그림이 일치**하게 한다. 대략 `Vec2::splat(radius * 2.0 * VISUAL_FIT)`(여백 보정 계수 `VISUAL_FIT` ≈ 1.1~1.3, config 상수). 테스트 대상. PNG는 선명도를 위해 목표의 약 2배 해상도로 굽고 `custom_size`로 축소.

### 4.4 라이선스 · 저장

- 전부 자체 제작이라 **외부 라이선스·크레딧 불필요.**
- **SVG 소스만 커밋**, 생성 PNG는 `.gitignore`(예: `/assets/sprites/*.png`) — 생성 WAV와 동일. PNG는 매 빌드 시 build.rs가 재생성.

---

## 5. 절차적 요소

- **없음.** 모든 시각 요소가 스프라이트/텍스처로 전환된다. 전환 완료 후 렌더링에서 Gizmos 호출은 0.

---

## 6. 모듈별 변경 요약

| 모듈 | 변경 |
|---|---|
| `main.rs` | `SpriteAssets` 로드 시스템 등록, `AnimationPlugin` 등록 |
| `build.rs` | 기존 WAV 생성에 더해 SVG→PNG 래스터화 추가(rerun-if-changed로 증분) |
| `Cargo.toml` | `[build-dependencies]`에 `resvg`/`usvg`/`tiny-skia` 추가 |
| 신규 `core::config` | 스프라이트 크기 계수·Z 레이어·폭발 프레임 타이밍 상수 |
| 신규 `fx::animation` | `FrameAnimation` + `advance_animation` |
| `fx::background` | `draw_stars` 제거 → 배경 스프라이트(+별) 스폰 |
| `fx::effects` | `draw_particles` 제거 → 스파크 스프라이트 파티클 + `fade_particles` |
| `entities::asteroid` | `AsteroidShape`·`draw_asteroids` 제거 → 스폰 시 크기별 `Sprite` |
| `entities::ufo` | `draw_ufos` 제거 → 스폰 시 UFO `Sprite` |
| `entities::bullet` | `draw_bullets`/`draw_enemy_bullets` 제거 → 스폰 시 총알 `Sprite` |
| `entities::powerup` | `draw_powerups` 제거 → 스폰 시 종류별 `Sprite` |
| `entities::player` | 우주선/실드/화염 `Sprite`, 스프라이트 빔; `draw_player`·`draw_shield`·화염 Gizmos 제거 |

---

## 7. 테스트 전략

렌더 자체는 단위 테스트 대상이 아니다. 이 프로젝트 관례대로 **순수 로직만 테스트**한다:

- `sprite_size_for(radius)` — 크기 매핑(반경↑ → 크기↑, 계수 반영).
- `next_frame_index(current, last)` — 폭발 프레임 진행/종료 클램프.
- 파티클 알파 매핑(수명 분율 → 알파, 0~1 경계).
- 파워업 종류→스프라이트 인덱스 매핑(있다면).

기존 게임플레이 테스트(이동·발사·충돌·웨이브·특수무기 등 56개)는 렌더와 무관하므로 **그대로 유지·통과**해야 한다. `AsteroidShape` 제거로 컴파일 영향 받는 코드/테스트가 있으면 최소 수정.

---

## 8. 롤아웃 (구현 계획 태스크 미리보기)

에셋과 엔티티를 단위로 점진 교체 — 각 단계가 독립적으로 실행되는 상태를 유지한다.

1. 아트 파이프라인: SVG 소스 + build.rs 래스터화(resvg/usvg, PNG는 gitignore·빌드 생성) + `SpriteAssets` 리소스 + 크기 매핑 함수(+테스트)
2. 배경 스프라이트(`draw_stars` 대체) — 레이어 상수 도입
3. 소행성 스프라이트(`AsteroidShape`/`draw_asteroids` 제거)
4. 우주선 + 화염 스프라이트(`draw_player` 제거)
5. UFO · 총알(아군/적) 스프라이트
6. 파워업 5종 스프라이트
7. 실드 스프라이트
8. 스프라이트 빔
9. 텍스처 파티클(파편) + `fade_particles`
10. `fx::animation` + 애니메이션 폭발
11. 팔레트 미세 조정 · 최종 정리

---

## 9. 리스크 · 오픈 이슈

- **아트 품질**: 자체 제작 SVG는 "깔끔한 플랫 카툰" 수준(프로 일러스트 아님). 게임 규모엔 충분하며 스타일 100% 일관.
- **빌드 의존성**: build.rs가 `resvg`/`usvg`/`tiny-skia`(build-dependencies)로 PNG를 생성 → 최초 빌드 시 이 크레이트를 네트워크로 받는다(오프라인 최초 빌드 불가). 순수 Rust라 rsvg-convert 등 외부 바이너리는 불필요. build-deps라 최종 실행 바이너리엔 포함되지 않음.
- **스프라이트 방향/크기 미세 조정**: 우주선·빔·화염의 기준 방향(정면 +Y)과 SVG 원본 방향 정렬 필요. 크기·정렬은 순수 함수/상수라 조정 용이.
- **폭발 프레임 수/타이밍**: 프레임 장수와 프레임 간격은 그려보며 확정(5~6장 기준).

## 10. 범위 밖 (향후)

- 패럴랙스 다층 배경, 우주선/파워업 스킨 선택, UFO 반짝임 등 추가 프레임 애니메이션은 이번 범위 밖. 구조는 이를 나중에 얹을 수 있게 둔다.
