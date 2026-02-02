use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

pub fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &mut Velocity,
        &mut Lifetime,
        &mut Projectile,
    )>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Projectile>)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (proj_entity, mut transform, mut velocity, mut lifetime, mut projectile) in
        projectiles.iter_mut()
    {
        transform.translation += (velocity.0 * time.delta_seconds()).extend(0.0);
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(proj_entity).despawn();
            continue;
        }
        let proj_pos = transform.translation.truncate();
        let mut target_enemy = None;
        for (enemy_entity, enemy_transform) in enemies.iter() {
            if projectile.hit_entities.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let distance = proj_pos.distance(enemy_pos);
            if distance < PROJECTILE_HIT_RADIUS {
                target_enemy = Some(enemy_entity);
                break;
            }
        }
        if let Some(enemy_entity) = target_enemy {
            projectile.hit_entities.insert(enemy_entity);
            damage_events.send(DamageEvent {
                target: enemy_entity,
                attacker: Some(projectile.owner),
                amount: projectile.damage,
                is_crit: projectile.is_crit,
            });
            let mut chained = false;
            if projectile.chain_count > 0 {
                let mut nearest_enemy = None;
                let mut min_dist = 200.0;
                for (other_enemy, other_transform) in enemies.iter() {
                    if projectile.hit_entities.contains(&other_enemy) {
                        continue;
                    }
                    let dist = proj_pos.distance(other_transform.translation.truncate());
                    if dist < min_dist {
                        min_dist = dist;
                        nearest_enemy = Some(other_transform.translation.truncate());
                    }
                }
                if let Some(target_pos) = nearest_enemy {
                    let new_dir = (target_pos - proj_pos).normalize_or_zero();
                    let speed = velocity.0.length();
                    velocity.0 = new_dir * speed;
                    transform.rotation = Quat::from_rotation_z(new_dir.y.atan2(new_dir.x));
                    projectile.chain_count -= 1;
                    chained = true;
                }
            }
            if !chained {
                if projectile.pierce == 0 {
                    commands.entity(proj_entity).despawn();
                } else {
                    projectile.pierce -= 1;
                }
            }
        }
    }
}

pub fn update_melee_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut melee_attacks: Query<(Entity, &Transform, &mut MeleeAttack)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (melee_entity, transform, mut melee) in melee_attacks.iter_mut() {
        melee.duration.tick(time.delta());
        if melee.duration.finished() {
            commands.entity(melee_entity).despawn();
            continue;
        }
        let melee_pos = transform.translation.truncate();
        for (enemy_entity, enemy_transform) in enemies.iter() {
            if melee.hit_entities.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let distance = melee_pos.distance(enemy_pos);
            if distance < MELEE_HIT_RADIUS {
                melee.hit_entities.insert(enemy_entity);
                damage_events.send(DamageEvent {
                    target: enemy_entity,
                    attacker: Some(melee.owner),
                    amount: melee.damage,
                    is_crit: melee.is_crit,
                });
            }
        }
    }
}

pub fn update_aoe_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut aoe_effects: Query<(Entity, &Transform, &mut AoeEffect, &mut Sprite)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (aoe_entity, transform, mut aoe, mut sprite) in aoe_effects.iter_mut() {
        aoe.duration.tick(time.delta());
        aoe.tick_timer.tick(time.delta());
        if aoe.duration.finished() {
            commands.entity(aoe_entity).despawn();
            continue;
        }
        let alpha = 0.45 * (1.0 - aoe.duration.fraction());
        sprite.color = sprite.color.with_alpha(alpha);
        if aoe.tick_timer.just_finished() {
            aoe.hit_this_tick.clear();
            let aoe_pos = transform.translation.truncate();
            let radius = sprite.custom_size.unwrap_or(Vec2::splat(100.0)).x / 2.0;
            for (enemy_entity, enemy_transform) in enemies.iter() {
                if aoe.hit_this_tick.contains(&enemy_entity) {
                    continue;
                }
                let enemy_pos = enemy_transform.translation.truncate();
                let distance = aoe_pos.distance(enemy_pos);
                if distance < radius {
                    aoe.hit_this_tick.insert(enemy_entity);
                    damage_events.send(DamageEvent {
                        target: enemy_entity,
                        attacker: Some(aoe.owner),
                        amount: aoe.damage,
                        is_crit: false,
                    });
                }
            }
        }
    }
}

