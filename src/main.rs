//! NOCTURNE — an arena shooter where you never choose your weapon.
//! Five moonlight arms in a fixed queue; when one runs dry it rotates out.
//! You don't pick. You plan.

use bevy::prelude::*;
use rand::Rng;

// Crimson Elite palette
const BG: Color = Color::srgb(0.04, 0.03, 0.04); // warm black
const CRIMSON: Color = Color::srgb(0.86, 0.08, 0.24);

const GOLD: Color = Color::srgb(0.95, 0.78, 0.25);

const PLAYER_SPEED: f32 = 320.0;
const ARENA_HALF: Vec2 = Vec2::new(610.0, 330.0);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Projectile {
    velocity: Vec2,
    ttl: f32,
}

#[derive(Resource)]
struct FireCooldown(Timer);

const HUSK: Color = Color::srgb(0.61, 0.57, 0.72); // pale moonlit gray-violet
const HUSK_SIZE: f32 = 22.0;
const HUSK_SPEED: f32 = 110.0;

#[derive(Component)]
struct Enemy {
    hp: i32,
}

#[derive(Resource)]
struct EnemySpawner(Timer);

fn main() {
    App::new()
        .insert_resource(ClearColor(BG))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NOCTURNE".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(FireCooldown(Timer::from_seconds(0.18, TimerMode::Once)))
        .insert_resource(EnemySpawner(Timer::from_seconds(1.4, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                fire_weapon,
                move_projectiles,
                spawn_enemies,
                chase_player,
                projectile_hits,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Player,
        Sprite::from_color(CRIMSON, Vec2::splat(26.0)),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    let delta = dir.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
    transform.translation.x = (transform.translation.x + delta.x).clamp(-ARENA_HALF.x, ARENA_HALF.x);
    transform.translation.y = (transform.translation.y + delta.y).clamp(-ARENA_HALF.y, ARENA_HALF.y);
}

/// World-space position of the mouse cursor, if it's inside the window.
fn cursor_world_pos(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn fire_weapon(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut cooldown: ResMut<FireCooldown>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    player: Query<&Transform, With<Player>>,
) {
    cooldown.0.tick(time.delta());
    if !buttons.pressed(MouseButton::Left) || !cooldown.0.is_finished() {
        return;
    }
    let (Ok(window), Ok((cam, cam_tf)), Ok(player_tf)) =
        (windows.single(), camera.single(), player.single())
    else {
        return;
    };
    let Some(target) = cursor_world_pos(window, cam, cam_tf) else {
        return;
    };

    let origin = player_tf.translation.truncate();
    let dir = (target - origin).normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }

    commands.spawn((
        Projectile {
            velocity: dir * 900.0,
            ttl: 1.2,
        },
        Sprite::from_color(GOLD, Vec2::new(14.0, 5.0)),
        Transform::from_translation(origin.extend(0.5))
            .with_rotation(Quat::from_rotation_z(dir.y.atan2(dir.x))),
    ));
    cooldown.0.reset();
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
) {
    spawner.0.tick(time.delta());
    if !spawner.0.just_finished() {
        return;
    }
    let mut rng = rand::thread_rng();
    // Spawn on a random edge of the arena
    let pos = match rng.gen_range(0..4) {
        0 => Vec2::new(rng.gen_range(-ARENA_HALF.x..ARENA_HALF.x), ARENA_HALF.y),
        1 => Vec2::new(rng.gen_range(-ARENA_HALF.x..ARENA_HALF.x), -ARENA_HALF.y),
        2 => Vec2::new(ARENA_HALF.x, rng.gen_range(-ARENA_HALF.y..ARENA_HALF.y)),
        _ => Vec2::new(-ARENA_HALF.x, rng.gen_range(-ARENA_HALF.y..ARENA_HALF.y)),
    };
    commands.spawn((
        Enemy { hp: 2 },
        Sprite::from_color(HUSK, Vec2::splat(HUSK_SIZE)),
        Transform::from_translation(pos.extend(0.8)),
    ));
}

fn chase_player(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let target = player_tf.translation.truncate();
    for mut tf in &mut enemies {
        let pos = tf.translation.truncate();
        let dir = (target - pos).normalize_or_zero();
        tf.translation += (dir * HUSK_SPEED * time.delta_secs()).extend(0.0);
    }
}

fn projectile_hits(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
) {
    for (proj_entity, proj_tf) in &projectiles {
        for (enemy_entity, enemy_tf, mut enemy) in &mut enemies {
            let dist = proj_tf
                .translation
                .truncate()
                .distance(enemy_tf.translation.truncate());
            if dist < HUSK_SIZE * 0.5 + 7.0 {
                commands.entity(proj_entity).despawn();
                enemy.hp -= 1;
                if enemy.hp <= 0 {
                    commands.entity(enemy_entity).despawn();
                    info!("husk down");
                }
                break;
            }
        }
    }
}

fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Projectile, &mut Transform)>,
) {
    for (entity, mut proj, mut transform) in &mut projectiles {
        proj.ttl -= time.delta_secs();
        let pos = transform.translation.truncate() + proj.velocity * time.delta_secs();
        if proj.ttl <= 0.0 || pos.x.abs() > ARENA_HALF.x + 120.0 || pos.y.abs() > ARENA_HALF.y + 120.0
        {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = pos.extend(0.5);
    }
}
