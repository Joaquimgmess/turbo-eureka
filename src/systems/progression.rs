use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::Rng;

pub fn tick_progression_event_timer(
    time: Res<Time>,
    mut event_timer: ResMut<ProgressionEventTimer>,
) {
    event_timer.timer.tick(time.delta());
}

pub fn spawn_progression_event(
    mut commands: Commands,
    mut event_timer: ResMut<ProgressionEventTimer>,
    mut horde_wave: ResMut<HordeWaveActive>,
    player_query: Query<&Transform, With<Player>>,
    sprites: Res<CharacterSprites>,
    map_tier: Res<MapTier>,
    player_level: Query<&Level, With<Player>>,
) {
    if !event_timer.timer.finished() {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let Ok(level) = player_level.get_single() else {
        return;
    };

    let mut rng = rand::thread_rng();
    let event_type = match rng.gen_range(0..5) {
        0 => ProgressionEventType::EliteSpawn,
        1 => ProgressionEventType::HordeWave,
        2 => ProgressionEventType::TreasureGoblin,
        3 => ProgressionEventType::Shrine,
        _ => ProgressionEventType::ChallengeZone,
    };

    let player_pos = player_transform.translation.truncate();

    match event_type {
        ProgressionEventType::EliteSpawn => {
            spawn_elite(
                &mut commands,
                &sprites,
                player_pos,
                &map_tier,
                level,
                &mut rng,
            );
        }
        ProgressionEventType::HordeWave => {
            horde_wave.active = true;
            horde_wave.timer = Some(Timer::from_seconds(HORDE_WAVE_DURATION, TimerMode::Once));
            commands.spawn((
                HordeWaveMarker,
                Text2dBundle {
                    text: Text::from_section(
                        "HORDE WAVE!",
                        TextStyle {
                            font_size: 48.0,
                            color: Color::srgb(1.0, 0.3, 0.3),
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(
                        player_pos.extend(100.0) + Vec3::Y * 100.0,
                    ),
                    ..default()
                },
                DamageNumber {
                    velocity: Vec2::new(0.0, 30.0),
                    lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                },
            ));
        }
        ProgressionEventType::TreasureGoblin => {
            spawn_treasure_goblin(&mut commands, &sprites, player_pos, level, &mut rng);
        }
        ProgressionEventType::Shrine => {
            spawn_shrine(&mut commands, player_pos, &mut rng);
        }
        ProgressionEventType::ChallengeZone => {
            spawn_challenge_zone(&mut commands, player_pos, &mut rng);
        }
    }

    event_timer.last_event = Some(event_type);

    let next_interval = rng.gen_range(EVENT_MIN_INTERVAL..EVENT_MAX_INTERVAL);
    event_timer.timer = Timer::from_seconds(next_interval, TimerMode::Once);
}

fn spawn_elite(
    commands: &mut Commands,
    sprites: &CharacterSprites,
    player_pos: Vec2,
    map_tier: &MapTier,
    level: &Level,
    rng: &mut impl Rng,
) {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(300.0..450.0);
    let spawn_pos = player_pos + Vec2::from_angle(angle) * distance;

    let tier_scale = 1.0 + (map_tier.0 as f32 - 1.0) * 0.3;
    let health_scale = 1.0 + (level.level as f32 - 1.0) * LEVEL_HEALTH_SCALE;
    let damage_scale = 1.0 + (level.level as f32 - 1.0) * LEVEL_DAMAGE_SCALE;

    let base_health = 100.0 * ELITE_HP_MULTIPLIER * tier_scale * health_scale;
    let base_damage = 20.0 * tier_scale * damage_scale;
    let base_xp = (50.0 * ELITE_XP_MULTIPLIER) as u32;

    let size = Vec2::new(280.0, 280.0);

    let elite_entity = commands
        .spawn((
            Elite {
                aura_radius: ELITE_AURA_RADIUS,
            },
            Enemy {
                damage: base_damage,
                xp_value: base_xp,
                attack_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
                speed: 70.0,
            },
            ElementalStatus::default(),
            Health {
                current: base_health,
                max: base_health,
            },
            Velocity(Vec2::ZERO),
            CharacterState::Idle,
            SpriteBundle {
                texture: sprites.orc_idle.clone(),
                sprite: Sprite {
                    color: Color::srgb(0.8, 0.4, 1.0),
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(spawn_pos.extend(5.0)),
                ..default()
            },
            TextureAtlas {
                layout: sprites.layout.clone(),
                index: 0,
            },
            AnimationConfig {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                frame_count: 6,
                state: CharacterState::Idle,
            },
        ))
        .id();

    commands.entity(elite_entity).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(size.x + 8.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.1),
                ..default()
            },
            HealthBar,
        ));
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.85, 0.1, 0.1),
                    custom_size: Some(Vec2::new(size.x + 6.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.2),
                ..default()
            },
            HealthBarFill(size.x + 6.0),
        ));
    });

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "ELITE!",
                TextStyle {
                    font_size: 36.0,
                    color: Color::srgb(0.8, 0.4, 1.0),
                    ..default()
                },
            ),
            transform: Transform::from_translation(spawn_pos.extend(100.0) + Vec3::Y * 80.0),
            ..default()
        },
        DamageNumber {
            velocity: Vec2::new(0.0, 40.0),
            lifetime: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

fn spawn_treasure_goblin(
    commands: &mut Commands,
    sprites: &CharacterSprites,
    player_pos: Vec2,
    level: &Level,
    rng: &mut impl Rng,
) {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(250.0..400.0);
    let spawn_pos = player_pos + Vec2::from_angle(angle) * distance;

    let base_xp = 30 * TREASURE_GOBLIN_XP_MULT;
    let health_scale = 1.0 + (level.level as f32 - 1.0) * LEVEL_HEALTH_SCALE;

    let size = Vec2::new(120.0, 120.0);

    let goblin_entity = commands
        .spawn((
            TreasureGoblin {
                flee_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                death_timer: Timer::from_seconds(TREASURE_GOBLIN_LIFETIME, TimerMode::Once),
            },
            Enemy {
                damage: 0.0,
                xp_value: base_xp,
                attack_cooldown: Timer::from_seconds(999.0, TimerMode::Once),
                speed: TREASURE_GOBLIN_SPEED,
            },
            ElementalStatus::default(),
            Health {
                current: 50.0 * health_scale,
                max: 50.0 * health_scale,
            },
            Velocity(Vec2::ZERO),
            CharacterState::Walking,
            SpriteBundle {
                texture: sprites.orc_idle.clone(),
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.85, 0.0),
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(spawn_pos.extend(5.0)),
                ..default()
            },
            TextureAtlas {
                layout: sprites.layout.clone(),
                index: 0,
            },
            AnimationConfig {
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
                frame_count: 6,
                state: CharacterState::Walking,
            },
        ))
        .id();

    commands.entity(goblin_entity).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(size.x + 8.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.1),
                ..default()
            },
            HealthBar,
        ));
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.85, 0.0),
                    custom_size: Some(Vec2::new(size.x + 6.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.2),
                ..default()
            },
            HealthBarFill(size.x + 6.0),
        ));
    });

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "TREASURE GOBLIN!",
                TextStyle {
                    font_size: 32.0,
                    color: Color::srgb(1.0, 0.85, 0.0),
                    ..default()
                },
            ),
            transform: Transform::from_translation(spawn_pos.extend(100.0) + Vec3::Y * 60.0),
            ..default()
        },
        DamageNumber {
            velocity: Vec2::new(0.0, 30.0),
            lifetime: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

fn spawn_shrine(commands: &mut Commands, player_pos: Vec2, rng: &mut impl Rng) {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(150.0..350.0);
    let spawn_pos = player_pos + Vec2::from_angle(angle) * distance;

    let shrine_type = match rng.gen_range(0..4) {
        0 => ShrineType::Damage,
        1 => ShrineType::Speed,
        2 => ShrineType::Defense,
        _ => ShrineType::CritChance,
    };

    let color = match shrine_type {
        ShrineType::Damage => Color::srgb(1.0, 0.3, 0.3),
        ShrineType::Speed => Color::srgb(0.3, 1.0, 0.3),
        ShrineType::Defense => Color::srgb(0.3, 0.5, 1.0),
        ShrineType::CritChance => Color::srgb(1.0, 1.0, 0.3),
    };

    commands.spawn((
        Shrine {
            buff_type: shrine_type,
            used: false,
        },
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(60.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_pos.extend(3.0)),
            ..default()
        },
    ));

    let shrine_name = match shrine_type {
        ShrineType::Damage => "DAMAGE SHRINE",
        ShrineType::Speed => "SPEED SHRINE",
        ShrineType::Defense => "DEFENSE SHRINE",
        ShrineType::CritChance => "CRIT SHRINE",
    };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                shrine_name,
                TextStyle {
                    font_size: 24.0,
                    color,
                    ..default()
                },
            ),
            transform: Transform::from_translation(spawn_pos.extend(100.0) + Vec3::Y * 50.0),
            ..default()
        },
        DamageNumber {
            velocity: Vec2::new(0.0, 20.0),
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
        },
    ));
}

