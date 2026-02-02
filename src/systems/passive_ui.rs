use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Component)]
pub struct PassiveNodeButton(pub u32);

#[derive(Component)]
pub struct PassivePointsText;

#[derive(Component)]
pub struct PassiveConnection(pub u32, pub u32);

#[derive(Component)]
pub struct PassiveTooltip;

#[derive(Component)]
pub struct PassiveTooltipText;

#[derive(Component)]
pub struct PassiveTreeContainer;

#[derive(Component)]
pub struct CenterViewButton;

#[derive(Component)]
pub struct PathHighlight;

#[derive(Resource)]
pub struct PassiveTreeViewState {
    pub zoom: f32,
    pub pan_offset: Vec2,
    pub dragging: bool,
    pub last_mouse_pos: Vec2,
    pub hovered_node_id: Option<u32>,
}

impl Default for PassiveTreeViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            dragging: false,
            last_mouse_pos: Vec2::ZERO,
            hovered_node_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    Damage,
    Defense,
    Utility,
    Elemental,
    Origin,
}

impl NodeCategory {
    pub fn from_effect(effect: &PassiveEffect) -> Self {
        match effect {
            PassiveEffect::StatAdd(stats) => {
                if stats.damage > 0.0
                    || stats.crit_chance > 0.0
                    || stats.crit_multiplier > 0.0
                    || stats.attack_speed > 0.0
                {
                    NodeCategory::Damage
                } else if stats.armor > 0.0 || stats.life_regen > 0.0 {
                    NodeCategory::Defense
                } else if stats.speed > 0.0 {
                    NodeCategory::Utility
                } else {
                    NodeCategory::Utility
                }
            }
            PassiveEffect::StatMult(stats) => {
                if stats.damage > 1.0
                    || stats.crit_chance > 0.0
                    || stats.crit_multiplier > 0.0
                    || stats.attack_speed > 0.0
                {
                    NodeCategory::Damage
                } else if stats.armor > 1.0 {
                    NodeCategory::Defense
                } else {
                    NodeCategory::Utility
                }
            }
            PassiveEffect::Ricochet | PassiveEffect::Explosion | PassiveEffect::Knockback => {
                NodeCategory::Utility
            }
            PassiveEffect::ChanceFire(_)
            | PassiveEffect::ChanceIce(_)
            | PassiveEffect::ChanceLightning(_)
            | PassiveEffect::MasteryFire
            | PassiveEffect::MasteryIce
            | PassiveEffect::MasteryLightning => NodeCategory::Elemental,
            PassiveEffect::ShieldRegen(_)
            | PassiveEffect::ShieldLeech(_)
            | PassiveEffect::LifeLeech(_) => NodeCategory::Defense,
        }
    }

    pub fn base_color(&self) -> Color {
        match self {
            NodeCategory::Origin => Color::srgb(1.0, 0.9, 0.4),
            NodeCategory::Damage => Color::srgb(0.9, 0.3, 0.2),
            NodeCategory::Defense => Color::srgb(0.2, 0.5, 0.9),
            NodeCategory::Utility => Color::srgb(0.3, 0.8, 0.4),
            NodeCategory::Elemental => Color::srgb(0.8, 0.4, 0.9),
        }
    }

    pub fn glow_color(&self) -> Color {
        match self {
            NodeCategory::Origin => Color::srgb(1.0, 0.95, 0.6),
            NodeCategory::Damage => Color::srgb(1.0, 0.5, 0.3),
            NodeCategory::Defense => Color::srgb(0.4, 0.7, 1.0),
            NodeCategory::Utility => Color::srgb(0.5, 1.0, 0.6),
            NodeCategory::Elemental => Color::srgb(1.0, 0.6, 1.0),
        }
    }

    pub fn locked_color(&self) -> Color {
        match self {
            NodeCategory::Origin => Color::srgb(0.4, 0.35, 0.15),
            NodeCategory::Damage => Color::srgb(0.35, 0.15, 0.1),
            NodeCategory::Defense => Color::srgb(0.1, 0.2, 0.35),
            NodeCategory::Utility => Color::srgb(0.12, 0.3, 0.15),
            NodeCategory::Elemental => Color::srgb(0.3, 0.15, 0.35),
        }
    }
}

pub fn get_node_category(node_id: u32, effect: &PassiveEffect) -> NodeCategory {
    if node_id == 0 {
        NodeCategory::Origin
    } else {
        NodeCategory::from_effect(effect)
    }
}

