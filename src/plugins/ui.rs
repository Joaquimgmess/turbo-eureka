use bevy::prelude::*;

use crate::components::*;
use crate::systems::combat::update_damage_numbers;
use crate::systems::ui::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                (
                    update_health_bars,
                    update_hud_health_bar,
                    update_hud_shield_bar,
                    update_hud_xp_bar,
                    update_skill_cooldowns_ui,
                    toggle_stats_panel,
                    update_stats_panel,
                    update_buff_display,
                    update_damage_numbers,
                    spawn_boss_health_bar,
                    update_boss_health_bar,
                    despawn_boss_health_bar,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// === UI COLOR PALETTE (Fantasy/Dark Theme) ===
pub const UI_BG_DARK: Color = Color::srgba(0.08, 0.08, 0.12, 0.95);
pub const UI_BG_MEDIUM: Color = Color::srgba(0.12, 0.12, 0.18, 0.90);
pub const UI_BORDER: Color = Color::srgba(0.3, 0.28, 0.22, 1.0);
pub const UI_BORDER_ACCENT: Color = Color::srgba(0.6, 0.5, 0.3, 1.0);
pub const UI_TEXT_PRIMARY: Color = Color::srgba(0.95, 0.92, 0.85, 1.0);
pub const UI_TEXT_SECONDARY: Color = Color::srgba(0.7, 0.68, 0.62, 1.0);
pub const UI_TEXT_MUTED: Color = Color::srgba(0.5, 0.48, 0.44, 1.0);

pub const HP_BAR_BG: Color = Color::srgba(0.15, 0.08, 0.08, 0.95);
pub const HP_BAR_FILL_DARK: Color = Color::srgb(0.6, 0.1, 0.1);
pub const HP_BAR_FILL_BRIGHT: Color = Color::srgb(0.85, 0.2, 0.15);
pub const HP_BAR_HIGHLIGHT: Color = Color::srgba(1.0, 0.5, 0.4, 0.3);

pub const SHIELD_BAR_BG: Color = Color::srgba(0.08, 0.1, 0.15, 0.95);
pub const SHIELD_BAR_FILL: Color = Color::srgb(0.3, 0.6, 0.9);
pub const SHIELD_BAR_HIGHLIGHT: Color = Color::srgba(0.6, 0.85, 1.0, 0.4);

pub const XP_BAR_BG: Color = Color::srgba(0.1, 0.1, 0.08, 0.9);
pub const XP_BAR_FILL: Color = Color::srgb(0.7, 0.55, 0.15);
pub const XP_BAR_HIGHLIGHT: Color = Color::srgba(1.0, 0.9, 0.4, 0.35);

pub const SKILL_BG: Color = Color::srgba(0.1, 0.1, 0.12, 0.95);
pub const SKILL_READY: Color = Color::srgb(0.2, 0.6, 0.3);
pub const SKILL_COOLDOWN: Color = Color::srgb(0.3, 0.3, 0.35);

// === UI COMPONENT MARKERS ===
#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HpBarContainer;

#[derive(Component)]
pub struct HpBarFill;

#[derive(Component)]
pub struct HpBarHighlight;

#[derive(Component)]
pub struct HpBarText;

#[derive(Component)]
pub struct ShieldBarContainer;

#[derive(Component)]
pub struct ShieldBarFill;

#[derive(Component)]
pub struct ShieldBarText;

#[derive(Component)]
pub struct XpBarContainer;

#[derive(Component)]
pub struct XpBarFill;

#[derive(Component)]
pub struct XpBarText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct SkillsContainer;

#[derive(Component)]
pub struct SkillSlot {
    pub skill_type: SkillType,
}

#[derive(Component)]
pub struct SkillCooldownOverlay {
    pub skill_type: SkillType,
}

#[derive(Component)]
pub struct SkillKeyText {
    pub skill_type: SkillType,
}

#[derive(Component)]
pub struct SkillCooldownText {
    pub skill_type: SkillType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkillType {
    Dash,
    Nova,
}

#[derive(Component)]
pub struct StatsPanelRoot;

#[derive(Component)]
pub struct StatsPanelContent;

#[derive(Component)]
pub struct BuffContainer;

#[derive(Component)]
pub struct BuffIcon {
    pub buff_type: BuffType,
}

#[derive(Component)]
pub struct BuffTimerBorder {
    pub buff_type: BuffType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuffType {
    Shield,
    SpeedBoost,
    DamageBoost,
    Invulnerable,
}

fn setup_hud(mut commands: Commands) {
    // === MAIN HUD ROOT ===
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            // === TOP-LEFT: HP & SHIELD BARS ===
            spawn_health_section(parent);

            // === BOTTOM-CENTER: SKILLS ===
            spawn_skills_section(parent);

            // === TOP-RIGHT: STATS PANEL (Tab Toggle) ===
            spawn_stats_panel(parent);

            // === BELOW HP: BUFF/DEBUFF ICONS ===
            spawn_buff_section(parent);

            // === BOTTOM: XP BAR ===
            spawn_xp_section(parent);

            // === BOTTOM-LEFT: CONTROLS HINT ===
            spawn_controls_hint(parent);
        });
}

fn spawn_health_section(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            ..default()
        })
        .with_children(|section| {
            // Level indicator
            section.spawn((
                TextBundle::from_section(
                    "Level 1",
                    TextStyle {
                        font_size: 18.0,
                        color: UI_TEXT_PRIMARY,
                        ..default()
                    },
                ),
                LevelText,
            ));

            // HP Bar (larger, more visible)
            spawn_bar_with_gradient(
                section,
                300.0,
                28.0,
                HP_BAR_BG,
                HP_BAR_FILL_BRIGHT,
                HP_BAR_HIGHLIGHT,
                HpBarContainer,
                HpBarFill,
                HpBarHighlight,
            );

            // HP Text overlay
            section.spawn((
                TextBundle::from_section(
                    "100 / 100",
                    TextStyle {
                        font_size: 14.0,
                        color: UI_TEXT_PRIMARY,
                        ..default()
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(26.0),
                    left: Val::Px(0.0),
                    width: Val::Px(300.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                }),
                HpBarText,
            ));

            // Shield Bar (below HP, slightly smaller)
            spawn_shield_bar(section);

            // Shield Text
            section.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 12.0,
                        color: SHIELD_BAR_FILL,
                        ..default()
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(58.0),
                    left: Val::Px(0.0),
                    width: Val::Px(300.0),
                    ..default()
                }),
                ShieldBarText,
            ));
        });
}

