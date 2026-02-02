mod components;
mod constants;
mod events;
mod helpers;
mod plugins;
mod resources;
mod systems;

use bevy::prelude::*;
use components::*;
use events::*;
use resources::*;
use std::collections::HashMap;
use systems::{
    animation::*, combat::*, enemy::*, passive_ui::*, pets::*, player::*, ui::*, world::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ARPG Minimal".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
        .insert_resource(CursorWorldPos(Vec2::ZERO))
        .insert_resource(GameStats::default())
        .insert_resource(PendingSelection::default())
        .insert_resource(MapTier(1))
        .insert_resource(MapData::default())
        .add_event::<DamageEvent>()
        .add_event::<SpawnXpOrbEvent>()
        .add_event::<ApplyStatusEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GameState::CharacterSelection),
            setup_class_selection,
        )
        .add_systems(
            Update,
            handle_class_selection.run_if(in_state(GameState::CharacterSelection)),
        )
        .add_systems(OnExit(GameState::CharacterSelection), despawn_selection_ui)
        .add_systems(OnEnter(GameState::PetSelection), setup_pet_selection)
        .add_systems(
            Update,
            handle_pet_selection.run_if(in_state(GameState::PetSelection)),
        )
        .add_systems(OnExit(GameState::PetSelection), despawn_selection_ui)
        .add_systems(OnEnter(GameState::Playing), (start_game, setup_minimap))
        .add_systems(
            Update,
            (
                toggle_passive_ui,
                update_cursor_world_pos,
                animate_sprite,
                update_character_animation_texture,
                player_movement,
                player_attack,
                player_skills,
                update_dash,
                update_invulnerability,
                regen_health,
                update_projectiles,
                update_melee_attacks,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                update_aoe_effects,
                enemy_ai,
                enemy_attack,
                process_damage,
                check_player_death,
                check_enemy_death,
                update_xp_orbs,
                collect_xp,
                spawn_enemies,
                update_pets,
                pet_actions,
                update_elemental_statuses,
                handle_status_applications,
                handle_mastery_effects,
                spawn_boss,
                update_hazards,
                handle_loot,
                update_minimap,
                generate_map,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                update_health_bars,
                update_cooldown_ui,
                update_stats_ui,
                update_damage_numbers,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::PassiveTree), setup_passive_ui)
        .add_systems(
            Update,
            (toggle_passive_ui, handle_node_click, update_passive_ui)
                .run_if(in_state(GameState::PassiveTree)),
        )
        .add_systems(OnExit(GameState::PassiveTree), despawn_passive_ui)
        .add_systems(Update, restart_game)
        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
    let layout_handle = texture_atlases.add(layout);
    commands.insert_resource(CharacterSprites {
        orc_idle: asset_server.load("sprites/orc/idle.png"),
        orc_walk: asset_server.load("sprites/orc/walk.png"),
        orc_attack: asset_server.load("sprites/orc/attack1.png"),
        soldier_idle: asset_server.load("sprites/soldier/idle.png"),
        soldier_walk: asset_server.load("sprites/soldier/walk.png"),
        soldier_attack: asset_server.load("sprites/soldier/attack1.png"),
        layout: layout_handle,
    });
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Skills:\n",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "[Q] Dash: Ready\n",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.5, 0.8, 1.0),
                    ..default()
                },
            ),
            TextSection::new(
                "[Space] Nova: Ready\n",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(1.0, 0.5, 0.3),
                    ..default()
                },
            ),
            TextSection::new(
                "\nLevel: 1\n",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(1.0, 0.9, 0.3),
                    ..default()
                },
            ),
            TextSection::new(
                "XP: 0/100",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.8, 0.7, 0.3),
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        CooldownUi,
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Stats (Tab):\n",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        StatsUi,
    ));
    commands.spawn(
        TextBundle::from_section(
            "WASD: Move | LMB: Shoot | RMB: Melee | Q: Dash | Space: Nova | Tab: Stats | P: Passives",
            TextStyle {
                font_size: 14.0,
                color: Color::srgb(0.5, 0.5, 0.5),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
    let mut nodes = HashMap::new();
    let mut connections = Vec::new();
    let zero_stats = Stats {
        speed: 0.0,
        damage: 0.0,
        attack_speed: 0.0,
        crit_chance: 0.0,
        crit_multiplier: 0.0,
        life_regen: 0.0,
        armor: 0.0,
    };
    nodes.insert(
        0,
        PassiveNode {
            id: 0,
            name: "Origin".to_string(),
            description: "+10 Damage".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                damage: 10.0,
                ..zero_stats
            }),
            requirements: vec![],
            position: Vec2::ZERO,
        },
    );
    nodes.insert(
        200,
        PassiveNode {
            id: 200,
            name: "Warrior Soul".to_string(),
            description: "+15 Damage".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                damage: 15.0,
                ..zero_stats
            }),
            requirements: vec![0],
            position: Vec2::new(120.0, 0.0),
        },
    );
    nodes.insert(
        201,
        PassiveNode {
            id: 201,
            name: "Brutality".to_string(),
            description: "25% More Damage".to_string(),
            effect: PassiveEffect::StatMult(Stats {
                damage: 1.25,
                ..zero_stats
            }),
            requirements: vec![200],
            position: Vec2::new(240.0, 60.0),
        },
    );
    nodes.insert(
        202,
        PassiveNode {
            id: 202,
            name: "Precision".to_string(),
            description: "+10% Crit Chance".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                crit_chance: 0.1,
                ..zero_stats
            }),
            requirements: vec![200],
            position: Vec2::new(240.0, -60.0),
        },
    );
    nodes.insert(
        203,
        PassiveNode {
            id: 203,
            name: "Soul Feast".to_string(),
            description: "1.5% Life Leech".to_string(),
            effect: PassiveEffect::LifeLeech(0.015),
            requirements: vec![201],
            position: Vec2::new(360.0, 100.0),
        },
    );
    nodes.insert(
        204,
        PassiveNode {
            id: 204,
            name: "Slaughter".to_string(),
            description: "10% More Damage".to_string(),
            effect: PassiveEffect::StatMult(Stats {
                damage: 1.1,
                ..zero_stats
            }),
            requirements: vec![200],
            position: Vec2::new(120.0, 100.0),
        },
    );
    nodes.insert(
        205,
        PassiveNode {
            id: 205,
            name: "Celerity".to_string(),
            description: "+15% Attack Speed".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                attack_speed: 0.15,
                ..zero_stats
            }),
            requirements: vec![204],
            position: Vec2::new(120.0, 200.0),
        },
    );
    nodes.insert(
        8,
        PassiveNode {
            id: 8,
            name: "Impact".to_string(),
            description: "Add knockback to attacks".to_string(),
            effect: PassiveEffect::Knockback,
            requirements: vec![200],
            position: Vec2::new(0.0, 120.0),
        },
    );
    nodes.insert(
        10,
        PassiveNode {
            id: 10,
            name: "Ricochet".to_string(),
            description: "Projectiles bounce once".to_string(),
            effect: PassiveEffect::Ricochet,
            requirements: vec![202],
            position: Vec2::new(360.0, -100.0),
        },
    );
    nodes.insert(
        9,
        PassiveNode {
            id: 9,
            name: "Combustion Corpse".to_string(),
            description: "Enemies explode on death".to_string(),
            effect: PassiveEffect::Explosion,
            requirements: vec![201],
            position: Vec2::new(360.0, 20.0),
        },
    );
    connections.push((0, 200));
    connections.push((200, 201));
    connections.push((200, 202));
    connections.push((201, 203));
    connections.push((200, 204));
    connections.push((204, 205));
    connections.push((200, 8));
    connections.push((202, 10));
    connections.push((201, 9));
    nodes.insert(
        100,
        PassiveNode {
            id: 100,
            name: "Guardian Core".to_string(),
            description: "+20 Armor".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                armor: 20.0,
                ..zero_stats
            }),
            requirements: vec![0],
            position: Vec2::new(-120.0, 0.0),
        },
    );
    nodes.insert(
        101,
        PassiveNode {
            id: 101,
            name: "Hardened Aegis".to_string(),
            description: "30% More Armor".to_string(),
            effect: PassiveEffect::StatMult(Stats {
                armor: 1.3,
                ..zero_stats
            }),
            requirements: vec![100],
            position: Vec2::new(-240.0, 60.0),
        },
    );
    nodes.insert(
        102,
        PassiveNode {
            id: 102,
            name: "Crystal Skin".to_string(),
            description: "8 Shield Regen/sec".to_string(),
            effect: PassiveEffect::ShieldRegen(8.0),
            requirements: vec![100],
            position: Vec2::new(-240.0, -60.0),
        },
    );
    nodes.insert(
        104,
        PassiveNode {
            id: 104,
            name: "Ethereal Barrier".to_string(),
            description: "3% Shield Leech".to_string(),
            effect: PassiveEffect::ShieldLeech(0.03),
            requirements: vec![102],
            position: Vec2::new(-360.0, -100.0),
        },
    );
    nodes.insert(
        105,
        PassiveNode {
            id: 105,
            name: "Sanctuary".to_string(),
            description: "+20 Shield".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                speed: 0.0,
                damage: 0.0,
                attack_speed: 0.0,
                crit_chance: 0.0,
                crit_multiplier: 0.0,
                life_regen: 0.0,
                armor: 0.0,
            }),
            requirements: vec![100],
            position: Vec2::new(-120.0, -100.0),
        },
    );
    nodes.insert(
        106,
        PassiveNode {
            id: 106,
            name: "Bastion".to_string(),
            description: "15% More Shield".to_string(),
            effect: PassiveEffect::StatMult(Stats {
                armor: 1.15,
                ..zero_stats
            }),
            requirements: vec![105],
            position: Vec2::new(-200.0, -180.0),
        },
    );
    nodes.insert(
        107,
        PassiveNode {
            id: 107,
            name: "Mending".to_string(),
            description: "+5 Life Regen".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                life_regen: 5.0,
                ..zero_stats
            }),
            requirements: vec![105],
            position: Vec2::new(-40.0, -180.0),
        },
    );
    connections.push((0, 100));
    connections.push((100, 101));
    connections.push((100, 102));
    connections.push((102, 104));
    connections.push((100, 105));
    connections.push((105, 106));
    connections.push((105, 107));
    nodes.insert(
        11,
        PassiveNode {
            id: 11,
            name: "Pyromancy".to_string(),
            description: "20% chance to Burn".to_string(),
            effect: PassiveEffect::ChanceFire(0.20),
            requirements: vec![201],
            position: Vec2::new(340.0, 140.0),
        },
    );
    nodes.insert(
        12,
        PassiveNode {
            id: 12,
            name: "Combustion".to_string(),
            description: "Enemies explode at 10 stacks".to_string(),
            effect: PassiveEffect::MasteryFire,
            requirements: vec![11],
            position: Vec2::new(460.0, 180.0),
        },
    );
    connections.push((201, 11));
    connections.push((11, 12));
    nodes.insert(
        14,
        PassiveNode {
            id: 14,
            name: "Cryomancy".to_string(),
            description: "25% chance to Chill".to_string(),
            effect: PassiveEffect::ChanceIce(0.25),
            requirements: vec![102],
            position: Vec2::new(-340.0, 20.0),
        },
    );
    nodes.insert(
        15,
        PassiveNode {
            id: 15,
            name: "Shatter".to_string(),
            description: "Max stacks freeze & burst".to_string(),
            effect: PassiveEffect::MasteryIce,
            requirements: vec![14],
            position: Vec2::new(-460.0, 60.0),
        },
    );
    connections.push((102, 14));
    connections.push((14, 15));
    nodes.insert(
        17,
        PassiveNode {
            id: 17,
            name: "Electromancy".to_string(),
            description: "15% chance to Shock".to_string(),
            effect: PassiveEffect::ChanceLightning(0.15),
            requirements: vec![202],
            position: Vec2::new(340.0, -180.0),
        },
    );
    nodes.insert(
        18,
        PassiveNode {
            id: 18,
            name: "Chain Lightning".to_string(),
            description: "Discharge at 10 stacks".to_string(),
            effect: PassiveEffect::MasteryLightning,
            requirements: vec![17],
            position: Vec2::new(460.0, -220.0),
        },
    );
    connections.push((202, 17));
    connections.push((17, 18));
    commands.insert_resource(PassiveTree { nodes, connections });
}
