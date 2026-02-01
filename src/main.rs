//! ARPG Minimalista
//!
//! Controles:
//! - WASD: Mover
//! - Mouse: Mirar
//! - Click Esquerdo: Projétil
//! - Click Direito: Ataque melee (área)
//! - Q: Dash
//! - Space: Fire Nova
//! - Tab: Mostrar/esconder stats
//! - R: Reiniciar

mod components;
mod events;
mod resources;
mod systems;

use bevy::{prelude::*, window::PrimaryWindow};
use components::*;
use events::*;
use resources::*;
use systems::{combat::*, enemy::*, pets::*, player::*, ui::*, world::*};

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
        .add_systems(Startup, setup)
        // Sistemas de Seleção
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
                update_cursor_world_pos,
                player_movement,
                player_attack,
                player_skills,
                update_dash,
                update_invulnerability,
                regen_health,
                update_projectiles,
                update_melee_attacks,
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
        .add_systems(Update, restart_game)
        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // UI - Cooldowns
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

    // UI - Stats
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

    // Instruções
    commands.spawn(
        TextBundle::from_section(
            "WASD: Move | LMB: Shoot | RMB: Melee | Q: Dash | Space: Nova | Tab: Stats",
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
}
