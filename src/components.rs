use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub radius: f32,
}

#[derive(Component)]
pub struct Wrapping;
