use crate::components::PassiveEffect;
use bevy::prelude::*;

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub attacker: Option<Entity>,
    pub amount: f32,
    pub is_crit: bool,
}

#[derive(Event)]
pub struct SpawnXpOrbEvent {
    pub position: Vec3,
    pub value: u32,
}

#[derive(Event)]
pub struct ApplyStatusEvent {
    pub target: Entity,
    pub effect: PassiveEffect,
}
