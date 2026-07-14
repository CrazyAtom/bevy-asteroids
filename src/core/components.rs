use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy)]
pub struct AngularVelocity(pub f32);

#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
}

#[derive(Component)]
pub struct Wrapping;

/// 경계에서 반사(벽 튕김) 대상임을 표시하는 마커. 실제 반사/순환 여부는
/// `StageModifiers.wall_bounce`가 결정한다(얼음 스테이지에서만 반사).
#[derive(Component)]
pub struct EdgeReflect;