fn spawn_challenge_zone(commands: &mut Commands, player_pos: Vec2, rng: &mut impl Rng) {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(100.0..250.0);
    let spawn_pos = player_pos + Vec2::from_angle(angle) * distance;

    commands.spawn((
        ChallengeZone {
            radius: CHALLENGE_ZONE_RADIUS,
            bonus_xp_mult: CHALLENGE_ZONE_XP_MULT,
            duration: Timer::from_seconds(CHALLENGE_ZONE_DURATION, TimerMode::Once),
        },
        ChallengeZoneVisual,
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.2, 0.8, 1.0, 0.3),
                custom_size: Some(Vec2::splat(CHALLENGE_ZONE_RADIUS * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_pos.extend(1.0)),
            ..default()
        },
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "CHALLENGE ZONE!",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.2, 0.8, 1.0),
                    ..default()
                },
            ),
            transform: Transform::from_translation(spawn_pos.extend(100.0) + Vec3::Y * 80.0),
            ..default()
        },
        DamageNumber {
            velocity: Vec2::new(0.0, 25.0),
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
        },
    ));
}

pub fn update_horde_wave(time: Res<Time>, mut horde_wave: ResMut<HordeWaveActive>) {
    if let Some(ref mut timer) = horde_wave.timer {
        timer.tick(time.delta());
        if timer.finished() {
            horde_wave.active = false;
            horde_wave.timer = None;
        }
    }
}

