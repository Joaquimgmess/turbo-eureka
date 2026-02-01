use crate::components::*;
use crate::resources::*;
use crate::systems::pets::spawn_pet;
use crate::systems::player::spawn_player;
use bevy::prelude::*;

pub fn update_health_bars(
    parents: Query<&Health>,
    mut health_bars: Query<(&Parent, &mut Sprite, &mut Transform, &HealthBarFill)>,
) {
    for (parent, mut sprite, mut transform, fill) in health_bars.iter_mut() {
        let Ok(health) = parents.get(parent.get()) else {
            continue;
        };

        let percent = (health.current / health.max).clamp(0.0, 1.0);

        if let Some(ref mut size) = sprite.custom_size {
            let full_width = fill.0;
            let new_width = full_width * percent;
            let offset = (full_width - new_width) / 2.0;
            transform.translation.x = -offset;
            size.x = new_width.max(0.1);
        }
    }
}

pub fn update_cooldown_ui(
    player: Query<(&SkillCooldowns, &Level, &Health, &Shield), With<Player>>,
    mut ui: Query<&mut Text, With<CooldownUi>>,
    mut game_stats: ResMut<GameStats>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((cooldowns, level, health, shield)) = player.get_single() else {
        return;
    };
    let Ok(mut text) = ui.get_single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Tab) {
        game_stats.show_stats = !game_stats.show_stats;
    }

    text.sections[1].value = if cooldowns.dash.finished() {
        "[Q] Dash: Ready\n".to_string()
    } else {
        format!("[Q] Dash: {:.1}s\n", cooldowns.dash.remaining_secs())
    };

    text.sections[2].value = if cooldowns.nova.finished() {
        "[Space] Nova: Ready\n".to_string()
    } else {
        format!("[Space] Nova: {:.1}s\n", cooldowns.nova.remaining_secs())
    };

    let shield_text = if shield.amount > 0.0 {
        format!(" | SHIELD: {:.0}", shield.amount)
    } else {
        "".to_string()
    };

    text.sections[3].value = format!(
        "\nLevel: {} | HP: {:.0}/{:.0}{}\n",
        level.level, health.current, health.max, shield_text
    );
    text.sections[4].value = format!("XP: {}/{}", level.xp, level.xp_to_next);
}

pub fn update_stats_ui(
    player: Query<&Stats, With<Player>>,
    game_stats: Res<GameStats>,
    mut ui: Query<&mut Text, With<StatsUi>>,
) {
    let Ok(stats) = player.get_single() else {
        return;
    };
    let Ok(mut text) = ui.get_single_mut() else {
        return;
    };

    if game_stats.show_stats {
        text.sections[1].value = format!(
            "Damage: {:.1}\n\
             Atk Speed: {:.2}x\n\
             Crit: {:.0}%\n\
             Crit Multi: {:.1}x\n\
             Speed: {:.0}\n\
             Armor: {:.0}\n\
             Regen: {:.1}/s\n\
             \n-- Session --\n\
             Kills: {}\n\
             Dmg: {:.0}\n\
             Time: {:.0}s",
            stats.damage,
            stats.attack_speed,
            stats.crit_chance * 100.0,
            stats.crit_multiplier,
            stats.speed,
            stats.armor,
            stats.life_regen,
            game_stats.enemies_killed,
            game_stats.damage_dealt,
            game_stats.time_survived,
        );
    } else {
        text.sections[1].value = "(Tab)".to_string();
    }
}

pub fn show_game_over(mut commands: Commands, game_stats: Res<GameStats>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "GAME OVER\n\n",
                TextStyle {
                    font_size: 64.0,
                    color: Color::srgb(0.95, 0.15, 0.15),
                    ..default()
                },
            ),
            TextSection::new(
                format!(
                    "Enemies Killed: {}\n\
                     Damage Dealt: {:.0}\n\
                     Time Survived: {:.0}s\n\n\
                     Press R to restart",
                    game_stats.enemies_killed, game_stats.damage_dealt, game_stats.time_survived,
                ),
                TextStyle {
                    font_size: 26.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(28.0),
            left: Val::Percent(32.0),
            ..default()
        }),
        GameOverUi,
    ));
}

