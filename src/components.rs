use bevy::prelude::*;
use std::collections::HashSet;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    CharacterSelection,
    PetSelection,
    Playing,
    PassiveTree,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerClass {
    Tank,
    Archer,
    Mage,
    Tamer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetType {
    Healer,
    Damager,
    Buffer,
    Tanker,
}

#[derive(Component)]
pub struct Player {
    pub class: PlayerClass,
}

#[derive(Component)]
pub struct TamerData {
    pub selected_pets: Vec<PetType>,
}

#[derive(Component)]
pub struct Enemy {
    pub damage: f32,
    pub xp_value: u32,
    pub attack_cooldown: Timer,
    pub speed: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Default)]
pub struct Shield {
    pub amount: f32,
}

#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct Stats {
    pub speed: f32,
    pub damage: f32,
    pub attack_speed: f32,
    pub crit_chance: f32,
    pub crit_multiplier: f32,
    pub life_regen: f32,
    pub armor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PassiveEffect {
    StatAdd(Stats),
    Ricochet,
    Explosion,
    Knockback,
    ChanceFire(f32),
    ChanceIce(f32),
    ChanceLightning(f32),
    MasteryFire,
    MasteryIce,
    MasteryLightning,
    StatMult(Stats),
    ShieldRegen(f32),
    LifeLeech(f32),
    ShieldLeech(f32),
}

#[derive(Component, Default)]
pub struct ElementalStatus {
    pub fire_stacks: u32,
    pub ice_stacks: u32,
    pub lightning_stacks: u32,
    pub is_ignited: bool,
    pub is_frozen: bool,
    pub is_discharged: bool,
}

#[derive(Component)]
pub struct Loot;

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct Hazard {
    pub damage: f32,
    pub effect: Option<PassiveEffect>,
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct MinimapUi;

#[derive(Component)]
pub struct MinimapPlayerIcon;

#[derive(Component)]
pub struct MinimapEnemyIcon(pub Entity);

#[derive(Component)]
pub struct MinimapIcon {
    pub entity: Entity,
    pub icon_type: IconType,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum IconType {
    Player,
    Enemy,
    Boss,
}

#[derive(Debug, Clone)]
pub struct PassiveNode {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub effect: PassiveEffect,
    pub requirements: Vec<u32>,
    pub position: Vec2,
}

#[derive(Component, Default)]
pub struct PlayerPassives {
    pub unlocked_nodes: Vec<u32>,
    pub points: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            speed: 200.0,
            damage: 25.0,
            attack_speed: 1.0,
            crit_chance: 0.1,
            crit_multiplier: 2.0,
            life_regen: 2.0,
            armor: 0.0,
        }
    }
}

#[derive(Component)]
pub struct Level {
    pub level: u32,
    pub xp: u32,
    pub xp_to_next: u32,
}

impl Level {
    pub fn new() -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_to_next: 100,
        }
    }

    pub fn add_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        if self.xp >= self.xp_to_next {
            self.xp -= self.xp_to_next;
            self.level += 1;
            self.xp_to_next = (self.xp_to_next as f32 * 1.4) as u32;
            true
        } else {
            false
        }
    }
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub owner: Entity,
    pub pierce: u32,
    pub chain_count: u32,
    pub hit_entities: HashSet<Entity>,
    pub is_crit: bool,
}

#[derive(Component)]
pub struct MeleeAttack {
    pub damage: f32,
    pub owner: Entity,
    pub duration: Timer,
    pub hit_entities: HashSet<Entity>,
    pub is_crit: bool,
}

#[derive(Component)]
pub struct AoeEffect {
    pub damage: f32,
    pub owner: Entity,
    pub tick_timer: Timer,
    pub duration: Timer,
    pub hit_this_tick: HashSet<Entity>,
}

#[derive(Component)]
pub struct Lifetime(pub Timer);

#[derive(Component)]
pub struct XpOrb {
    pub value: u32,
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarFill(pub f32);

#[derive(Component)]
pub struct Dash {
    pub direction: Vec2,
    pub speed: f32,
    pub duration: Timer,
}

#[derive(Component)]
pub struct Invulnerable(pub Timer);

#[derive(Component)]
pub struct AttackCooldown(pub Timer);

#[derive(Component)]
pub struct SkillCooldowns {
    pub dash: Timer,
    pub nova: Timer,
}

#[derive(Component)]
pub struct DamageNumber {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct CooldownUi;

#[derive(Component)]
pub struct StatsUi;

#[derive(Component)]
pub struct GameOverUi;

#[derive(Component)]
pub struct SelectionUi;

#[derive(Component)]
pub struct PassiveUi;

#[derive(Component)]
pub struct ClassButton(pub PlayerClass);

#[derive(Component)]
pub struct PetButton(pub PetType);

#[derive(Component)]
pub struct Pet {
    pub owner: Entity,
    pub pet_type: PetType,
    pub action_timer: Timer,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterState {
    #[default]
    Idle,
    Walking,
    Attacking,
    Death,
}

#[derive(Component)]
pub struct AnimationConfig {
    pub timer: Timer,
    pub frame_count: usize,
    pub state: CharacterState,
}

#[derive(Component)]
pub struct Taunt;
