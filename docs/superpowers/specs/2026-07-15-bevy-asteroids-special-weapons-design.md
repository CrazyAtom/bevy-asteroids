# Bevy Asteroids — 특수무기 다양화 설계 문서: 5종 무기 + 카트라이더 큐

- **작성일**: 2026-07-15
- **목표**: 특수무기를 1종(레이저 빔) → **5종**으로 확장하고, 획득·발동을 **카트라이더 방식 큐(FIFO)**로 바꾼다. 획득한 순서대로 쌓이고 X를 누르면 먼저 얻은 무기부터 발동. 드롭 아이콘이 종류별로 구분되어 "보고 줍는" 재미를 더한다.
- **환경**: Rust 1.96 / Bevy `0.19` / rand `0.10`, macOS arm64. 카툰 스프라이트(자체 SVG→PNG).
- **선행**: 스테이지 다양화(테마 6종) 완성, 모듈 재구조화("폴더=기능, 파일=변형" — `boss/` 패턴) 병합 완료.

---

## 1. 배경 — 현재 구조

`entities/special_weapon.rs`(101줄): `SpecialWeapon{kind, charges}`, `SpecialBeam`, `activate_special`(X → 충전 소모 → kind match, 현재 LaserBeam 1개 arm), `tick_beam`. `SpecialWeaponKind{LaserBeam}`은 **powerup.rs에 정의**(역방향 의존 — special_weapon이 powerup을 import). 드롭은 `PowerupKind::SpecialWeapon(kind)` 단일 아이콘, 획득 시 `kind` 교체 + `charges += 1`. HUD는 아이콘 1개 + 충전 숫자 텍스트.

---

## 2. 결정 사항 (브레인스토밍 합의)

1. **무기 5종** = 레이저 빔(기존 유지) + **산탄 노바** + **유도 미사일(5발)** + **충격파** + **공격형 실드**.
2. **카트라이더 큐(FIFO)**: 랜덤 종류가 드롭되고, 획득 순서대로 큐에 쌓이며, **X = 큐 맨 앞 무기 발동**(pop). 무기 선택/순환 키 없음 — "무엇을 주웠는가"가 전략.
3. **공격형 실드 표현** = 버블 없이 **우주선 자체가 번쩍임**(주황 고주파 틴트) + **크기 펄스(±5%)**, 종료 시 원복.
4. **소유권 정리**: `SpecialWeaponKind` 정의를 powerup.rs → special_weapon.rs로 이동(의존 방향을 powerup→special_weapon으로 바로잡음).
5. **모듈 구조**: 재구조화 규칙("파일=변형") 적용 — `special_weapon.rs` 루트(큐·발동 디스패치) + `special_weapon/{beam,nova,missile,shockwave,shield_burst}.rs`.

---

## 3. 상세 설계

### 3.1 무기 큐 (카트라이더 방식)

```rust
// special_weapon.rs (루트)
pub enum SpecialWeaponKind { LaserBeam, ScatterNova, HomingMissile, Shockwave, ShieldBurst }

#[derive(Component)]
pub struct SpecialWeapon {
    pub queue: VecDeque<SpecialWeaponKind>, // 획득 순서(FIFO). 앞 = 다음 발동.
}
```

- **시작 큐** = `[LaserBeam]`(기존 STARTING_SPECIAL_CHARGES=1과 동일한 체감).
- `activate_special`: X 입력 → `queue.pop_front()` → kind match로 **서브모듈 fire 위임**(boss 디스패치 패턴). 큐 비면 무동작.
- 큐는 무제한(내부), **HUD 표시는 앞 5개**.
- 드롭 획득: `queue.push_back(kind)`.
- F1 디버그: 5종을 하나씩 push(전 무기 시연용).

### 3.2 무기별 동작 (서브모듈)

| 모듈 | 발동(fire) | 재사용 |
|---|---|---|
| `beam.rs` | 기존 레이저 빔(SpecialBeam·tick_beam 이동, 동작 불변) | collision::beam_vs_targets, boss_combat |
| `nova.rs` | 우주선 위치에서 전방위 **아군 탄 16발**(NOVA_BULLETS) | `bullet.rs`에 `spawn_bullet(commands, assets, pos, dir)` 헬퍼 추출 후 재사용(fire_bullet 내부도 이 헬퍼 사용으로 정리) |
| `missile.rs` | 전방 부채꼴로 **미사일 5발**(MISSILE_COUNT). 각 미사일 = `Bullet` + `Homing` 마커 → **기존 총알 충돌·보스 피해 경로 그대로 재사용**. `homing_steer` 시스템이 가장 가까운 표적(소행성/UFO/보스)으로 조향(MISSILE_TURN_RATE 제한) | bullet 충돌 전부 |
| `shockwave.rs` | 반경 SHOCKWAVE_RADIUS 내: **적탄 전부 소멸**, 소행성/UFO **강하게 밀침**(중심→바깥 SHOCKWAVE_IMPULSE), **Small 소행성 파괴**(점수+폭발, 분열 없음). 링 연출(실드 스프라이트 확대·페이드) + 흔들림 | try_despawn, spawn_explosion |
| `shield_burst.rs` | `ShieldBurst{timer}`(SHIELD_BURST_SECS) 부여 + 기존 `Shield`(같은 시간) 동시 부여(무적 재사용). 지속 중 **접촉한 소행성/UFO/적탄 파괴**(점수+폭발, 분열 없음; 보스는 무적만). **우주선 번쩍임**: `Sprite.color`를 주황↔흰색 고주파 토글 + `custom_size` ±5% 펄스, 만료 프레임에 색·크기 원복 | Shield, circles_overlap |