pub fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    _current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    despawn_query: Query<
        Entity,
        Or<(
            With<Player>,
            With<Enemy>,
            With<Projectile>,
            With<XpOrb>,
            With<AoeEffect>,
            With<MeleeAttack>,
            With<DamageNumber>,
            With<GameOverUi>,
            With<Pet>,
        )>,
    >,
    mut game_stats: ResMut<GameStats>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for entity in despawn_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        *game_stats = GameStats::default();

        next_state.set(GameState::CharacterSelection);
    }
}

pub fn setup_class_selection(mut commands: Commands) {
    commands
        .spawn((
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
            SelectionUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "ESCOLHA SUA CLASSE",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            let classes = [
                (PlayerClass::Tank, "TANK (Defesa/AoE)"),
                (PlayerClass::Archer, "ARCHER (Velocidade/Range)"),
                (PlayerClass::Mage, "MAGE (Alto Dano/CD)"),
                (PlayerClass::Tamer, "TAMER (Domador de Pets)"),
            ];

            for (class, label) in classes {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(300.0),
                                height: Val::Px(50.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                            ..default()
                        },
                        ClassButton(class),
                    ))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

pub fn handle_class_selection(
    mut next_state: ResMut<NextState<GameState>>,
    mut pending: ResMut<PendingSelection>,
    mut interaction_query: Query<
        (&Interaction, &ClassButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, class_btn, mut color) in interaction_query.iter_mut() {
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
                *color = Color::srgb(0.4, 0.4, 0.4).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}

pub fn setup_pet_selection(mut commands: Commands) {
    commands
        .spawn((
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
            SelectionUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "ESCOLHA 2 PETS",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            let pets = [
                (PetType::Healer, "CURA (Regen HP)"),
                (PetType::Damager, "DANO (Ataca Inimigos)"),
                (PetType::Buffer, "BUFF (Escudo/Dano)"),
                (PetType::Tanker, "TANK (Atrai Agro)"),
            ];

            for (pet, label) in pets {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(300.0),
                                height: Val::Px(50.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                            ..default()
                        },
                        PetButton(pet),
                    ))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

pub fn handle_pet_selection(
    mut next_state: ResMut<NextState<GameState>>,
    mut pending: ResMut<PendingSelection>,
    mut interaction_query: Query<
        (&Interaction, &PetButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, pet_btn, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if !pending.pets.contains(&pet_btn.0) {
                    pending.pets.push(pet_btn.0);
                    *color = Color::srgb(0.2, 0.6, 0.2).into();
                }

                if pending.pets.len() >= 2 {
                    next_state.set(GameState::Playing);
                }
            }
            Interaction::Hovered => {
                if !pending.pets.contains(&pet_btn.0) {
                    *color = Color::srgb(0.4, 0.4, 0.4).into();
                }
            }
            Interaction::None => {
                if !pending.pets.contains(&pet_btn.0) {
                    *color = Color::srgb(0.2, 0.2, 0.2).into();
                }
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
    sprites: Res<CharacterSprites>,
    mut pending: ResMut<PendingSelection>,
    existing_player: Query<Entity, With<Player>>,
) {

    if existing_player.get_single().is_ok() {
        return;
    }

    let class = pending.class.unwrap_or(PlayerClass::Tank);
    let player_entity = spawn_player(&mut commands, &sprites, Vec3::ZERO, class);

    if class == PlayerClass::Tamer {
        for pet_type in pending.pets.iter() {
            spawn_pet(&mut commands, player_entity, *pet_type);
        }
    }

    *pending = PendingSelection::default();
}
