use crate::components::*;
use crate::plugins::ui::*;
use crate::resources::*;
use bevy::prelude::*;

// === BOSS HEALTH BAR ===
pub fn spawn_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<Entity, Added<Boss>>,
    existing_bar: Query<Entity, With<BossHealthBarUi>>,
) {
    if boss_query.is_empty() {
        return;
    }
    if !existing_bar.is_empty() {
        return;
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    left: Val::Percent(50.0),
                    margin: UiRect {
                        left: Val::Px(-200.0),
                        ..default()
                    },
                    width: Val::Px(400.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(UI_BG_DARK),
                border_color: BorderColor(Color::srgb(0.6, 0.2, 0.2)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BossHealthBarUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "BOSS",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(1.0, 0.3, 0.3),
                    ..default()
                },
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(380.0),
                        height: Val::Px(24.0),
                        margin: UiRect::top(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(HP_BAR_BG),
                    border_color: BorderColor(UI_BORDER),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|bar_parent| {
                    bar_parent.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.7, 0.15, 0.15)),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        BossHealthBarFill,
                    ));

                    bar_parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(40.0),
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(1.0, 0.4, 0.3, 0.25)),
                        border_radius: BorderRadius {
                            top_left: Val::Px(2.0),
                            top_right: Val::Px(2.0),
                            bottom_left: Val::Px(0.0),
                            bottom_right: Val::Px(0.0),
                        },
                        ..default()
                    });
                });
        });
}

pub fn update_boss_health_bar(
    boss_query: Query<&Health, With<Boss>>,
    mut fill_query: Query<&mut Style, With<BossHealthBarFill>>,
) {
    let Ok(health) = boss_query.get_single() else {
        return;
    };
    let Ok(mut style) = fill_query.get_single_mut() else {
        return;
    };

    let percent = (health.current / health.max).clamp(0.0, 1.0) * 100.0;
    style.width = Val::Percent(percent);
}

pub fn despawn_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<Entity, With<Boss>>,
    bar_query: Query<Entity, With<BossHealthBarUi>>,
) {
    if boss_query.is_empty() {
        for entity in bar_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// === WORLD HEALTH BARS (on enemies) ===
pub fn update_health_bars(
    parents: Query<(&Health, &Children), Changed<Health>>,
    mut health_bars: Query<(&mut Sprite, &mut Transform, &HealthBarFill)>,
) {
    for (health, children) in parents.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut transform, fill)) = health_bars.get_mut(child) {
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
    }
}

// === HUD HP BAR ===
pub fn update_hud_health_bar(
    player: Query<(&Health, &Level), With<Player>>,
    mut hp_fill: Query<&mut Style, With<HpBarFill>>,
    mut hp_text: Query<&mut Text, (With<HpBarText>, Without<LevelText>)>,
    mut level_text: Query<&mut Text, (With<LevelText>, Without<HpBarText>)>,
) {
    let Ok((health, level)) = player.get_single() else {
        return;
    };

    if let Ok(mut style) = hp_fill.get_single_mut() {
        let percent = (health.current / health.max).clamp(0.0, 1.0) * 100.0;
        style.width = Val::Percent(percent);
    }

    if let Ok(mut text) = hp_text.get_single_mut() {
        text.sections[0].value = format!("{:.0} / {:.0}", health.current, health.max);
    }

    if let Ok(mut text) = level_text.get_single_mut() {
        text.sections[0].value = format!("Level {}", level.level);
    }
}