fn spawn_bar_with_gradient<C1: Component, C2: Component, C3: Component>(
    parent: &mut ChildBuilder,
    width: f32,
    height: f32,
    bg_color: Color,
    fill_color: Color,
    highlight_color: Color,
    container_marker: C1,
    fill_marker: C2,
    highlight_marker: C3,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(width),
                    height: Val::Px(height),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(bg_color),
                border_color: BorderColor(UI_BORDER),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            container_marker,
        ))
        .with_children(|bar| {
            // Main fill
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: BackgroundColor(fill_color),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                fill_marker,
            ));

            // Top highlight for gradient effect
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(40.0),
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        ..default()
                    },
                    background_color: BackgroundColor(highlight_color),
                    border_radius: BorderRadius {
                        top_left: Val::Px(2.0),
                        top_right: Val::Px(2.0),
                        bottom_left: Val::Px(0.0),
                        bottom_right: Val::Px(0.0),
                    },
                    ..default()
                },
                highlight_marker,
            ));
        });
}

fn spawn_shield_bar(parent: &mut ChildBuilder) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(16.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: BackgroundColor(SHIELD_BAR_BG),
                border_color: BorderColor(UI_BORDER),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                visibility: Visibility::Hidden,
                ..default()
            },
            ShieldBarContainer,
        ))
        .with_children(|bar| {
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: BackgroundColor(SHIELD_BAR_FILL),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                ShieldBarFill,
            ));
        });
}

