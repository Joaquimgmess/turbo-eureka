use crate::components::{PassiveNode, PetType, PlayerClass};
use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct EnemyCount(pub usize);

#[derive(Resource)]
pub struct GameRng(pub StdRng);

impl Default for GameRng {
    fn default() -> Self {
        Self(StdRng::from_entropy())
    }
}

#[derive(Resource)]
pub struct HitStop {
    pub timer: Timer,
    pub active: bool,
}

impl Default for HitStop {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            active: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct MapTier(pub u32);

#[derive(Resource)]
pub struct MapData {
    pub seed: u64,
    pub bounds: f32,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            seed: 0,
            bounds: 1200.0,
        }
    }
}

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
