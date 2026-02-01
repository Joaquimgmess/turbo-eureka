use crate::components::{PetType, PlayerClass};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PendingSelection {
    pub class: Option<PlayerClass>,
    pub pets: Vec<PetType>,
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
