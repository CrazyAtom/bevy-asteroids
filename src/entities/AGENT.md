# AGENT.md — `src/entities/`

게임 **오브젝트**. 오브젝트당 한 모듈이며, 각 모듈은 보통 `컴포넌트 + 스폰 함수 + Plugin + 시스템`으로 구성됩니다.

## 모듈

- **`player.rs`** (가장 큼) — 우주선. 입력·회전(램프)·추진/브레이크·화염(자식 스프라이트)·실드 스프라이트·연사/확산 타이머·하이퍼스페이스·감쇠·디버그 핫키(F1).
- **`special_weapon.rs`** — 특수무기(X). `SpecialWeapon{kind,charges}`·`SpecialBeam`, `activate_special`(발동·충전 소모·빔 스폰)·`tick_beam`(수명). 판정은 `systems/collision`, 보스 피해는 `entities/boss`.
- **`asteroid.rs`** — `Asteroid{size}`, `spawn_asteroid`, `random_spawn_position`/`random_velocity`. 스폰만 담당(분열은 `systems/collision`이 호출).
- **`bullet.rs`** — `Bullet`(아군)/`EnemyBullet`(적), 발사·수명. 확산탄 발사 수 = `Spread.level * 3`.
- **`ufo.rs`** — `Ufo{size}` 2종, 타이머 스폰, 조준/무작위 사격(`ufo_fire`).
- **`powerup.rs`** — `PowerupKind` 5종, `pick_powerup_kind`, 자석(`magnet_velocity`/`attract_powerups`), 수집(`collect_powerup`), `powerup_sprite_index`.
- **`boss.rs` + `boss/`** — 루트는 보스 공통: `BossKind`(6종), `Boss{health,phase}`, `spawn_boss`, `boss_max_health`/`apply_boss_damage`, 이동·공격 **디스패치**(exhaustive match), `boss_combat`(피해·격파→다음 스테이지). **보스별 전용 로직은 서브모듈**: `golem`(교대 공격·다단계 페이즈) · `tesla`(블링크·폭풍-동기 EMP·체인 전격) · `singularity`(나선 탄·흡인 강화·러쉬).
- **`black_hole.rs`** — 블랙홀 테마 중력장: `BlackHole` 엔티티(Startup 스폰), `BlackHoleActive{active,strength}` 리소스(기본 off, stage가 동기화), `gravity_pull`(GravityBody 흡인), 사건의 지평선(소행성 소멸; 우주선 피해는 collision의 player_damage).

## 규칙

- **스폰 함수는 `&SpriteAssets`를 받아** 스폰 시 `Sprite`를 부착합니다. 표시 크기는 `fx::sprites::sprite_size_for(콜라이더 반경)`.
- 판 동안만 존재하는 엔티티엔 `core::state::GameplayEntity`를, 충돌 대상엔 `core::components::Collider{radius}`를 붙입니다.
- 이동/회전은 `Velocity`/`AngularVelocity`를, 경계 처리는 순환이면 `Wrapping`을, **벽 반사 대상(소행성·얼음 보스)**이면 `EdgeReflect`를 붙이면 `systems/movement`가 처리합니다(반사 활성 여부는 `StageModifiers`가 결정).
- 충돌·피해 판정은 여기서 하지 말고 `systems/collision`에 둡니다(엔티티 교차 관심사).
- **확장**: 새 특수무기는 `SpecialWeaponKind`(powerup.rs) 변형 + `special_weapon.rs::activate_special`의 발동 분기로. 새 보스는 `BossKind` 변형 + 루트 디스패치 arm 추가 + **전용 로직은 `boss/<이름>.rs` 서브모듈**로(골렘/테슬라/특이점 패턴).
- 방향성 스프라이트(총알·빔·화염)는 진행 방향으로 `Transform.rotation`을 맞춥니다(스프라이트 원본은 +Y 기준).
