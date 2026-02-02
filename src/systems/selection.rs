use bevy::prelude::*;

use crate::components::*;
use crate::plugins::ui::HudRoot;
use crate::resources::*;
use crate::systems::pets::spawn_pet;
use crate::systems::player::spawn_player;

pub fn setup_class_selection(mut commands: Commands) {
    commands
        .spawn((
            SelectionUi,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Choose Your Class",
                TextStyle {
                    font_size: 48.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::top(Val::Px(40.0)),
                        column_gap: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    for (class, name, color) in [
                        (PlayerClass::Tank, "Tank", Color::srgb(0.3, 0.5, 0.9)),
                        (PlayerClass::Archer, "Archer", Color::srgb(0.8, 0.7, 0.2)),
                        (PlayerClass::Mage, "Mage", Color::srgb(0.6, 0.2, 0.8)),
                        (PlayerClass::Tamer, "Tamer", Color::srgb(0.2, 0.8, 0.3)),
                    ] {
                        row.spawn((
                            ClassButton(class),
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(150.0),
                                    height: Val::Px(60.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: color.into(),
                                ..default()
                            },
                        ))
                        .with_children(|btn| {
                            btn.spawn(TextBundle::from_section(
                                name,
                                TextStyle {
                                    font_size: 24.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                        });
                    }
                });
        });
}

pub fn handle_class_selection(
    mut interaction_query: Query<
        (&Interaction, &ClassButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut pending: ResMut<PendingSelection>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, class_btn, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                pending.class = Some(class_btn.0);
                if class_btn.0 == PlayerClass::Tamer {
                    next_state.set(GameState::PetSelection);
                } else {
                    next_state.set(GameState::Playing);
                }
            }
            Interaction::Hovered => {
                *bg = Color::srgb(0.9, 0.9, 0.9).into();
            }
            Interaction::None => {
                *bg = match class_btn.0 {
                    PlayerClass::Tank => Color::srgb(0.3, 0.5, 0.9),
                    PlayerClass::Archer => Color::srgb(0.8, 0.7, 0.2),
                    PlayerClass::Mage => Color::srgb(0.6, 0.2, 0.8),
                    PlayerClass::Tamer => Color::srgb(0.2, 0.8, 0.3),
                }
                .into();
            }
        }
    }
}

pub fn setup_pet_selection(mut commands: Commands) {
    commands
        .spawn((
            SelectionUi,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Choose Your Pets (2)",
                TextStyle {
                    font_size: 48.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::top(Val::Px(40.0)),
                        column_gap: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    for (pet_type, name, color) in [
                        (PetType::Healer, "Healer", Color::srgb(0.2, 0.9, 0.4)),
                        (PetType::Damager, "Damager", Color::srgb(0.9, 0.3, 0.3)),
                        (PetType::Buffer, "Buffer", Color::srgb(0.9, 0.8, 0.2)),
                        (PetType::Tanker, "Tanker", Color::srgb(0.3, 0.5, 0.9)),
                    ] {
                        row.spawn((
                            PetButton(pet_type),
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(120.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: color.into(),
                                ..default()
                            },
                        ))
                        .with_children(|btn| {
                            btn.spawn(TextBundle::from_section(
                                name,
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                        });
                    }
                });
        });
}

pub fn handle_pet_selection(
    mut interaction_query: Query<
        (&Interaction, &PetButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut pending: ResMut<PendingSelection>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, pet_btn, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if !pending.pets.contains(&pet_btn.0) {
                    pending.pets.push(pet_btn.0);
                    if pending.pets.len() >= 2 {
                        next_state.set(GameState::Playing);
                    }
                }
            }
            Interaction::Hovered => {
                *bg = Color::srgb(0.9, 0.9, 0.9).into();
            }
            Interaction::None => {
                *bg = match pet_btn.0 {
                    PetType::Healer => Color::srgb(0.2, 0.9, 0.4),
                    PetType::Damager => Color::srgb(0.9, 0.3, 0.3),
                    PetType::Buffer => Color::srgb(0.9, 0.8, 0.2),
                    PetType::Tanker => Color::srgb(0.3, 0.5, 0.9),
                }
                .into();
            }
        }
    }
}

pub fn despawn_selection_ui(mut commands: Commands, query: Query<Entity, With<SelectionUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn start_game(
    mut commands: Commands,
    pending: Res<PendingSelection>,
    sprites: Res<CharacterSprites>,
    player_query: Query<Entity, With<Player>>,
) {
    if player_query.get_single().is_ok() {
        return;
    }

    let class = pending.class.unwrap_or(PlayerClass::Archer);
    let player_entity = spawn_player(&mut commands, &sprites, Vec3::ZERO, class);

    if class == PlayerClass::Tamer {
        for (i, &pet_type) in pending.pets.iter().enumerate() {
            let offset = Vec2::new((i as f32 - 0.5) * 80.0, -60.0);
            spawn_pet(&mut commands, player_entity, pet_type, offset);
        }
        commands.entity(player_entity).insert(TamerData {
            selected_pets: pending.pets.clone(),
        });
    }
}

pub fn show_game_over(mut commands: Commands, game_stats: Res<GameStats>) {
    commands.spawn((
        GameOverUi,
        TextBundle::from_sections([
            TextSection::new(
                "GAME OVER\n\n",
                TextStyle {
                    font_size: 64.0,
                    color: Color::srgb(0.9, 0.2, 0.2),
                    ..default()
                },
            ),
            TextSection::new(
                format!(
                    "Time: {:.1}s\nEnemies: {}\nDamage: {:.0}\n\n",
                    game_stats.time_survived, game_stats.enemies_killed, game_stats.damage_dealt
                ),
                TextStyle {
                    font_size: 28.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "Press R to Restart",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(35.0),
            ..default()
        }),
    ));
}

pub fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_stats: ResMut<GameStats>,
    mut pending: ResMut<PendingSelection>,
    all_entities: Query<Entity, (Without<Camera2d>, Without<Window>, Without<HudRoot>)>,
    game_over_ui: Query<Entity, With<GameOverUi>>,
) {
    if *state.get() == GameState::GameOver && keyboard.just_pressed(KeyCode::KeyR) {
        *game_stats = GameStats::default();
        *pending = PendingSelection::default();

        for entity in game_over_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for entity in all_entities.iter() {
            commands.entity(entity).despawn_recursive();
        }

        next_state.set(GameState::CharacterSelection);
    }
}