pub fn treasure_goblin_ai(
    time: Res<Time>,
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut goblins: Query<
        (
            Entity,
            &mut Transform,
            &mut TreasureGoblin,
            &mut Sprite,
            &Enemy,
        ),
        Without<Player>,
    >,
    mut xp_events: EventWriter<SpawnXpOrbEvent>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (entity, mut transform, mut goblin, mut sprite, enemy) in goblins.iter_mut() {
        goblin.death_timer.tick(time.delta());
        goblin.flee_timer.tick(time.delta());

        if goblin.death_timer.finished() {
            for _ in 0..5 {
                xp_events.send(SpawnXpOrbEvent {
                    position: transform.translation,
                    value: enemy.xp_value / 5,
                });
            }
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let goblin_pos = transform.translation.truncate();
        let away_from_player = (goblin_pos - player_pos).normalize_or_zero();

        if goblin.flee_timer.finished() {
            let mut rng = rand::thread_rng();
            let jitter = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5));
            let direction = (away_from_player + jitter).normalize_or_zero();
            let movement = direction * enemy.speed * time.delta_seconds();
            transform.translation += movement.extend(0.0);

            if direction.x < 0.0 {
                sprite.flip_x = true;
            } else if direction.x > 0.0 {
                sprite.flip_x = false;
            }
        }

        let remaining = goblin.death_timer.remaining_secs();
        if remaining < 2.0 {
            let flash = (remaining * 10.0).sin().abs();
            sprite.color = Color::srgb(1.0, 0.85 * flash, 0.0);
        }
    }
}

pub fn update_elite_aura(
    mut commands: Commands,
    elites: Query<(Entity, &Transform, &Elite)>,
    existing_auras: Query<(Entity, &EliteAura)>,
) {
    let elite_entities: std::collections::HashSet<_> = elites.iter().map(|(e, _, _)| e).collect();

    for (aura_entity, aura) in existing_auras.iter() {
        if !elite_entities.contains(&aura.owner) {
            commands.entity(aura_entity).despawn();
        }
    }

    for (elite_entity, transform, elite) in elites.iter() {
        let has_aura = existing_auras.iter().any(|(_, a)| a.owner == elite_entity);
        if !has_aura {
            commands.spawn((
                EliteAura {
                    owner: elite_entity,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0.8, 0.4, 1.0, 0.2),
                        custom_size: Some(Vec2::splat(elite.aura_radius * 2.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation - Vec3::Z),
                    ..default()
                },
            ));
        }
    }
}

pub fn follow_elite_aura(
    elites: Query<&Transform, With<Elite>>,
    mut auras: Query<(&mut Transform, &EliteAura), Without<Elite>>,
) {
    for (mut aura_transform, aura) in auras.iter_mut() {
        if let Ok(elite_transform) = elites.get(aura.owner) {
            aura_transform.translation = elite_transform.translation - Vec3::Z;
        }
    }
}

