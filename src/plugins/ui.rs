use bevy::prelude::*;

use crate::components::*;
use crate::systems::combat::update_damage_numbers;
use crate::systems::ui::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(
            Update,
            (
                update_health_bars,
                update_cooldown_ui,
                update_stats_ui,
                update_damage_numbers,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_ui(mut commands: Commands) {
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
}
