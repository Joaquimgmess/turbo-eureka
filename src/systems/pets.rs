use crate::components::*;
use crate::events::*;
use bevy::prelude::*;

pub fn spawn_pet(commands: &mut Commands, owner: Entity, pet_type: PetType) {
    let color = match pet_type {
        PetType::Healer => Color::srgb(0.2, 1.0, 0.5),
        PetType::Damager => Color::srgb(1.0, 0.2, 0.2),
        PetType::Buffer => Color::srgb(0.2, 0.5, 1.0),
        PetType::Tanker => Color::srgb(0.7, 0.7, 0.7),
    };

    let mut entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 8.0),
            ..default()
        },
        Pet {
            owner,
            pet_type,
            action_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        },
    ));

    if pet_type == PetType::Tanker {
        entity.insert(Taunt);
    }
}

pub fn update_pets(
    time: Res<Time>,
    owner_query: Query<&Transform, With<Player>>,
    mut pet_query: Query<(&mut Transform, &Pet), Without<Player>>,
) {
    for (mut transform, pet) in pet_query.iter_mut() {
        if let Ok(owner_transform) = owner_query.get(pet.owner) {
            let target = owner_transform.translation.truncate();
            let current = transform.translation.truncate();

            let offset = match pet.pet_type {
                PetType::Healer => Vec2::new(-40.0, 20.0),
                PetType::Damager => Vec2::new(40.0, 20.0),
                PetType::Buffer => Vec2::new(-40.0, -20.0),
                PetType::Tanker => Vec2::new(40.0, -20.0),
            };

            let dest = target + offset;
            let dist = current.distance(dest);

            if dist > 5.0 {
                let dir = (dest - current).normalize();
                let speed = 250.0;
                transform.translation += (dir * speed * time.delta_seconds()).extend(0.0);
            }
        }
    }
}

pub fn pet_actions(
    time: Res<Time>,
    mut pet_query: Query<(Entity, &Transform, &mut Pet)>,
    mut owner_query: Query<(&mut Health, &mut Stats), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (pet_entity, transform, mut pet) in pet_query.iter_mut() {
        pet.action_timer.tick(time.delta());

        if pet.action_timer.just_finished() {
            match pet.pet_type {
                PetType::Healer => {
                    if let Ok((mut health, _)) = owner_query.get_mut(pet.owner) {
                        health.current = (health.current + health.max * 0.01).min(health.max);
                    }
                }
                PetType::Damager => {

                    let pet_pos = transform.translation.truncate();
                    let mut nearest: Option<(Entity, f32)> = None;

                    for (enemy_entity, enemy_transform) in enemy_query.iter() {
                        let dist = pet_pos.distance(enemy_transform.translation.truncate());
                        if dist < 250.0 {
                            if nearest.is_none() || dist < nearest.unwrap().1 {
                                nearest = Some((enemy_entity, dist));
                            }
                        }
                    }

                    if let Some((target, _)) = nearest {
                        damage_events.send(DamageEvent {
                            target,
                            attacker: Some(pet_entity),
                            amount: 10.0,
                            is_crit: false,
                        });
                    }
                }
                PetType::Buffer => {
                    if let Ok((_, mut stats)) = owner_query.get_mut(pet.owner) {
                        stats.armor = (stats.armor).max(25.0);
                        stats.damage = (stats.damage).max(35.0);
                    }
                }
                PetType::Tanker => {

                }
            }
        }
    }
}