pub fn interact_with_shrine(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut shrines: Query<(Entity, &Transform, &mut Shrine, &mut Sprite)>,
) {
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (shrine_entity, shrine_transform, mut shrine, mut sprite) in shrines.iter_mut() {
        if shrine.used {
            continue;
        }

        let shrine_pos = shrine_transform.translation.truncate();
        if shrine_pos.distance(player_pos) < 50.0 {
            shrine.used = true;
            sprite.color = Color::srgba(0.5, 0.5, 0.5, 0.5);

            commands.entity(player_entity).insert(ShrineBuff {
                buff_type: shrine.buff_type,
                timer: Timer::from_seconds(SHRINE_BUFF_DURATION, TimerMode::Once),
            });

            let buff_name = match shrine.buff_type {
                ShrineType::Damage => "+50% DAMAGE",
                ShrineType::Speed => "+30% SPEED",
                ShrineType::Defense => "+50 ARMOR",
                ShrineType::CritChance => "+25% CRIT",
            };

            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        buff_name,
                        TextStyle {
                            font_size: 28.0,
                            color: Color::srgb(1.0, 1.0, 1.0),
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(
                        player_transform.translation + Vec3::new(0.0, 80.0, 100.0),
                    ),
                    ..default()
                },
                DamageNumber {
                    velocity: Vec2::new(0.0, 50.0),
                    lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                },
            ));

            commands
                .entity(shrine_entity)
                .insert(Lifetime(Timer::from_seconds(3.0, TimerMode::Once)));
        }
    }
}

pub fn apply_shrine_buffs(
    time: Res<Time>,
    mut commands: Commands,
    mut players: Query<(Entity, Option<&mut ShrineBuff>), With<Player>>,
) {
    for (entity, shrine_buff_opt) in players.iter_mut() {
        if let Some(mut buff) = shrine_buff_opt {
            buff.timer.tick(time.delta());
            if buff.timer.just_finished() {
                commands.entity(entity).remove::<ShrineBuff>();
            }
        }
    }
}

pub fn update_challenge_zones(
    time: Res<Time>,
    mut commands: Commands,
    mut zones: Query<(Entity, &mut ChallengeZone, &mut Sprite, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (entity, mut zone, mut sprite, transform) in zones.iter_mut() {
        zone.duration.tick(time.delta());

        if zone.duration.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let remaining_fraction =
            zone.duration.remaining_secs() / zone.duration.duration().as_secs_f32();
        let alpha = 0.3 * remaining_fraction;
        sprite.color = Color::srgba(0.2, 0.8, 1.0, alpha);

        let zone_pos = transform.translation.truncate();
        let in_zone = zone_pos.distance(player_pos) < zone.radius;

        if in_zone {
            let pulse = (time.elapsed_seconds() * 5.0).sin() * 0.1 + 0.4;
            sprite.color = Color::srgba(0.2, 0.8, 1.0, pulse);
        }
    }
}

pub fn cleanup_used_shrines(
    time: Res<Time>,
    mut commands: Commands,
    mut shrines: Query<(Entity, &mut Lifetime), With<Shrine>>,
) {
    for (entity, mut lifetime) in shrines.iter_mut() {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_mini_boss(
    mut commands: Commands,
    time: Res<Time>,
    mut mini_boss_timer: ResMut<MiniBossTimer>,
    sprites: Res<CharacterSprites>,
    player_query: Query<(&Transform, &Level), With<Player>>,
    map_tier: Res<MapTier>,
    mini_boss_query: Query<&MiniBoss>,
    boss_query: Query<&Boss>,
) {
    mini_boss_timer.timer.tick(time.delta());

    if !mini_boss_timer.timer.just_finished() {
        return;
    }

    if mini_boss_query.iter().count() > 0 || boss_query.iter().count() > 0 {
        return;
    }

    let Ok((player_transform, level)) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(300.0..500.0);
    let spawn_pos = player_pos + Vec2::from_angle(angle) * distance;

    let tier_scale = 1.0 + (map_tier.0 as f32 - 1.0) * 0.4;
    let health_scale = 1.0 + (level.level as f32 - 1.0) * LEVEL_HEALTH_SCALE;
    let damage_scale = 1.0 + (level.level as f32 - 1.0) * LEVEL_DAMAGE_SCALE;

    let size = Vec2::splat(280.0);

    let mini_boss_entity = commands
        .spawn((
            MiniBoss,
            Enemy {
                damage: 40.0 * tier_scale * damage_scale,
                xp_value: 300 * map_tier.0,
                attack_cooldown: Timer::from_seconds(0.7, TimerMode::Once),
                speed: 100.0 + (map_tier.0 as f32 * 5.0),
            },
            ElementalStatus::default(),
            Health {
                current: 2000.0 * tier_scale * health_scale,
                max: 2000.0 * tier_scale * health_scale,
            },
            Velocity(Vec2::ZERO),
            CharacterState::Idle,
            SpriteBundle {
                texture: sprites.orc_idle.clone(),
                sprite: Sprite {
                    color: Color::srgb(0.7, 0.2, 0.2),
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(spawn_pos.extend(6.0)),
                ..default()
            },
            TextureAtlas {
                layout: sprites.layout.clone(),
                index: 0,
            },
            AnimationConfig {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                frame_count: 6,
                state: CharacterState::Idle,
            },
        ))
        .id();

    commands.entity(mini_boss_entity).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(size.x + 8.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.1),
                ..default()
            },
            HealthBar,
        ));
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.85, 0.1, 0.1),
                    custom_size: Some(Vec2::new(size.x + 6.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.2),
                ..default()
            },
            HealthBarFill(size.x + 6.0),
        ));
    });

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "MINI-BOSS!",
                TextStyle {
                    font_size: 40.0,
                    color: Color::srgb(0.9, 0.3, 0.3),
                    ..default()
                },
            ),
            transform: Transform::from_translation(spawn_pos.extend(100.0) + Vec3::Y * 100.0),
            ..default()
        },
        DamageNumber {
            velocity: Vec2::new(0.0, 40.0),
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
        },
    ));
}