fn find_path_to_node(target_id: u32, tree: &PassiveTree, unlocked: &[u32]) -> Vec<u32> {
    if unlocked.contains(&target_id) {
        return vec![target_id];
    }

    let mut path = vec![target_id];
    let mut current = target_id;

    while current != 0 {
        if let Some(node) = tree.nodes.get(&current) {
            if let Some(&req) = node.requirements.first() {
                path.push(req);
                current = req;
                if unlocked.contains(&req) {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    path.reverse();
    path
}

fn format_stat_preview(current_stats: &Stats, node: &PassiveNode) -> String {
    match &node.effect {
        PassiveEffect::StatAdd(add) => {
            let mut parts = Vec::new();
            if add.damage > 0.0 {
                parts.push(format!(
                    "Damage: {:.0} -> {:.0}",
                    current_stats.damage,
                    current_stats.damage + add.damage
                ));
            }
            if add.speed > 0.0 {
                parts.push(format!(
                    "Speed: {:.0} -> {:.0}",
                    current_stats.speed,
                    current_stats.speed + add.speed
                ));
            }
            if add.attack_speed > 0.0 {
                parts.push(format!(
                    "Atk Speed: {:.0}% -> {:.0}%",
                    current_stats.attack_speed * 100.0,
                    (current_stats.attack_speed + add.attack_speed) * 100.0
                ));
            }
            if add.crit_chance > 0.0 {
                parts.push(format!(
                    "Crit: {:.0}% -> {:.0}%",
                    current_stats.crit_chance * 100.0,
                    (current_stats.crit_chance + add.crit_chance) * 100.0
                ));
            }
            if add.crit_multiplier > 0.0 {
                parts.push(format!(
                    "Crit Mult: {:.1}x -> {:.1}x",
                    current_stats.crit_multiplier,
                    current_stats.crit_multiplier + add.crit_multiplier
                ));
            }
            if add.life_regen > 0.0 {
                parts.push(format!(
                    "Life Regen: {:.0} -> {:.0}",
                    current_stats.life_regen,
                    current_stats.life_regen + add.life_regen
                ));
            }
            if add.armor > 0.0 {
                parts.push(format!(
                    "Armor: {:.0} -> {:.0}",
                    current_stats.armor,
                    current_stats.armor + add.armor
                ));
            }
            if parts.is_empty() {
                node.description.clone()
            } else {
                parts.join("\n")
            }
        }
        PassiveEffect::StatMult(mult) => {
            let mut parts = Vec::new();
            if mult.damage > 1.0 {
                parts.push(format!(
                    "Damage: {:.0} -> {:.0}",
                    current_stats.damage,
                    current_stats.damage * mult.damage
                ));
            }
            if mult.armor > 1.0 {
                parts.push(format!(
                    "Armor: {:.0} -> {:.0}",
                    current_stats.armor,
                    current_stats.armor * mult.armor
                ));
            }
            if parts.is_empty() {
                node.description.clone()
            } else {
                parts.join("\n")
            }
        }
        PassiveEffect::LifeLeech(pct) => format!("Leech {:.1}% of damage as life", pct * 100.0),
        PassiveEffect::ShieldLeech(pct) => format!("Leech {:.1}% of damage as shield", pct * 100.0),
        PassiveEffect::ShieldRegen(amt) => format!("Regenerate {:.0} shield per second", amt),
        _ => node.description.clone(),
    }
}

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
    let Ok(player_passives) = player_query.get_single() else {
        return;
    };
    let points = player_passives.points;

    commands.insert_resource(PassiveTreeViewState::default());

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                background_color: Color::srgba(0.02, 0.02, 0.05, 0.95).into(),
                ..default()
            },
            PassiveUi,
        ))
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|header| {
                    header.spawn(TextBundle::from_section(
                        "PASSIVE TREE",
                        TextStyle {
                            font_size: 36.0,
                            color: Color::srgb(0.9, 0.85, 0.7),
                            ..default()
                        },
                    ));

                    header.spawn((
                        TextBundle::from_section(
                            format!("Points: {}", points),
                            TextStyle {
                                font_size: 24.0,
                                color: Color::srgb(1.0, 0.9, 0.3),
                                ..default()
                            },
                        ),
                        PassivePointsText,
                    ));

                    header
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                background_color: Color::srgb(0.2, 0.2, 0.25).into(),
                                border_color: Color::srgb(0.4, 0.4, 0.5).into(),
                                ..default()
                            },
                            CenterViewButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn(TextBundle::from_section(
                                "Center View",
                                TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                        });
                });

            parent
                .spawn((NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(20.0),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|legend| {
                    spawn_legend_item(legend, "Damage", NodeCategory::Damage.base_color());
                    spawn_legend_item(legend, "Defense", NodeCategory::Defense.base_color());
                    spawn_legend_item(legend, "Utility", NodeCategory::Utility.base_color());
                    spawn_legend_item(legend, "Elemental", NodeCategory::Elemental.base_color());
                });

            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(900.0),
                            height: Val::Px(500.0),
                            position_type: PositionType::Relative,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        background_color: Color::srgba(0.05, 0.05, 0.08, 0.8).into(),
                        border_color: Color::srgb(0.3, 0.3, 0.35).into(),
                        ..default()
                    },
                    PassiveTreeContainer,
                ))
                .with_children(|map| {
                    for &(id1, id2) in &passive_tree.connections {
                        if let (Some(node1), Some(node2)) =
                            (passive_tree.nodes.get(&id1), passive_tree.nodes.get(&id2))
                        {
                            let pos1 = node1.position;
                            let pos2 = node2.position;
                            let mid = (pos1 + pos2) / 2.0;
                            let diff = pos2 - pos1;
                            let length = diff.length();

                            map.spawn((
                                NodeBundle {
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        width: Val::Px(length),
                                        height: Val::Px(3.0),
                                        left: Val::Px(mid.x + 450.0 - length / 2.0),
                                        top: Val::Px(-mid.y + 250.0 - 1.5),
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.25, 0.25, 0.3).into(),
                                    ..default()
                                },
                                PassiveConnection(id1, id2),
                            ));
                        }
                    }

                    for (&id, node) in &passive_tree.nodes {
                        let category = get_node_category(id, &node.effect);
                        let node_size = if id == 0 { 45.0 } else { 36.0 };

                        map.spawn((
                            ButtonBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(node_size),
                                    height: Val::Px(node_size),
                                    left: Val::Px(node.position.x + 450.0 - node_size / 2.0),
                                    top: Val::Px(-node.position.y + 250.0 - node_size / 2.0),
                                    border: UiRect::all(Val::Px(3.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: category.locked_color().into(),
                                border_color: Color::srgb(0.3, 0.3, 0.35).into(),
                                ..default()
                            },
                            PassiveNodeButton(id),
                        ));
                    }
                });

            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(500.0),
                            min_height: Val::Px(120.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(15.0)),
                            margin: UiRect::top(Val::Px(15.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        background_color: Color::srgba(0.08, 0.08, 0.12, 0.95).into(),
                        border_color: Color::srgb(0.3, 0.3, 0.4).into(),
                        ..default()
                    },
                    PassiveTooltip,
                ))
                .with_children(|tooltip| {
                    tooltip.spawn((
                        TextBundle::from_sections([
                            TextSection::new(
                                "",
                                TextStyle {
                                    font_size: 22.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            TextSection::new(
                                "\n",
                                TextStyle {
                                    font_size: 8.0,
                                    color: Color::srgb(0.5, 0.5, 0.5),
                                    ..default()
                                },
                            ),
                            TextSection::new(
                                "",
                                TextStyle {
                                    font_size: 16.0,
                                    color: Color::srgb(0.7, 0.7, 0.7),
                                    ..default()
                                },
                            ),
                            TextSection::new(
                                "\n",
                                TextStyle {
                                    font_size: 8.0,
                                    color: Color::srgb(0.5, 0.5, 0.5),
                                    ..default()
                                },
                            ),
                            TextSection::new(
                                "",
                                TextStyle {
                                    font_size: 14.0,
                                    color: Color::srgb(0.5, 0.9, 0.5),
                                    ..default()
                                },
                            ),
                        ]),
                        PassiveTooltipText,
                    ));
                });

            parent
                .spawn((NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|controls| {
                    controls.spawn(TextBundle::from_section(
                        "Scroll: Zoom | Drag/WASD: Pan | P: Close",
                        TextStyle {
                            font_size: 14.0,
                            color: Color::srgb(0.5, 0.5, 0.55),
                            ..default()
                        },
                    ));
                });
        });
}