- 순수 함수(테스트 대상): `nova_directions(n) -> Vec<Vec2>`(균등 각), `nearest_target(from, &[Vec2]) -> Option<usize>`, `steer_toward(cur_dir, to_target, max_turn, dt) -> Vec2`(회전 제한 조향), `shockwave_hits(center, r, pos) -> bool`, `burst_flash_color(t)`/`burst_pulse_scale(t)`(번쩍임·펄스 커브).

### 3.3 드롭·아이콘

- `pick_powerup_kind`의 특수무기 분기에서 **5종 균등 랜덤**(`pick_weapon_kind(roll)` 순수 함수).
- 드롭 엔티티 스프라이트 = **종류별 아이콘**: `weapon_icon(kind, &SpriteAssets)`. 기존 `powerup_special`(빔) + 신규 SVG 4종(`weapon_nova`, `weapon_missile`, `weapon_shockwave`, `weapon_burst`).
- 미사일 전용 스프라이트 `missile.svg` 1종(방향성 — +Y 기준, 진행 방향 회전).

### 3.4 HUD — 큐 슬롯

- 기존 "아이콘+숫자" → **무기 슬롯 아이콘 나열**(`WeaponSlotIcon(usize)`, HUD_QUEUE_SLOTS=5).
- 매 프레임 큐 앞 5개의 kind 아이콘으로 `ImageNode` 이미지 교체·표시, 빈 슬롯 숨김. **맨 앞(다음 발동)은 불투명, 나머지는 반투명**(alpha 구분).
- 큐가 5개 초과면 5개까지만 표시(내부 보존).

### 3.5 기존 시스템과의 관계 / 변경점 요약

| 파일 | 변경 |
|---|---|
| `entities/special_weapon.rs` | 루트로 개편: Kind(이동해 옴)·SpecialWeapon{queue}·발동 디스패치·플러그인. 서브모듈 선언 |
| `entities/special_weapon/` (신규 5파일) | beam(기존 로직 이동)·nova·missile·shockwave·shield_burst |
| `entities/powerup.rs` | `SpecialWeaponKind` 정의 제거(→import), 드롭 kind 5종 랜덤, 드롭 스프라이트 kind별, collect 시 `queue.push_back` |
| `entities/bullet.rs` | `spawn_bullet` 헬퍼 추출(pub), fire_bullet이 이를 사용 |
| `entities/player.rs` | 시작 큐 `[LaserBeam]`, F1 디버그 5종 push, (collision의 리스폰 경로 동일) |
| `systems/collision.rs` | 변경 최소(미사일=Bullet이라 그대로). shield_burst 접촉 파괴는 shield_burst.rs 시스템이 담당 |
| `ui/hud.rs` | SpChargeText → WeaponSlotIcon 슬롯 5개 |
| `core/config/combat.rs` | NOVA_BULLETS=16, MISSILE_COUNT=5, MISSILE_SPEED=420, MISSILE_TURN_RATE=6.0, MISSILE_LIFETIME_SECS=2.5, SHOCKWAVE_RADIUS=220, SHOCKWAVE_IMPULSE=380, SHIELD_BURST_SECS=4.0, HUD_QUEUE_SLOTS=5 |
| `fx/sprites.rs` | `weapon_nova/missile/shockwave/burst`, `missile` 핸들 추가 |
| `assets/sprites/src/` | 신규 SVG 5종(아이콘 4 + 미사일 1) |

- **STARTING_SPECIAL_CHARGES**는 "시작 큐 길이" 의미로 유지(=1, LaserBeam).
- `debug_fill_hud`(F1): 기존 `charges += 1` → 5종 각 1개 push로 변경.

---

## 4. 테스트 전략

순수 로직 위주(렌더/연출 비테스트):

- **큐 FIFO**: push 순서대로 pop(획득 순서 발동), 발동 시 1개 소모, 빈 큐 무동작.
- `nova_directions(16)` — 16개, 균등 각, 단위 벡터.
- `nearest_target` — 최근접 선택, 빈 목록 None. `steer_toward` — 회전량 ≤ max_turn·dt, 표적 방향으로 수렴.
- `shockwave_hits` — 반경 내/외 판정.
- `pick_weapon_kind` — roll 구간별 5종 전부 도달.
- `weapon_icon` — kind→핸들 매핑(더미 에셋).
- spawn 테스트: 노바 발동 → Bullet 16개, 미사일 발동 → Bullet+Homing 5개, shield_burst 발동 → ShieldBurst+Shield 부여, 접촉 파괴(run_system_once).
- 기존 95개 테스트 유지·통과(빔 동작 불변 포함).

---

## 5. 리스크 · 오픈 이슈

- **미사일 조향 체감**: TURN_RATE가 낮으면 빗나가고 높으면 100% 명중이라 지루 → 상수 튜닝(실기).
- **충격파 밀침 vs 벽 반사**(얼음 테마): 밀린 소행성이 벽에서 튕겨 되돌아옴 — 의도된 상호작용으로 수용.
- **공격실드 vs 사건의 지평선**(블랙홀): Shield 재사용 덕에 지평선 무피해(기존 실드 규칙 상속) — 명시적으로 유지.
- **HUD 폭**: 슬롯 5개 추가로 상단 행이 길어짐 — 아이콘 크기(HUD_ICON_PX)로 조절.
- **틴트 원복**: shield_burst 만료 프레임에 `Color::WHITE`·원크기 복원 필수(테슬라 블링크 원복과 같은 규칙).

## 6. 범위 밖 (YAGNI)

- 무기 슬롯 상한/버리기 키, 무기별 레벨/강화, 큐 조작(순서 바꾸기) — 범위 밖.
- 시간 감속 무기 — 이번 라인업에서 제외(전역 속도 변조 복잡도).
- 무기별 전용 사운드 — 기존 Special 사운드 공유(후속에서 분화 가능).
