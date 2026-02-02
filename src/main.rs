mod components;
mod events;
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
        .add_systems(OnEnter(GameState::Playing), start_game)
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
            name: "Start".to_string(),
            description: "+5 Damage".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                damage: 5.0,
                ..zero_stats
            }),
            requirements: vec![],
            position: Vec2::ZERO,
        },
    );

    nodes.insert(
        1,
        PassiveNode {
            id: 1,
            name: "Titan I".to_string(),
            description: "+5 Armor".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                armor: 5.0,
                ..zero_stats
            }),
            requirements: vec![0],
            position: Vec2::new(-150.0, 0.0),
        },
    );
    nodes.insert(
        2,
        PassiveNode {
            id: 2,
            name: "Titan II".to_string(),
            description: "+10 Armor".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                armor: 10.0,
                ..zero_stats
            }),
            requirements: vec![1],
            position: Vec2::new(-250.0, 50.0),
        },
    );
    nodes.insert(
        3,
        PassiveNode {
            id: 3,
            name: "Titan III".to_string(),
            description: "+15 Armor".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                armor: 15.0,
                ..zero_stats
            }),
            requirements: vec![1],
            position: Vec2::new(-250.0, -50.0),
        },
    );
    connections.push((0, 1));
    connections.push((1, 2));
    connections.push((1, 3));

    nodes.insert(
        4,
        PassiveNode {
            id: 4,
            name: "Falcon I".to_string(),
            description: "+10% Attack Speed".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                attack_speed: 0.1,
                ..zero_stats
            }),
            requirements: vec![0],
            position: Vec2::new(0.0, 150.0),
        },
    );
    nodes.insert(
        5,
        PassiveNode {
            id: 5,
            name: "Falcon II".to_string(),
            description: "+5% Crit Chance".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                crit_chance: 0.05,
                ..zero_stats
            }),
            requirements: vec![4],
            position: Vec2::new(-50.0, 250.0),
        },
    );
    nodes.insert(
        6,
        PassiveNode {
            id: 6,
            name: "Falcon III".to_string(),
            description: "+50% Crit Multiplier".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                crit_multiplier: 0.5,
                ..zero_stats
            }),
            requirements: vec![4],
            position: Vec2::new(50.0, 250.0),
        },
    );
    nodes.insert(
        10,
        PassiveNode {
            id: 10,
            name: "Ricochet".to_string(),
            description: "Projectiles bounce once".to_string(),
            effect: PassiveEffect::Ricochet,
            requirements: vec![4],
            position: Vec2::new(0.0, 300.0),
        },
    );
    connections.push((0, 4));
    connections.push((4, 5));
    connections.push((4, 6));
    connections.push((4, 10));

    nodes.insert(
        7,
        PassiveNode {
            id: 7,
            name: "Arcanist I".to_string(),
            description: "+10 Damage".to_string(),
            effect: PassiveEffect::StatAdd(Stats {
                damage: 10.0,
                ..zero_stats
            }),
            requirements: vec![0],
            position: Vec2::new(150.0, 0.0),
        },
    );
    nodes.insert(
        8,
        PassiveNode {
            id: 8,
            name: "Arcanist II".to_string(),
            description: "Adds knockback to attacks".to_string(),
            effect: PassiveEffect::Knockback,
            requirements: vec![7],
            position: Vec2::new(250.0, 50.0),
        },
    );
    nodes.insert(
        9,
        PassiveNode {
            id: 9,
            name: "Arcanist III".to_string(),
            description: "Enemies explode on death".to_string(),
            effect: PassiveEffect::Explosion,
            requirements: vec![7],
            position: Vec2::new(250.0, -50.0),
        },
    );
    connections.push((0, 7));
    connections.push((7, 8));
    connections.push((7, 9));

    // Elemental Branches
    // Fire Branch (Lower-Left)
    nodes.insert(
        11,
        PassiveNode {
            id: 11,
            name: "Pyromancy I".to_string(),
            description: "15% chance to Burn on hit".to_string(),
            effect: PassiveEffect::ChanceFire(0.15),
            requirements: vec![1],
            position: Vec2::new(-200.0, -200.0),
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
            position: Vec2::new(-300.0, -250.0),
        },
    );
    connections.push((1, 11));
    connections.push((11, 12));

    // Ice Branch (Lower-Right)
    nodes.insert(
        14,
        PassiveNode {
            id: 14,
            name: "Cryomancy I".to_string(),
            description: "20% chance to Chill on hit".to_string(),
            effect: PassiveEffect::ChanceIce(0.20),
            requirements: vec![7],
            position: Vec2::new(200.0, -200.0),
        },
    );
    nodes.insert(
        15,
        PassiveNode {
            id: 15,
            name: "Shatter".to_string(),
            description: "Max stacks freeze & explode".to_string(),
            effect: PassiveEffect::MasteryIce,
            requirements: vec![14],
            position: Vec2::new(300.0, -250.0),
        },
    );
    connections.push((7, 14));
    connections.push((14, 15));

    // Lightning Branch (Upper-Right)
    nodes.insert(
        17,
        PassiveNode {
            id: 17,
            name: "Electromancy I".to_string(),
            description: "10% chance to Shock on hit".to_string(),
            effect: PassiveEffect::ChanceLightning(0.10),
            requirements: vec![4],
            position: Vec2::new(200.0, 200.0),
        },
    );
    nodes.insert(
        18,
        PassiveNode {
            id: 18,
            name: "Chain Discharge".to_string(),
            description: "Max stacks chain to nearby".to_string(),
            effect: PassiveEffect::MasteryLightning,
            requirements: vec![17],
            position: Vec2::new(300.0, 250.0),
        },
    );
    connections.push((4, 17));
    connections.push((17, 18));

    commands.insert_resource(PassiveTree { nodes, connections });
}
