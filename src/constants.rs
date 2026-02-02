// === WORLD ===
pub const MAP_BOUNDS: f32 = 1200.0;
pub const TILE_SIZE: f32 = 200.0;

// === PLAYER ===
pub const PLAYER_RADIUS: f32 = 25.0;
pub const PLAYER_SPRITE_SIZE: f32 = 180.0;
pub const PLAYER_Z: f32 = 10.0;

// === COMBAT ===
pub const PROJECTILE_HIT_RADIUS: f32 = 65.0;
pub const MELEE_HIT_RADIUS: f32 = 110.0;
pub const PROJECTILE_SPEED: f32 = 550.0;
pub const PROJECTILE_LIFETIME: f32 = 2.0;
pub const KNOCKBACK_FORCE: f32 = 35.0;

// === XP ===
pub const XP_PICKUP_RADIUS: f32 = 22.0;
pub const XP_ATTRACT_RADIUS: f32 = 120.0;
pub const XP_ORB_LIFETIME: f32 = 12.0;

// === ENEMY ===
pub const ENEMY_ATTACK_RANGE: f32 = 85.0;
pub const ENEMY_STOP_RANGE: f32 = 80.0;
pub const MAX_ENEMIES_BASE: u32 = 8;
pub const MAX_ENEMIES_CAP: u32 = 30;
pub const BOSS_SPAWN_INTERVAL: f32 = 120.0;

// === UI ===
pub const HEALTH_BAR_WIDTH: f32 = 50.0;
pub const HEALTH_BAR_HEIGHT: f32 = 8.0;
pub const DAMAGE_NUMBER_LIFETIME: f32 = 0.7;

// === SKILLS ===
pub const DASH_COOLDOWN: f32 = 2.0;
pub const DASH_SPEED: f32 = 900.0;
pub const DASH_DURATION: f32 = 0.12;
pub const NOVA_COOLDOWN_DEFAULT: f32 = 5.0;

// === SCALING ===
pub const LEVEL_HEALTH_SCALE: f32 = 0.25;
pub const LEVEL_DAMAGE_SCALE: f32 = 0.15;
pub const LEVEL_XP_MULTIPLIER: f32 = 1.4;

// === ELEMENTAL ===
pub const MAX_ELEMENTAL_STACKS: u32 = 10;
pub const FIRE_ARMOR_REDUCTION_PER_STACK: f32 = 0.05;
pub const FIRE_MAX_ARMOR_REDUCTION: f32 = 0.50;