pub fn process_damage(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut target_query: Query<
        (
            &mut Health,
            Option<&mut Shield>,
            &mut Transform,
            Option<&Stats>,
            Option<&Invulnerable>,
            Option<&mut ElementalStatus>,
            Option<&PlayerPassives>,
        ),
        Without<Player>,
    >,
    mut player_query: Query<
        (
            Entity,
            &mut Health,
            Option<&mut Shield>,
            &Transform,
            &PlayerPassives,
        ),
        With<Player>,
    >,
    mut game_stats: ResMut<GameStats>,
    mut status_events: EventWriter<ApplyStatusEvent>,
) {
    let mut rng = rand::thread_rng();

    for event in damage_events.read() {
        let mut knockback_info = None;
        let mut elemental_chances = Vec::new();
        let mut life_leech_pct = 0.0;
        let mut shield_leech_pct = 0.0;
        let mut damage_mult = 1.0;

        if let Some(attacker_entity) = event.attacker {
            if let Ok((p_entity, _, _, p_transform, passives)) = player_query.get(attacker_entity) {
                if p_entity == attacker_entity {
                    if passives.unlocked_nodes.contains(&8) {
                        knockback_info = Some(p_transform.translation);
                    }
                    for &node_id in &passives.unlocked_nodes {
                        match node_id {
                            11 => elemental_chances.push(PassiveEffect::ChanceFire(0.20)),
                            14 => elemental_chances.push(PassiveEffect::ChanceIce(0.25)),
                            17 => elemental_chances.push(PassiveEffect::ChanceLightning(0.15)),
                            201 => damage_mult *= 1.25,
                            204 => damage_mult *= 1.1,
                            203 => life_leech_pct += 0.015,
                            104 => shield_leech_pct += 0.03,
                            _ => {}
                        }
                    }
                }
            }
        }

        let mut final_damage = 0.0;
        let mut target_transform_pos = Vec3::ZERO;
        let target_is_crit = event.is_crit;

        if let Ok((mut health, mut shield, mut transform, stats, invuln, status, target_passives)) =
            target_query.get_mut(event.target)
        {
            if invuln.is_some() {
                continue;
            }
            let mut armor = stats.map(|s| s.armor).unwrap_or(0.0);
            if let Some(passives) = target_passives {
                if passives.unlocked_nodes.contains(&101) {
                    armor *= 1.3;
                }
            }
            if let Some(ref s) = status {
                if s.fire_stacks > 0 {
                    armor *= 1.0 - (s.fire_stacks as f32 * 0.05).min(0.5);
                }
            }
            let damage_reduction = armor / (armor + 100.0);
            final_damage = event.amount * damage_mult * (1.0 - damage_reduction);
            target_transform_pos = transform.translation;

            if let Some(ref mut s) = shield {
                if s.amount > 0.0 {
                    if s.amount >= final_damage {
                        s.amount -= final_damage;
                    } else {
                        let rem = final_damage - s.amount;
                        s.amount = 0.0;
                        health.current -= rem;
                    }
                } else {
                    health.current -= final_damage;
                }
            } else {
                health.current -= final_damage;
            }

            for chance_effect in &elemental_chances {
                match chance_effect {
                    PassiveEffect::ChanceFire(c) => {
                        if rng.r#gen::<f32>() < *c {
                            status_events.send(ApplyStatusEvent {
                                target: event.target,
                                effect: *chance_effect,
                            });
                        }
                    }
                    PassiveEffect::ChanceIce(c) => {
                        if rng.r#gen::<f32>() < *c {
                            status_events.send(ApplyStatusEvent {
                                target: event.target,
                                effect: *chance_effect,
                            });
                        }
                    }
                    PassiveEffect::ChanceLightning(c) => {
                        if rng.r#gen::<f32>() < *c {
                            status_events.send(ApplyStatusEvent {
                                target: event.target,
                                effect: *chance_effect,
                            });
                        }
                    }
                    _ => {}
                }
            }
            if let Some(attacker_pos) = knockback_info {
                let dir = (transform.translation - attacker_pos)
                    .truncate()
                    .normalize_or_zero();
                transform.translation += (dir * KNOCKBACK_FORCE).extend(0.0);
            }
        } else if let Ok((_e, mut health, mut shield, transform, passives)) =
            player_query.get_mut(event.target)
        {
            let mut armor = 0.0;
            if passives.unlocked_nodes.contains(&101) {
                armor = 50.0;
            }
            let damage_reduction = armor / (armor + 100.0);
            final_damage = event.amount * (1.0 - damage_reduction);
            target_transform_pos = transform.translation;

            if let Some(ref mut s) = shield {
                if s.amount > 0.0 {
                    if s.amount >= final_damage {
                        s.amount -= final_damage;
                    } else {
                        let rem = final_damage - s.amount;
                        s.amount = 0.0;
                        health.current -= rem;
                    }
                } else {
                    health.current -= final_damage;
                }
            } else {
                health.current -= final_damage;
            }
        }

        if final_damage > 0.0 {
            game_stats.damage_dealt += final_damage;
            if let Some(attacker_entity) = event.attacker {
                if let Ok((p_entity, mut p_health, mut p_shield, _, _)) =
                    player_query.get_single_mut()
                {
                    if p_entity == attacker_entity {
                        if life_leech_pct > 0.0 {
                            p_health.current = (p_health.current + final_damage * life_leech_pct)
                                .min(p_health.max);
                        }
                        if let Some(ref mut s) = p_shield {
                            if shield_leech_pct > 0.0 {
                                s.amount = (s.amount + final_damage * shield_leech_pct)
                                    .min(p_health.max * 0.5);
                            }
                        }
                    }
                }
            }
            let color = if target_is_crit {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                Color::srgb(1.0, 0.3, 0.3)
            };
            let font_size = if target_is_crit { 26.0 } else { 18.0 };
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        format!("{:.0}", final_damage),
                        TextStyle {
                            font_size,
                            color,
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(
                        target_transform_pos
                            + Vec3::new(rand::thread_rng().r#gen_range(-15.0..15.0), 20.0, 100.0),
                    ),
                    ..default()
                },
                DamageNumber {
                    velocity: Vec2::new(rand::thread_rng().r#gen_range(-25.0..25.0), 60.0),
                    lifetime: Timer::from_seconds(DAMAGE_NUMBER_LIFETIME, TimerMode::Once),
                },
            ));
        }
    }
}

pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut numbers: Query<(Entity, &mut Transform, &mut DamageNumber, &mut Text)>,
) {
    for (entity, mut transform, mut number, mut text) in numbers.iter_mut() {
        number.lifetime.tick(time.delta());
        if number.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        number.velocity.y -= 120.0 * time.delta_seconds();
        transform.translation += (number.velocity * time.delta_seconds()).extend(0.0);
        let alpha = 1.0 - number.lifetime.fraction();
        for section in text.sections.iter_mut() {
            section.style.color = section.style.color.with_alpha(alpha);
        }
    }
}

pub fn spawn_melee_attack(
    commands: &mut Commands,
    player_entity: Entity,
    player_pos: Vec2,
    direction: Vec2,
    damage: f32,
    _attack_speed: f32,
    is_tank: bool,
) {
    let mut rng = rand::thread_rng();
    let is_crit = rng.r#gen::<f32>() < 0.1;
    let melee_damage = damage * 1.8;
    let spawn_pos = player_pos + direction * 75.0;
    let (size, color) = if is_tank {
        (Vec2::new(220.0, 180.0), Color::srgba(0.2, 0.5, 1.0, 0.7))
    } else {
        (Vec2::new(160.0, 120.0), Color::srgba(0.9, 0.4, 0.1, 0.7))
    };
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: if is_crit {
                    Color::srgba(1.0, 0.6, 0.0, 0.85)
                } else {
                    color
                },
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(spawn_pos.extend(4.0))
                .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ..default()
        },
        MeleeAttack {
            damage: melee_damage,
            owner: player_entity,
            duration: Timer::from_seconds(0.12, TimerMode::Once),
            hit_entities: HashSet::new(),
            is_crit,
        },
    ));
}
