use crate::components::{PassiveNode, PetType, PlayerClass};
use bevy::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;
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

#[derive(Resource)]
pub struct KillFeedback {
    pub recent_kills: u32,
    pub kill_timer: Timer,
    pub last_kill_was_crit: bool,
}

impl Default for KillFeedback {
    fn default() -> Self {
        Self {
            recent_kills: 0,
            kill_timer: Timer::from_seconds(0.5, TimerMode::Once),
            last_kill_was_crit: false,
        }
    }
}

#[derive(Resource)]
pub struct SlowMotion {
    pub timer: Timer,
    pub time_scale: f32,
    pub active: bool,
}

impl Default for SlowMotion {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            time_scale: 1.0,
            active: false,
        }
    }
}

// === PROGRESSION EVENTS ===

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressionEventType {
    EliteSpawn,
    HordeWave,
    TreasureGoblin,
    Shrine,
    ChallengeZone,
}

#[derive(Resource)]
pub struct ProgressionEventTimer {
    pub timer: Timer,
    pub last_event: Option<ProgressionEventType>,
}

impl Default for ProgressionEventTimer {
    fn default() -> Self {
        use crate::constants::EVENT_MIN_INTERVAL;
        Self {
            timer: Timer::from_seconds(EVENT_MIN_INTERVAL, TimerMode::Once),
            last_event: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct HordeWaveActive {
    pub active: bool,
    pub timer: Option<Timer>,
}

#[derive(Resource, Default)]
pub struct MiniBossTimer {
    pub timer: Timer,
}

impl MiniBossTimer {
    pub fn new() -> Self {
        use crate::constants::MINI_BOSS_SPAWN_INTERVAL;
        Self {
            timer: Timer::from_seconds(MINI_BOSS_SPAWN_INTERVAL, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Default)]
pub struct BossEntranceActive(pub bool);

#[derive(Resource, Default)]
pub struct BossDeathActive(pub bool);
