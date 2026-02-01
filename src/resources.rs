use crate::components::{PassiveNode, PetType, PlayerClass};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct PassiveTree {
    pub nodes: HashMap<u32, PassiveNode>,
    pub connections: Vec<(u32, u32)>,
}

#[derive(Resource, Default)]
pub struct PendingSelection {
    pub class: Option<PlayerClass>,
    pub pets: Vec<PetType>,
}

#[derive(Resource)]
pub struct CharacterSprites {
    pub orc_idle: Handle<Image>,
    pub orc_walk: Handle<Image>,
    pub orc_attack: Handle<Image>,
    pub soldier_idle: Handle<Image>,
    pub soldier_walk: Handle<Image>,
    pub soldier_attack: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Vec2);

#[derive(Resource)]
pub struct GameStats {
    pub enemies_killed: u32,
    pub damage_dealt: f32,
    pub time_survived: f32,
    pub show_stats: bool,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            enemies_killed: 0,
            damage_dealt: 0.0,
            time_survived: 0.0,
            show_stats: false,
        }
    }
}
