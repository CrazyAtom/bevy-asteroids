# AGENT.md — `src/entities/`

게임 **오브젝트**. 오브젝트당 한 모듈이며, 각 모듈은 보통 `컴포넌트 + 스폰 함수 + Plugin + 시스템`으로 구성됩니다.

## 모듈

- **`player.rs`** (가장 큼) — 우주선. 입력·회전(램프)·추진/브레이크·화염(자식 스프라이트)·실드 스프라이트·연사/확산 타이머·하이퍼스페이스·감쇠·디버그 핫키(F1).
- **`special_weapon.rs`** — 특수무기(X). `SpecialWeapon{kind,charges}`·`SpecialBeam`, `activate_special`(발동·충전 소모·빔 스폰)·`tick_beam`(수명). 판정은 `systems/collision`, 보스 피해는 `entities/boss`.
- **`asteroid.rs`** — `Asteroid{size}`, `spawn_asteroid`, `random_spawn_position`/`random_velocity`. 스폰만 담당(분열은 `systems/collision`이 호출).
- **`bullet.rs`** — `Bullet`(아군)/`EnemyBullet`(적), 발사·수명. 확산탄 발사 수 = `Spread.level * 3`.
- **`ufo.rs`** — `Ufo{size}` 2종, 타이머 스폰, 조준/무작위 사격(`ufo_fire`).
- **`powerup.rs`** — `PowerupKind` 5종, `pick_powerup_kind`, 자석(`magnet_velocity`/`attract_powerups`), 수집(`collect_powerup`), `powerup_sprite_index`.
- **`boss.rs`** — `BossKind`(모암/모함/코어), `Boss{health}`, `spawn_boss`, `boss_max_health`/`apply_boss_damage`, kind별 `boss_movement`/`boss_attack`, `boss_combat`(피해·격파→다음 스테이지).

## 규칙

- **스폰 함수는 `&SpriteAssets`를 받아** 스폰 시 `Sprite`를 부착합니다. 표시 크기는 `fx::sprites::sprite_size_for(콜라이더 반경)`.
- 판 동안만 존재하는 엔티티엔 `core::state::GameplayEntity`를, 충돌 대상엔 `core::components::Collider{radius}`를 붙입니다.
- 이동/회전/화면순환은 `Velocity`/`AngularVelocity`/`Wrapping`만 붙이면 `systems/movement`가 처리합니다.
- 충돌·피해 판정은 여기서 하지 말고 `systems/collision`에 둡니다(엔티티 교차 관심사).
- **확장**: 새 특수무기는 `SpecialWeaponKind`(powerup.rs) 변형 + `special_weapon.rs::activate_special`의 발동 분기로, 새 보스는 `BossKind` 변형 + `boss_movement`/`boss_attack`의 arm 추가로.
- 방향성 스프라이트(총알·빔·화염)는 진행 방향으로 `Transform.rotation`을 맞춥니다(스프라이트 원본은 +Y 기준).