pub fn boss_entrance_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut boss_entrance_active: ResMut<BossEntranceActive>,
    mut entrances: Query<(Entity, &mut BossEntrance)>,
    mut darkens: Query<(Entity, &mut Sprite, &ScreenDarken)>,
) {
    for (entity, mut entrance) in entrances.iter_mut() {
        entrance.timer.tick(time.delta());

        match entrance.phase {
            BossEntrancePhase::Darkening => {
                if entrance.timer.fraction() > 0.5 {
                    entrance.phase = BossEntrancePhase::Spawning;
                }
                for (_, mut sprite, _) in darkens.iter_mut() {
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, entrance.timer.fraction() * 0.6);
                }
            }
            BossEntrancePhase::Spawning => {
                for (_, mut sprite, _) in darkens.iter_mut() {
                    let alpha = 0.6 * (1.0 - (entrance.timer.fraction() - 0.5) * 2.0).max(0.0);
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                }
                if entrance.timer.finished() {
                    entrance.phase = BossEntrancePhase::Complete;
                }
            }
            BossEntrancePhase::Complete => {
                for (darken_entity, _, _) in darkens.iter() {
                    commands.entity(darken_entity).despawn();
                }
                commands.entity(entity).despawn();
                boss_entrance_active.0 = false;
            }
        }
    }
}

