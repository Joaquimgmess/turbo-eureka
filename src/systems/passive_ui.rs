use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct PassiveNodeButton(pub u32);

#[derive(Component)]
pub struct PassivePointsText;

#[derive(Component)]
pub struct PassiveConnection(pub u32, pub u32);

pub fn toggle_passive_ui(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::PassiveTree),
            GameState::PassiveTree => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

pub fn setup_passive_ui(
    mut commands: Commands,
    passive_tree: Res<PassiveTree>,
    player_query: Query<&PlayerPassives, With<Player>>,
) {
    let player_passives = player_query.get_single().expect("Player should exist");
    let points = player_passives.points;

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
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.9).into(),
                ..default()
            },
            PassiveUi,
        ))
        .with_children(|parent| {
            // Header
            parent.spawn(TextBundle::from_section(
                "PASSIVE TREE",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    format!("Points Available: {}", points),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::srgb(1.0, 0.9, 0.3),
                        ..default()
                    },
                ),
                PassivePointsText,
            ));

            // Map Container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(800.0),
                        height: Val::Px(600.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    background_color: Color::srgba(0.1, 0.1, 0.1, 0.5).into(),
                    ..default()
                })
                .with_children(|map| {
                    // Render connections
                    for &(id1, id2) in &passive_tree.connections {
                        if let (Some(node1), Some(node2)) =
                            (passive_tree.nodes.get(&id1), passive_tree.nodes.get(&id2))
                        {
                            let pos1 = node1.position;
                            let pos2 = node2.position;
                            let mid = (pos1 + pos2) / 2.0;
                            let diff = pos2 - pos1;
                            let length = diff.length();
                            let angle = diff.y.atan2(diff.x);

                            map.spawn((
                                NodeBundle {
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        width: Val::Px(length),
                                        height: Val::Px(2.0),
                                        left: Val::Px(mid.x + 400.0 - length / 2.0),
                                        top: Val::Px(-mid.y + 300.0 - 1.0),
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.3, 0.3, 0.3).into(),
                                    transform: Transform::from_rotation(Quat::from_rotation_z(
                                        angle,
                                    )),
                                    ..default()
                                },
                                PassiveConnection(id1, id2),
                            ));
                        }
                    }

                    // Render nodes
                    for (&id, node) in &passive_tree.nodes {
                        map.spawn((
                            ButtonBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(30.0),
                                    height: Val::Px(30.0),
                                    left: Val::Px(node.position.x + 400.0 - 15.0),
                                    top: Val::Px(-node.position.y + 300.0 - 15.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                                ..default()
                            },
                            PassiveNodeButton(id),
                        ));
                    }
                });

            parent.spawn(TextBundle::from_section(
                "Press P to Close",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ));
        });
}

pub fn update_passive_ui(
    player_query: Query<&PlayerPassives, With<Player>>,
    passive_tree: Res<PassiveTree>,
    mut points_text: Query<&mut Text, With<PassivePointsText>>,
    mut node_buttons: Query<
        (
            &PassiveNodeButton,
            &mut BackgroundColor,
            &mut BorderColor,
            &Interaction,
        ),
        With<Button>,
    >,
    mut connections: Query<(&PassiveConnection, &mut BackgroundColor), Without<Button>>,
) {
    let Ok(passives) = player_query.get_single() else {
        return;
    };

    if let Ok(mut text) = points_text.get_single_mut() {
        text.sections[0].value = format!("Points Available: {}", passives.points);
    }

    for (node_btn, mut bg_color, mut border_color, interaction) in node_buttons.iter_mut() {
        let id = node_btn.0;
        let is_unlocked = passives.unlocked_nodes.contains(&id);
        let node = passive_tree.nodes.get(&id).unwrap();
        let can_unlock = passives.points > 0
            && !is_unlocked
            && (id == 0
                || node
                    .requirements
                    .iter()
                    .any(|req| passives.unlocked_nodes.contains(req)));

        if is_unlocked {
            *bg_color = Color::srgb(0.8, 0.6, 0.0).into();
            *border_color = Color::srgb(1.0, 0.8, 0.0).into();
        } else {
            match *interaction {
                Interaction::Hovered => {
                    if can_unlock {
                        *bg_color = Color::srgb(0.3, 0.5, 0.3).into();
                    } else {
                        *bg_color = Color::srgb(0.3, 0.3, 0.3).into();
                    }
                }
                _ => {
                    *bg_color = Color::srgb(0.15, 0.15, 0.15).into();
                }
            }

            if can_unlock {
                *border_color = Color::srgb(0.5, 1.0, 0.5).into();
            } else {
                *border_color = Color::srgb(0.4, 0.4, 0.4).into();
            }
        }
    }

    for (conn, mut bg_color) in connections.iter_mut() {
        let is_unlocked =
            passives.unlocked_nodes.contains(&conn.0) && passives.unlocked_nodes.contains(&conn.1);
        *bg_color = if is_unlocked {
            Color::srgb(1.0, 0.8, 0.0).into()
        } else {
            Color::srgb(0.3, 0.3, 0.3).into()
        };
    }
}

pub fn handle_node_click(
    mut player_query: Query<(&mut PlayerPassives, &mut Stats), With<Player>>,
    passive_tree: Res<PassiveTree>,
    mut interaction_query: Query<
        (&Interaction, &PassiveNodeButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let Ok((mut passives, mut stats)) = player_query.get_single_mut() else {
        return;
    };

    for (interaction, node_btn) in interaction_query.iter_mut() {
        let node_id = node_btn.0;
        let is_unlocked = passives.unlocked_nodes.contains(&node_id);

        let node = passive_tree.nodes.get(&node_id).unwrap();
        let can_unlock = passives.points > 0
            && !is_unlocked
            && (node_id == 0
                || node
                    .requirements
                    .iter()
                    .any(|req| passives.unlocked_nodes.contains(req)));

        if *interaction == Interaction::Pressed && can_unlock {
            passives.unlocked_nodes.push(node_id);
            passives.points -= 1;

            match node.effect {
                PassiveEffect::StatAdd(s) => {
                    stats.damage += s.damage;
                    stats.speed += s.speed;
                    stats.attack_speed += s.attack_speed;
                    stats.crit_chance += s.crit_chance;
                    stats.crit_multiplier += s.crit_multiplier;
                    stats.life_regen += s.life_regen;
                    stats.armor += s.armor;
                }
                _ => {}
            }
        }
    }
}

pub fn despawn_passive_ui(mut commands: Commands, query: Query<Entity, With<PassiveUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
