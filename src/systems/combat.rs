use crate::components::*;
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

            if distance < 65.0 {
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

            if distance < 110.0 {
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
    mut set: ParamSet<(
        Query<(
            &mut Health,
            Option<&mut Shield>,
            &mut Transform,
            Option<&Stats>,
            Option<&Invulnerable>,
        )>,
        Query<(&Transform, &PlayerPassives)>,
    )>,
    mut game_stats: ResMut<GameStats>,
) {
    for event in damage_events.read() {
        let mut knockback_info = None;

        if let Some(attacker_entity) = event.attacker {
            if let Ok((attacker_transform, passives)) = set.p1().get(attacker_entity) {
                if passives.unlocked_nodes.contains(&8) {
                    knockback_info = Some(attacker_transform.translation);
                }
            }
        }

        if let Ok((mut health, shield, mut transform, stats, invuln)) =
            set.p0().get_mut(event.target)
        {
            if invuln.is_some() {
                continue;
            }

            let armor = stats.map(|s| s.armor).unwrap_or(0.0);
            let damage_reduction = armor / (armor + 100.0);
            let mut final_damage = event.amount * (1.0 - damage_reduction);

            if let Some(mut s) = shield {
                if s.amount > 0.0 {
                    if s.amount >= final_damage {
                        s.amount -= final_damage;
                        final_damage = 0.0;
                    } else {
                        final_damage -= s.amount;
                        s.amount = 0.0;
                    }
                }
            }

            health.current -= final_damage;
            game_stats.damage_dealt += event.amount;

            if let Some(attacker_pos) = knockback_info {
                let dir = (transform.translation - attacker_pos)
                    .truncate()
                    .normalize_or_zero();
                transform.translation += (dir * 35.0).extend(0.0);
            }

            let color = if event.is_crit {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                Color::srgb(1.0, 0.3, 0.3)
            };

            let font_size = if event.is_crit { 26.0 } else { 18.0 };

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
                        transform.translation
                            + Vec3::new(rand::thread_rng().gen_range(-15.0..15.0), 20.0, 100.0),
                    ),
                    ..default()
                },
                DamageNumber {
                    velocity: Vec2::new(rand::thread_rng().gen_range(-25.0..25.0), 60.0),
                    lifetime: Timer::from_seconds(0.7, TimerMode::Once),
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
