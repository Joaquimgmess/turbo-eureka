use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::*;
use crate::resources::*;
use crate::systems::passive_ui::*;

pub struct PassiveTreePlugin;

impl Plugin for PassiveTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_passive_tree_data)
            .add_systems(OnEnter(GameState::PassiveTree), setup_passive_ui)
            .add_systems(
                Update,
                (
                    toggle_passive_ui,
                    handle_passive_tree_controls,
                    track_hovered_node,
                    handle_node_click,
                    update_passive_ui,
                )
                    .chain()
                    .run_if(in_state(GameState::PassiveTree)),
            )
            .add_systems(OnExit(GameState::PassiveTree), despawn_passive_ui)
            .add_systems(
                Update,
                toggle_passive_ui.run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_passive_tree_data(mut commands: Commands) {
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
            effect: PassiveEffect::StatAdd(Stats { ..zero_stats }),
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

    connections.extend([
        (0, 200),
        (200, 201),
        (200, 202),
        (201, 203),
        (200, 204),
        (204, 205),
        (200, 8),
        (202, 10),
        (201, 9),
        (0, 100),
        (100, 101),
        (100, 102),
        (102, 104),
        (100, 105),
        (105, 106),
        (105, 107),
        (201, 11),
        (11, 12),
        (102, 14),
        (14, 15),
        (202, 17),
        (17, 18),
    ]);

    commands.insert_resource(PassiveTree { nodes, connections });
}