fn spawn_skills_section(parent: &mut ChildBuilder) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(80.0),
                    left: Val::Percent(50.0),
                    margin: UiRect {
                        left: Val::Px(-130.0),
                        ..default()
                    },
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                background_color: BackgroundColor(UI_BG_DARK),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            SkillsContainer,
        ))
        .with_children(|container| {
            spawn_skill_slot(
                container,
                SkillType::Dash,
                "Q",
                "DASH",
                Color::srgb(0.4, 0.7, 1.0),
            );
            spawn_skill_slot(
                container,
                SkillType::Nova,
                "SPC",
                "NOVA",
                Color::srgb(1.0, 0.5, 0.3),
            );
        });
}

fn spawn_skill_slot(
    parent: &mut ChildBuilder,
    skill_type: SkillType,
    key: &str,
    name: &str,
    accent_color: Color,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(SKILL_BG),
                border_color: BorderColor(accent_color),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            SkillSlot { skill_type },
        ))
        .with_children(|slot| {
            // Cooldown overlay (darkens when on cooldown)
            slot.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(0.0),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                SkillCooldownOverlay { skill_type },
            ));

            // Key binding text
            slot.spawn((
                TextBundle::from_section(
                    key,
                    TextStyle {
                        font_size: 16.0,
                        color: UI_TEXT_PRIMARY,
                        ..default()
                    },
                ),
                SkillKeyText { skill_type },
            ));

            // Skill name
            slot.spawn(TextBundle::from_section(
                name,
                TextStyle {
                    font_size: 10.0,
                    color: accent_color,
                    ..default()
                },
            ));

            // Cooldown timer text
            slot.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 12.0,
                        color: UI_TEXT_SECONDARY,
                        ..default()
                    },
                ),
                SkillCooldownText { skill_type },
            ));
        });
}

fn spawn_stats_panel(parent: &mut ChildBuilder) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    right: Val::Px(20.0),
                    min_width: Val::Px(220.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(UI_BG_DARK),
                border_color: BorderColor(UI_BORDER_ACCENT),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                visibility: Visibility::Hidden,
                ..default()
            },
            StatsPanelRoot,
        ))
        .with_children(|panel| {
            // Header
            panel.spawn(TextBundle::from_section(
                "CHARACTER STATS",
                TextStyle {
                    font_size: 16.0,
                    color: UI_BORDER_ACCENT,
                    ..default()
                },
            ));

            // Divider
            panel.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                background_color: BackgroundColor(UI_BORDER),
                ..default()
            });

            // Stats content (will be updated dynamically)
            panel.spawn((
                TextBundle::from_sections([TextSection::new(
                    "",
                    TextStyle {
                        font_size: 14.0,
                        color: UI_TEXT_PRIMARY,
                        ..default()
                    },
                )]),
                StatsPanelContent,
            ));
        });
}

fn spawn_buff_section(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(110.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                ..default()
            },
            ..default()
        },
        BuffContainer,
    ));
}

fn spawn_xp_section(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-200.0),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ..default()
        })
        .with_children(|section| {
            // XP Bar
            section
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(400.0),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(XP_BAR_BG),
                        border_color: BorderColor(UI_BORDER),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    XpBarContainer,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            background_color: BackgroundColor(XP_BAR_FILL),
                            border_radius: BorderRadius::all(Val::Px(5.0)),
                            ..default()
                        },
                        XpBarFill,
                    ));

                    // XP Highlight
                    bar.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(35.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            ..default()
                        },
                        background_color: BackgroundColor(XP_BAR_HIGHLIGHT),
                        border_radius: BorderRadius {
                            top_left: Val::Px(5.0),
                            top_right: Val::Px(5.0),
                            bottom_left: Val::Px(0.0),
                            bottom_right: Val::Px(0.0),
                        },
                        ..default()
                    });
                });

            // XP Text
            section.spawn((
                TextBundle::from_section(
                    "0 / 100 XP",
                    TextStyle {
                        font_size: 12.0,
                        color: UI_TEXT_SECONDARY,
                        ..default()
                    },
                ),
                XpBarText,
            ));
        });
}

fn spawn_controls_hint(parent: &mut ChildBuilder) {
    parent.spawn(
        TextBundle::from_section(
            "WASD: Move | LMB: Shoot | RMB: Melee | Q: Dash | Space: Nova | Tab: Stats | P: Passives",
            TextStyle {
                font_size: 12.0,
                color: UI_TEXT_MUTED,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(20.0),
            ..default()
        }),
    );
}