// === HUD SHIELD BAR ===
pub fn update_hud_shield_bar(
    player: Query<(&Shield, &Health), With<Player>>,
    mut shield_container: Query<&mut Visibility, With<ShieldBarContainer>>,
    mut shield_fill: Query<&mut Style, With<ShieldBarFill>>,
    mut shield_text: Query<&mut Text, With<ShieldBarText>>,
) {
    let Ok((shield, health)) = player.get_single() else {
        return;
    };

    let has_shield = shield.amount > 0.0;

    if let Ok(mut visibility) = shield_container.get_single_mut() {
        *visibility = if has_shield {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if has_shield {
        if let Ok(mut style) = shield_fill.get_single_mut() {
            let percent = (shield.amount / health.max).clamp(0.0, 1.0) * 100.0;
            style.width = Val::Percent(percent);
        }

        if let Ok(mut text) = shield_text.get_single_mut() {
            text.sections[0].value = format!("Shield: {:.0}", shield.amount);
        }
    }
}

// === HUD XP BAR ===
pub fn update_hud_xp_bar(
    player: Query<&Level, With<Player>>,
    mut xp_fill: Query<&mut Style, With<XpBarFill>>,
    mut xp_text: Query<&mut Text, With<XpBarText>>,
) {
    let Ok(level) = player.get_single() else {
        return;
    };

    if let Ok(mut style) = xp_fill.get_single_mut() {
        let percent = (level.xp as f32 / level.xp_to_next as f32).clamp(0.0, 1.0) * 100.0;
        style.width = Val::Percent(percent);
    }

    if let Ok(mut text) = xp_text.get_single_mut() {
        text.sections[0].value = format!("{} / {} XP", level.xp, level.xp_to_next);
    }
}

// === SKILL COOLDOWNS ===
pub fn update_skill_cooldowns_ui(
    player: Query<&SkillCooldowns, With<Player>>,
    mut cooldown_overlays: Query<(&mut Style, &SkillCooldownOverlay)>,
    mut cooldown_texts: Query<(&mut Text, &SkillCooldownText)>,
) {
    let Ok(cooldowns) = player.get_single() else {
        return;
    };

    for (mut style, overlay) in cooldown_overlays.iter_mut() {
        let (timer, max_duration) = match overlay.skill_type {
            SkillType::Dash => (&cooldowns.dash, crate::constants::DASH_COOLDOWN),
            SkillType::Nova => (&cooldowns.nova, crate::constants::NOVA_COOLDOWN_DEFAULT),
        };

        if timer.finished() {
            style.height = Val::Percent(0.0);
        } else {
            let remaining_percent = (timer.remaining_secs() / max_duration).clamp(0.0, 1.0) * 100.0;
            style.height = Val::Percent(remaining_percent);
        }
    }

    for (mut text, cooldown_text) in cooldown_texts.iter_mut() {
        let timer = match cooldown_text.skill_type {
            SkillType::Dash => &cooldowns.dash,
            SkillType::Nova => &cooldowns.nova,
        };

        if timer.finished() {
            text.sections[0].value = "".to_string();
        } else {
            text.sections[0].value = format!("{:.1}s", timer.remaining_secs());
        }
    }
}

// === STATS PANEL TOGGLE ===
pub fn toggle_stats_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_stats: ResMut<GameStats>,
    mut panel: Query<&mut Visibility, With<StatsPanelRoot>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        game_stats.show_stats = !game_stats.show_stats;

        if let Ok(mut visibility) = panel.get_single_mut() {
            *visibility = if game_stats.show_stats {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// === STATS PANEL CONTENT ===
pub fn update_stats_panel(
    player: Query<(&Stats, &PlayerPassives), With<Player>>,
    game_stats: Res<GameStats>,
    mut panel_content: Query<&mut Text, With<StatsPanelContent>>,
) {
    if !game_stats.show_stats {
        return;
    }

    let Ok((stats, passives)) = player.get_single() else {
        return;
    };

    let Ok(mut text) = panel_content.get_single_mut() else {
        return;
    };

    let dps = stats.damage
        * stats.attack_speed
        * (1.0 + stats.crit_chance * (stats.crit_multiplier - 1.0));

    text.sections[0].value = format!(
        "-- Combat --\n\
         DPS: {:.1}\n\
         Damage: {:.1}\n\
         Attack Speed: {:.2}x\n\
         Crit Chance: {:.0}%\n\
         Crit Multiplier: {:.1}x\n\
         \n-- Defense --\n\
         Armor: {:.0}\n\
         Life Regen: {:.1}/s\n\
         \n-- Utility --\n\
         Movement Speed: {:.0}\n\
         \n-- Session --\n\
         Kills: {}\n\
         Total Damage: {:.0}\n\
         Time: {:.0}s\n\
         \n-- Passives --\n\
         Unlocked: {}",
        dps,
        stats.damage,
        stats.attack_speed,
        stats.crit_chance * 100.0,
        stats.crit_multiplier,
        stats.armor,
        stats.life_regen,
        stats.speed,
        game_stats.enemies_killed,
        game_stats.damage_dealt,
        game_stats.time_survived,
        passives.unlocked_nodes.len(),
    );
}

// === BUFF/DEBUFF DISPLAY ===
pub fn update_buff_display(
    mut commands: Commands,
    player: Query<(Entity, &Shield, Option<&Invulnerable>), With<Player>>,
    buff_container: Query<Entity, With<BuffContainer>>,
    existing_buffs: Query<(Entity, &BuffIcon)>,
) {
    let Ok((_player_entity, shield, invulnerable)) = player.get_single() else {
        return;
    };

    let Ok(container_entity) = buff_container.get_single() else {
        return;
    };

    let has_shield = shield.amount > 0.0;
    let has_invuln = invulnerable.is_some();

    let mut shield_icon_exists = false;
    let mut invuln_icon_exists = false;

    for (entity, buff) in existing_buffs.iter() {
        match buff.buff_type {
            BuffType::Shield => {
                if !has_shield {
                    commands.entity(entity).despawn_recursive();
                } else {
                    shield_icon_exists = true;
                }
            }
            BuffType::Invulnerable => {
                if !has_invuln {
                    commands.entity(entity).despawn_recursive();
                } else {
                    invuln_icon_exists = true;
                }
            }
            _ => {}
        }
    }

    if has_shield && !shield_icon_exists {
        spawn_buff_icon(
            &mut commands,
            container_entity,
            BuffType::Shield,
            "S",
            SHIELD_BAR_FILL,
        );
    }

    if has_invuln && !invuln_icon_exists {
        spawn_buff_icon(
            &mut commands,
            container_entity,
            BuffType::Invulnerable,
            "I",
            Color::srgb(1.0, 0.9, 0.3),
        );
    }
}

fn spawn_buff_icon(
    commands: &mut Commands,
    container: Entity,
    buff_type: BuffType,
    label: &str,
    color: Color,
) {
    let icon = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(UI_BG_DARK),
                border_color: BorderColor(color),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BuffIcon { buff_type },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.0,
                    color,
                    ..default()
                },
            ));
        })
        .id();

    commands.entity(container).add_child(icon);
}