fn spawn_legend_item(parent: &mut ChildBuilder, label: &str, color: Color) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
            ..default()
        })
        .with_children(|item| {
            item.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(14.0),
                    height: Val::Px(14.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: color.into(),
                border_color: Color::srgb(0.5, 0.5, 0.5).into(),
                ..default()
            });
            item.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 13.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ));
        });
}

pub fn handle_passive_tree_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut view_state: ResMut<PassiveTreeViewState>,
    windows: Query<&Window>,
    time: Res<Time>,
    mut center_button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CenterViewButton>),
    >,
) {
    let pan_speed = 300.0 * time.delta_seconds();

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        view_state.pan_offset.y -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        view_state.pan_offset.y += pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        view_state.pan_offset.x += pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        view_state.pan_offset.x -= pan_speed;
    }

    for scroll in scroll_events.read() {
        let zoom_delta = scroll.y * 0.1;
        view_state.zoom = (view_state.zoom + zoom_delta).clamp(0.5, 2.0);
    }

    if let Ok(window) = windows.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            if mouse_button.just_pressed(MouseButton::Middle)
                || (mouse_button.just_pressed(MouseButton::Left)
                    && keyboard.pressed(KeyCode::Space))
            {
                view_state.dragging = true;
                view_state.last_mouse_pos = cursor_pos;
            }

            if view_state.dragging {
                let delta = cursor_pos - view_state.last_mouse_pos;
                view_state.pan_offset += delta;
                view_state.last_mouse_pos = cursor_pos;
            }
        }
    }

    if mouse_button.just_released(MouseButton::Middle)
        || mouse_button.just_released(MouseButton::Left)
        || !keyboard.pressed(KeyCode::Space)
    {
        view_state.dragging = false;
    }

    for (interaction, mut bg_color) in center_button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                view_state.pan_offset = Vec2::ZERO;
                view_state.zoom = 1.0;
                *bg_color = Color::srgb(0.3, 0.3, 0.35).into();
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.25, 0.25, 0.3).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.2, 0.2, 0.25).into();
            }
        }
    }
}

