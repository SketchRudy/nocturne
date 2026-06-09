//! NOCTURNE — an arena shooter where you never choose your weapon.
//! Five moonlight arms in a fixed queue; when one runs dry it rotates out.
//! You don't pick. You plan.

use bevy::prelude::*;

// Crimson Elite palette
const BG: Color = Color::srgb(0.04, 0.03, 0.04); // warm black
const CRIMSON: Color = Color::srgb(0.86, 0.08, 0.24);

const PLAYER_SPEED: f32 = 320.0;
const ARENA_HALF: Vec2 = Vec2::new(610.0, 330.0);

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .insert_resource(ClearColor(BG))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NOCTURNE".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
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
