use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

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