pub fn update_passive_ui(
    player_query: Query<(&PlayerPassives, &Stats), With<Player>>,
    passive_tree: Res<PassiveTree>,
    view_state: Res<PassiveTreeViewState>,
    mut points_text: Query<&mut Text, (With<PassivePointsText>, Without<PassiveTooltipText>)>,
    mut node_buttons: Query<
        (
            &PassiveNodeButton,
            &mut BackgroundColor,
            &mut BorderColor,
            &Interaction,
            &mut Style,
        ),
        With<Button>,
    >,
    mut connections: Query<
        (&PassiveConnection, &mut BackgroundColor, &mut Style),
        (Without<Button>, Without<PassivePointsText>),
    >,
    mut tooltip_text: Query<&mut Text, (With<PassiveTooltipText>, Without<PassivePointsText>)>,
) {
    let Ok((passives, stats)) = player_query.get_single() else {
        return;
    };

    if let Ok(mut text) = points_text.get_single_mut() {
        text.sections[0].value = format!("Points: {}", passives.points);
    }

    let mut hovered_node: Option<&PassiveNode> = None;
    let mut hovered_node_id: Option<u32> = None;
    let path_nodes: HashSet<u32> = if let Some(hovered_id) = view_state.hovered_node_id {
        find_path_to_node(hovered_id, &passive_tree, &passives.unlocked_nodes)
            .into_iter()
            .collect()
    } else {
        HashSet::new()
    };

    let zoom = view_state.zoom;
    let pan = view_state.pan_offset;

    for (node_btn, mut bg_color, mut border_color, interaction, mut style) in
        node_buttons.iter_mut()
    {
        let id = node_btn.0;
        let is_unlocked = passives.unlocked_nodes.contains(&id);
        let node = passive_tree.nodes.get(&id).unwrap();
        let category = get_node_category(id, &node.effect);
        let can_unlock = passives.points > 0
            && !is_unlocked
            && (id == 0
                || node
                    .requirements
                    .iter()
                    .any(|req| passives.unlocked_nodes.contains(req)));

        let node_size = if id == 0 { 45.0 } else { 36.0 };
        let scaled_size = node_size * zoom;
        let base_x = node.position.x + 450.0 - node_size / 2.0;
        let base_y = -node.position.y + 250.0 - node_size / 2.0;

        style.width = Val::Px(scaled_size);
        style.height = Val::Px(scaled_size);
        style.left = Val::Px((base_x - 450.0) * zoom + 450.0 + pan.x);
        style.top = Val::Px((base_y - 250.0) * zoom + 250.0 + pan.y);

        let is_in_path = path_nodes.contains(&id);

        if is_unlocked {
            *bg_color = category.glow_color().into();
            *border_color = Color::srgb(1.0, 0.95, 0.7).into();
        } else if can_unlock {
            let base = category.base_color();
            match *interaction {
                Interaction::Hovered => {
                    *bg_color = base.into();
                }
                _ => {
                    *bg_color = Color::srgba(
                        base.to_srgba().red * 0.6,
                        base.to_srgba().green * 0.6,
                        base.to_srgba().blue * 0.6,
                        1.0,
                    )
                    .into();
                }
            }
            *border_color = Color::srgb(0.5, 1.0, 0.5).into();
        } else {
            let locked = category.locked_color();
            if is_in_path && !passives.unlocked_nodes.contains(&id) {
                *bg_color = Color::srgba(
                    locked.to_srgba().red * 1.5,
                    locked.to_srgba().green * 1.5,
                    locked.to_srgba().blue * 1.5,
                    1.0,
                )
                .into();
                *border_color = Color::srgb(0.6, 0.6, 0.3).into();
            } else {
                *bg_color = locked.into();
                *border_color = Color::srgb(0.25, 0.25, 0.3).into();
            }
        }

        if *interaction == Interaction::Hovered {
            hovered_node = Some(node);
            hovered_node_id = Some(id);
        }
    }

    for (conn, mut bg_color, mut style) in connections.iter_mut() {
        if let (Some(node1), Some(node2)) = (
            passive_tree.nodes.get(&conn.0),
            passive_tree.nodes.get(&conn.1),
        ) {
            let pos1 = node1.position;
            let pos2 = node2.position;
            let mid = (pos1 + pos2) / 2.0;
            let diff = pos2 - pos1;
            let length = diff.length() * zoom;

            let base_x = mid.x + 450.0 - diff.length() / 2.0;
            let base_y = -mid.y + 250.0 - 1.5;

            style.width = Val::Px(length);
            style.left = Val::Px((base_x - 450.0) * zoom + 450.0 + pan.x);
            style.top = Val::Px((base_y - 250.0) * zoom + 250.0 + pan.y);

            let both_unlocked = passives.unlocked_nodes.contains(&conn.0)
                && passives.unlocked_nodes.contains(&conn.1);
            let in_path = path_nodes.contains(&conn.0) && path_nodes.contains(&conn.1);

            if both_unlocked {
                *bg_color = Color::srgb(1.0, 0.85, 0.4).into();
            } else if in_path {
                *bg_color = Color::srgb(0.6, 0.55, 0.3).into();
            } else if passives.unlocked_nodes.contains(&conn.0)
                || passives.unlocked_nodes.contains(&conn.1)
            {
                *bg_color = Color::srgb(0.4, 0.4, 0.35).into();
            } else {
                *bg_color = Color::srgb(0.2, 0.2, 0.25).into();
            }
        }
    }

    if let Ok(mut text) = tooltip_text.get_single_mut() {
        if let Some(node) = hovered_node {
            let is_unlocked = hovered_node_id
                .map(|id| passives.unlocked_nodes.contains(&id))
                .unwrap_or(false);
            let category = hovered_node_id
                .map(|id| get_node_category(id, &node.effect))
                .unwrap_or(NodeCategory::Utility);

            text.sections[0].value = node.name.clone();
            text.sections[0].style.color = category.base_color();
            text.sections[2].value = node.description.clone();

            if !is_unlocked {
                let preview = format_stat_preview(stats, node);
                text.sections[4].value = format!("Preview: {}", preview);
            } else {
                text.sections[4].value = "(Unlocked)".to_string();
                text.sections[4].style.color = Color::srgb(0.7, 0.7, 0.7);
            }
        } else {
            text.sections[0].value = "Hover a node".to_string();
            text.sections[0].style.color = Color::srgb(0.6, 0.6, 0.6);
            text.sections[2].value = "Click to unlock available nodes".to_string();
            text.sections[4].value = "".to_string();
        }
    }
}

pub fn track_hovered_node(
    mut view_state: ResMut<PassiveTreeViewState>,
    node_buttons: Query<(&PassiveNodeButton, &Interaction), With<Button>>,
) {
    let mut found_hovered = None;
    for (node_btn, interaction) in node_buttons.iter() {
        if *interaction == Interaction::Hovered {
            found_hovered = Some(node_btn.0);
            break;
        }
    }
    view_state.hovered_node_id = found_hovered;
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
    commands.remove_resource::<PassiveTreeViewState>();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
