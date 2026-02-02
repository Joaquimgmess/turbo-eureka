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

// === GAME FEEL ===
pub const HITSTOP_NORMAL_FRAMES: u64 = 4;
pub const HITSTOP_CRIT_FRAMES: u64 = 7;
pub const HITSTOP_KILL_FRAMES: u64 = 11;
pub const FRAME_MS: u64 = 16;
pub const DAMAGE_FLASH_DURATION: f32 = 0.15;
pub const KNOCKBACK_SMALL_MULT: f32 = 1.5;
pub const KNOCKBACK_MEDIUM_MULT: f32 = 1.0;
pub const KNOCKBACK_LARGE_MULT: f32 = 0.5;
pub const KNOCKBACK_BOSS_MULT: f32 = 0.15;
pub const KNOCKBACK_DURATION: f32 = 0.12;
pub const SCREEN_SHAKE_MAX: f32 = 18.0;
pub const ZOOM_PUNCH_AMOUNT: f32 = 0.025;
pub const ZOOM_PUNCH_DURATION: f32 = 0.08;
pub const ZOOM_RETURN_DURATION: f32 = 0.15;

// === XP ===
pub const XP_PICKUP_RADIUS: f32 = 22.0;
pub const XP_ATTRACT_RADIUS: f32 = 120.0;
pub const XP_ORB_LIFETIME: f32 = 12.0;

// === ENEMY ===
pub const ENEMY_ATTACK_RANGE: f32 = 85.0;
pub const ENEMY_STOP_RANGE: f32 = 80.0;
pub const MAX_ENEMIES_BASE: u32 = 8;
pub const MAX_ENEMIES_CAP: u32 = 30;
pub const BOSS_SPAWN_INTERVAL: f32 = 67.5;
pub const MINI_BOSS_SPAWN_INTERVAL: f32 = 30.0;

// === PROGRESSION EVENTS ===
pub const EVENT_MIN_INTERVAL: f32 = 30.0;
pub const EVENT_MAX_INTERVAL: f32 = 45.0;
pub const ELITE_HP_MULTIPLIER: f32 = 3.0;
pub const ELITE_XP_MULTIPLIER: f32 = 3.0;
pub const ELITE_AURA_RADIUS: f32 = 100.0;
pub const HORDE_WAVE_DURATION: f32 = 10.0;
pub const HORDE_SPAWN_MULTIPLIER: f32 = 2.0;
pub const TREASURE_GOBLIN_LIFETIME: f32 = 5.0;
pub const TREASURE_GOBLIN_SPEED: f32 = 280.0;
pub const TREASURE_GOBLIN_XP_MULT: u32 = 10;
pub const SHRINE_BUFF_DURATION: f32 = 30.0;
pub const CHALLENGE_ZONE_RADIUS: f32 = 150.0;
pub const CHALLENGE_ZONE_DURATION: f32 = 15.0;
pub const CHALLENGE_ZONE_XP_MULT: f32 = 2.0;
pub const BOSS_ENTRANCE_DURATION: f32 = 1.5;
pub const BOSS_DEATH_SLOWMO_DURATION: f32 = 1.0;

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