pub fn boss_death_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut slow_motion: ResMut<SlowMotion>,
    mut boss_death_active: ResMut<BossDeathActive>,
    mut death_effects: Query<(Entity, &mut BossDeathEffect)>,
    mut slain_texts: Query<(Entity, &mut BossSlainText, &mut Transform, &mut Text)>,
) {
    for (entity, mut effect) in death_effects.iter_mut() {
        effect.timer.tick(time.delta());

        if !effect.slow_mo_active {
            slow_motion.active = true;
            slow_motion.time_scale = 0.3;
            slow_motion.timer = Timer::from_seconds(BOSS_DEATH_SLOWMO_DURATION, TimerMode::Once);
            effect.slow_mo_active = true;
        }

        if effect.timer.finished() {
            commands.entity(entity).despawn();
            boss_death_active.0 = false;
        }
    }

    for (entity, mut slain, mut transform, mut text) in slain_texts.iter_mut() {
        slain.timer.tick(time.delta());

        let scale = 1.0 + slain.timer.fraction() * 0.5;
        transform.scale = Vec3::splat(scale);

        if let Some(section) = text.sections.first_mut() {
            let alpha = 1.0 - slain.timer.fraction();
            section.style.color = section.style.color.with_alpha(alpha);
        }

        if slain.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_boss_with_entrance(
    mut commands: Commands,
    time: Res<Time>,
    game_stats: Res<GameStats>,
    map_tier: Res<MapTier>,
    sprites: Res<CharacterSprites>,
    player_query: Query<&Transform, With<Player>>,
    boss_query: Query<&Boss>,
    mut boss_entrance_active: ResMut<BossEntranceActive>,
    entrance_query: Query<&BossEntrance>,
) {
    let spawn_interval = BOSS_SPAWN_INTERVAL;
    let current_time = game_stats.time_survived;
    let last_time = current_time - time.delta_seconds();

    let should_spawn = current_time > 0.1
        && (current_time / spawn_interval).floor() > (last_time / spawn_interval).floor()
        && boss_query.iter().count() == 0
        && entrance_query.iter().count() == 0;

    if !should_spawn {
        return;
    }

    if player_query.get_single().is_err() {
        return;
    }

    boss_entrance_active.0 = true;

    commands.spawn((
        ScreenDarken { alpha: 0.0 },
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::splat(5000.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 50.0)),
            ..default()
        },
    ));

    commands.spawn(BossEntrance {
        timer: Timer::from_seconds(BOSS_ENTRANCE_DURATION, TimerMode::Once),
        phase: BossEntrancePhase::Darkening,
    });

    let spawn_pos = Vec2::ZERO;
    let tier_scale = 1.0 + (map_tier.0 as f32 - 1.0) * 0.5;

    commands.spawn((
        Boss,
        Enemy {
            damage: 80.0 * tier_scale,
            xp_value: 1000 * map_tier.0,
            attack_cooldown: Timer::from_seconds(0.6, TimerMode::Once),
            speed: 140.0 + (map_tier.0 as f32 * 8.0),
        },
        ElementalStatus::default(),
        Health {
            current: 8000.0 * tier_scale,
            max: 8000.0 * tier_scale,
        },
        Velocity(Vec2::ZERO),
        CharacterState::Idle,
        SpriteBundle {
            texture: sprites.orc_idle.clone(),
            sprite: Sprite {
                color: Color::srgb(0.5, 0.1, 0.1),
                custom_size: Some(Vec2::splat(400.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_pos.extend(6.0)),
            ..default()
        },
        TextureAtlas {
            layout: sprites.layout.clone(),
            index: 0,
        },
        AnimationConfig {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            frame_count: 6,
            state: CharacterState::Idle,
        },
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "BOSS INCOMING!",
                TextStyle {
                    font_size: 56.0,
                    color: Color::srgb(1.0, 0.2, 0.2),
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 150.0, 100.0)),
            ..default()
        },
        DamageNumber {
            velocity: Vec2::new(0.0, 20.0),
            lifetime: Timer::from_seconds(2.5, TimerMode::Once),
        },
    ));
}

pub fn handle_boss_death(
    mut commands: Commands,
    boss_query: Query<(Entity, &Health, &Transform, &Enemy), With<Boss>>,
    mut map_tier: ResMut<MapTier>,
    mut xp_events: EventWriter<SpawnXpOrbEvent>,
    mut boss_death_active: ResMut<BossDeathActive>,
    mut camera_shake: Query<&mut CameraShake, With<Camera2d>>,
) {
    for (entity, health, transform, enemy) in boss_query.iter() {
        if health.current <= 0.0 {
            map_tier.0 += 1;

            for i in 0..20 {
                let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
                let offset = Vec2::from_angle(angle) * 50.0;
                xp_events.send(SpawnXpOrbEvent {
                    position: transform.translation + offset.extend(0.0),
                    value: enemy.xp_value / 20,
                });
            }

            commands.spawn((
                Loot,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 0.8, 0.0),
                        custom_size: Some(Vec2::splat(25.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                },
            ));

            boss_death_active.0 = true;

            commands.spawn(BossDeathEffect {
                timer: Timer::from_seconds(BOSS_DEATH_SLOWMO_DURATION + 0.5, TimerMode::Once),
                slow_mo_active: false,
            });

            commands.spawn((
                BossSlainText {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                },
                Text2dBundle {
                    text: Text::from_section(
                        "BOSS SLAIN!",
                        TextStyle {
                            font_size: 72.0,
                            color: Color::srgb(1.0, 0.8, 0.2),
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(
                        transform.translation + Vec3::new(0.0, 0.0, 100.0),
                    ),
                    ..default()
                },
            ));

            if let Ok(mut shake) = camera_shake.get_single_mut() {
                shake.trauma = 1.0;
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}
