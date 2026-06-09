//! NOCTURNE — an arena shooter where you never choose your weapon.
//! Five moonlight arms in a fixed queue; when one runs dry it rotates out.
//! You don't pick. You plan.

use std::collections::VecDeque;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::render::view::Hdr;
use bevy::prelude::*;
use rand::Rng;

// Crimson Elite palette. HDR values (>1.0) so bright things bloom into light.
const BG: Color = Color::srgb(0.025, 0.02, 0.03); // warm black
const CRIMSON: Color = Color::linear_rgb(5.0, 0.35, 0.9);
const GOLD: Color = Color::linear_rgb(6.0, 4.0, 1.2);
const HUSK: Color = Color::linear_rgb(1.3, 1.0, 2.4); // moonlit violet, gently glowing
const GRID: Color = Color::srgb(0.10, 0.07, 0.13);
const BORDER: Color = Color::linear_rgb(2.2, 0.5, 1.0);

const PLAYER_SPEED: f32 = 320.0;
const ARENA_HALF: Vec2 = Vec2::new(610.0, 330.0);
const HUSK_SIZE: f32 = 22.0;
const HUSK_SPEED: f32 = 110.0;

// ---------------------------------------------------------------- weapons

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WeaponKind {
    Lumen,    // baseline rifle — steady gold rounds
    Pierce,   // railgun lance — slow, heavy, goes through everything
    Scatter,  // moonburst — 5 pellets in a cone, short range
    Graven,   // gravity orb — slow projectile that chills enemies it hits
    Crescent, // returning blade — boomerangs back, hits both ways
}

struct WeaponStats {
    name: &'static str,
    ammo: u32,
    cooldown: f32,
    speed: f32,
    ttl: f32,
    damage: i32,
    pierce: bool,
    boomerang: bool,
    slow: bool,
    pellets: u32,
    spread: f32, // radians, total cone width
    color: Color,
    size: Vec2,
}

impl WeaponKind {
    fn stats(self) -> WeaponStats {
        match self {
            WeaponKind::Lumen => WeaponStats {
                name: "LUMEN",
                ammo: 18,
                cooldown: 0.18,
                speed: 900.0,
                ttl: 1.2,
                damage: 1,
                pierce: false,
                boomerang: false,
                slow: false,
                pellets: 1,
                spread: 0.0,
                color: GOLD,
                size: Vec2::new(14.0, 5.0),
            },
            WeaponKind::Pierce => WeaponStats {
                name: "PIERCE",
                ammo: 6,
                cooldown: 0.6,
                speed: 1400.0,
                ttl: 1.0,
                damage: 3,
                pierce: true,
                boomerang: false,
                slow: false,
                pellets: 1,
                spread: 0.0,
                color: Color::linear_rgb(2.2, 4.5, 7.0), // pale ice
                size: Vec2::new(34.0, 4.0),
            },
            WeaponKind::Scatter => WeaponStats {
                name: "SCATTER",
                ammo: 8,
                cooldown: 0.45,
                speed: 750.0,
                ttl: 0.42,
                damage: 1,
                pierce: false,
                boomerang: false,
                slow: false,
                pellets: 5,
                spread: 0.55,
                color: Color::linear_rgb(7.0, 2.0, 0.4), // ember orange
                size: Vec2::new(9.0, 5.0),
            },
            WeaponKind::Graven => WeaponStats {
                name: "GRAVEN",
                ammo: 10,
                cooldown: 0.32,
                speed: 520.0,
                ttl: 1.6,
                damage: 1,
                pierce: false,
                boomerang: false,
                slow: true,
                pellets: 1,
                spread: 0.0,
                color: Color::linear_rgb(2.5, 1.0, 7.0), // deep violet
                size: Vec2::splat(10.0),
            },
            WeaponKind::Crescent => WeaponStats {
                name: "CRESCENT",
                ammo: 6,
                cooldown: 0.55,
                speed: 800.0,
                ttl: 0.55, // outbound time; then it comes home
                damage: 2,
                pierce: true,
                boomerang: true,
                slow: false,
                pellets: 1,
                spread: 0.0,
                color: Color::linear_rgb(5.5, 5.5, 6.5), // moon silver
                size: Vec2::new(18.0, 8.0),
            },
        }
    }
}

#[derive(Resource)]
struct Armory {
    queue: VecDeque<WeaponKind>,
    ammo: u32,
}

impl Default for Armory {
    fn default() -> Self {
        let queue: VecDeque<WeaponKind> = [
            WeaponKind::Lumen,
            WeaponKind::Pierce,
            WeaponKind::Scatter,
            WeaponKind::Graven,
            WeaponKind::Crescent,
        ]
        .into();
        let ammo = queue[0].stats().ammo;
        Armory { queue, ammo }
    }
}

impl Armory {
    fn current(&self) -> WeaponKind {
        self.queue[0]
    }

    /// Spend one round; rotate to the next weapon when dry. Returns true if rotated.
    fn spend(&mut self) -> bool {
        self.ammo = self.ammo.saturating_sub(1);
        if self.ammo == 0 {
            self.queue.rotate_left(1);
            self.ammo = self.queue[0].stats().ammo;
            info!("rotated to {}", self.queue[0].stats().name);
            return true;
        }
        false
    }
}

// ---------------------------------------------------------------- components

#[derive(Component)]
struct Player {
    hp: i32,
    invuln: f32,
}

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum Phase {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
struct GameStats {
    kills: u32,
    elapsed: f32,
}

#[derive(Component)]
struct StatsHud;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct Projectile {
    velocity: Vec2,
    ttl: f32,
    damage: i32,
    pierce: bool,
    boomerang: bool,
    returning: bool,
    slow: bool,
    already_hit: Vec<Entity>,
}

#[derive(Component)]
struct Enemy {
    hp: i32,
    slow_left: f32,
    flash: f32,
}

/// Short-lived fading sprite — projectile trails and burst debris.
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    ttl: f32,
    life: f32,
    spin: f32,
}

#[derive(Component)]
struct WeaponHud;

#[derive(Resource)]
struct FireCooldown(Timer);

#[derive(Resource)]
struct EnemySpawner(Timer);

// ---------------------------------------------------------------- app

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
        .init_resource::<Armory>()
        .init_resource::<GameStats>()
        .init_state::<Phase>()
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
                contact_damage,
                update_particles,
                update_hud,
            )
                .run_if(in_state(Phase::Playing)),
        )
        .add_systems(OnEnter(Phase::GameOver), show_game_over)
        .add_systems(Update, restart.run_if(in_state(Phase::GameOver)))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        Tonemapping::AcesFitted,
        Bloom {
            intensity: 0.28,
            ..Bloom::NATURAL
        },
    ));

    // Arena floor: a faint grid so the space reads as a place, not a void.
    let step = 61.0;
    let mut x = -ARENA_HALF.x;
    while x <= ARENA_HALF.x + 0.1 {
        commands.spawn((
            Sprite::from_color(GRID, Vec2::new(1.5, ARENA_HALF.y * 2.0)),
            Transform::from_xyz(x, 0.0, -1.0),
        ));
        x += step;
    }
    let mut y = -ARENA_HALF.y;
    while y <= ARENA_HALF.y + 0.1 {
        commands.spawn((
            Sprite::from_color(GRID, Vec2::new(ARENA_HALF.x * 2.0, 1.5)),
            Transform::from_xyz(0.0, y, -1.0),
        ));
        y += step;
    }
    // Glowing arena border (four bars).
    let t = 4.0;
    for (size, pos) in [
        (Vec2::new(ARENA_HALF.x * 2.0 + t, t), Vec2::new(0.0, ARENA_HALF.y)),
        (Vec2::new(ARENA_HALF.x * 2.0 + t, t), Vec2::new(0.0, -ARENA_HALF.y)),
        (Vec2::new(t, ARENA_HALF.y * 2.0 + t), Vec2::new(ARENA_HALF.x, 0.0)),
        (Vec2::new(t, ARENA_HALF.y * 2.0 + t), Vec2::new(-ARENA_HALF.x, 0.0)),
    ] {
        commands.spawn((
            Sprite::from_color(BORDER, size),
            Transform::from_translation(pos.extend(-0.5)),
        ));
    }

    commands.spawn((
        Player { hp: 3, invuln: 0.0 },
        Sprite::from_color(CRIMSON, Vec2::splat(26.0)),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    commands.spawn((
        WeaponHud,
        Text2d::new(""),
        TextFont::from_font_size(22.0),
        TextColor(GOLD),
        Transform::from_xyz(0.0, ARENA_HALF.y + 18.0, 5.0),
    ));

    commands.spawn((
        StatsHud,
        Text2d::new("HP ### | KILLS 0"),
        TextFont::from_font_size(20.0),
        TextColor(CRIMSON),
        Transform::from_xyz(-ARENA_HALF.x + 90.0, -ARENA_HALF.y - 18.0, 5.0),
    ));
}

// ---------------------------------------------------------------- systems

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
    transform.translation.x =
        (transform.translation.x + delta.x).clamp(-ARENA_HALF.x, ARENA_HALF.x);
    transform.translation.y =
        (transform.translation.y + delta.y).clamp(-ARENA_HALF.y, ARENA_HALF.y);
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
    mut armory: ResMut<Armory>,
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
    let aim = (target - origin).normalize_or_zero();
    if aim == Vec2::ZERO {
        return;
    }

    let stats = armory.current().stats();
    let base_angle = aim.y.atan2(aim.x);

    for i in 0..stats.pellets {
        let angle = if stats.pellets > 1 {
            base_angle - stats.spread / 2.0
                + stats.spread * (i as f32 / (stats.pellets - 1) as f32)
        } else {
            base_angle
        };
        let dir = Vec2::from_angle(angle);
        commands.spawn((
            Projectile {
                velocity: dir * stats.speed,
                ttl: stats.ttl,
                damage: stats.damage,
                pierce: stats.pierce,
                boomerang: stats.boomerang,
                returning: false,
                slow: stats.slow,
                already_hit: Vec::new(),
            },
            Sprite::from_color(stats.color, stats.size),
            Transform::from_translation(origin.extend(0.5))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }

    // One trigger pull = one round, even for multi-pellet weapons.
    let rotated = armory.spend();
    let next_cooldown = armory.current().stats().cooldown;
    cooldown.0 = Timer::from_seconds(
        if rotated { next_cooldown.max(0.35) } else { stats.cooldown },
        TimerMode::Once,
    );
}

fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut projectiles: Query<(Entity, &mut Projectile, &mut Transform), Without<Player>>,
) {
    let player_pos = player
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (entity, mut proj, mut transform) in &mut projectiles {
        proj.ttl -= time.delta_secs();
        let pos = transform.translation.truncate();

        if proj.ttl <= 0.0 {
            if proj.boomerang && !proj.returning {
                // Turn the blade around: head back to the player.
                proj.returning = true;
                proj.ttl = 2.0;
                proj.already_hit.clear();
            } else {
                commands.entity(entity).despawn();
                continue;
            }
        }

        if proj.returning {
            let home = (player_pos - pos).normalize_or_zero();
            let speed = proj.velocity.length();
            proj.velocity = home * speed;
            if pos.distance(player_pos) < 20.0 {
                commands.entity(entity).despawn();
                continue;
            }
        }

        let next = pos + proj.velocity * time.delta_secs();
        if next.x.abs() > ARENA_HALF.x + 120.0 || next.y.abs() > ARENA_HALF.y + 120.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = next.extend(0.5);
        let v = proj.velocity;
        transform.rotation = Quat::from_rotation_z(v.y.atan2(v.x));

        // Leave a fading ember trail behind fast shots.
        commands.spawn((
            Particle {
                velocity: -v * 0.05,
                ttl: 0.18,
                life: 0.18,
                spin: 0.0,
            },
            Sprite::from_color(sprite_color(&proj), Vec2::splat(5.0)),
            Transform::from_translation(pos.extend(0.4)),
        ));
    }
}

/// Trail color keyed off the projectile's role (returning blades cool down).
fn sprite_color(proj: &Projectile) -> Color {
    if proj.returning {
        Color::linear_rgb(3.0, 3.0, 4.0)
    } else if proj.slow {
        Color::linear_rgb(2.0, 0.8, 5.0)
    } else if proj.pierce {
        Color::linear_rgb(1.8, 3.5, 5.5)
    } else {
        Color::linear_rgb(5.0, 3.2, 1.0)
    }
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
    mut stats: ResMut<GameStats>,
) {
    stats.elapsed += time.delta_secs();
    spawner.0.tick(time.delta());
    if !spawner.0.just_finished() {
        return;
    }
    // Pressure ramps: spawn interval shrinks from 1.4s toward 0.45s over ~2 min.
    let interval = (1.4 - stats.elapsed * 0.008).max(0.45);
    spawner.0.set_duration(std::time::Duration::from_secs_f32(interval));
    let mut rng = rand::thread_rng();
    let pos = match rng.gen_range(0..4) {
        0 => Vec2::new(rng.gen_range(-ARENA_HALF.x..ARENA_HALF.x), ARENA_HALF.y),
        1 => Vec2::new(rng.gen_range(-ARENA_HALF.x..ARENA_HALF.x), -ARENA_HALF.y),
        2 => Vec2::new(ARENA_HALF.x, rng.gen_range(-ARENA_HALF.y..ARENA_HALF.y)),
        _ => Vec2::new(-ARENA_HALF.x, rng.gen_range(-ARENA_HALF.y..ARENA_HALF.y)),
    };
    commands.spawn((
        Enemy {
            hp: 2,
            slow_left: 0.0,
            flash: 0.0,
        },
        // Rotated 45° so it reads as a diamond husk, not a box.
        Sprite::from_color(HUSK, Vec2::splat(HUSK_SIZE)),
        Transform::from_translation(pos.extend(0.8))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
}

/// Spray a short burst of fading debris from a point.
fn spawn_burst(commands: &mut Commands, at: Vec2, color: Color, count: u32, spread: f32) {
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(60.0..spread);
        let life = rng.gen_range(0.25..0.6);
        let size = rng.gen_range(3.0..7.0);
        commands.spawn((
            Particle {
                velocity: Vec2::from_angle(angle) * speed,
                ttl: life,
                life,
                spin: rng.gen_range(-12.0..12.0),
            },
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_translation(at.extend(0.6)),
        ));
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut tf, mut sprite) in &mut particles {
        p.ttl -= dt;
        if p.ttl <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation += (p.velocity * dt).extend(0.0);
        p.velocity *= 1.0 - 3.0 * dt; // drag
        tf.rotate_z(p.spin * dt);
        let k = (p.ttl / p.life).clamp(0.0, 1.0);
        tf.scale = Vec3::splat(k.max(0.15));
        sprite.color = sprite.color.with_alpha(k);
    }
}

fn chase_player(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &mut Enemy, &mut Sprite), Without<Player>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let target = player_tf.translation.truncate();
    let dt = time.delta_secs();
    for (mut tf, mut enemy, mut sprite) in &mut enemies {
        let speed = if enemy.slow_left > 0.0 {
            enemy.slow_left -= dt;
            sprite.color = Color::linear_rgb(0.8, 1.2, 4.0); // chilled tint
            HUSK_SPEED * 0.35
        } else {
            sprite.color = HUSK;
            HUSK_SPEED
        };
        // White hit-flash briefly overrides the body color.
        if enemy.flash > 0.0 {
            enemy.flash -= dt;
            sprite.color = Color::linear_rgb(8.0, 8.0, 9.0);
        }
        let pos = tf.translation.truncate();
        let dir = (target - pos).normalize_or_zero();
        tf.translation += (dir * speed * dt).extend(0.0);
        tf.rotate_z(1.6 * dt); // slow spin
    }
}

fn projectile_hits(
    mut commands: Commands,
    mut stats: ResMut<GameStats>,
    mut projectiles: Query<(Entity, &Transform, &mut Projectile)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
) {
    for (proj_entity, proj_tf, mut proj) in &mut projectiles {
        let proj_pos = proj_tf.translation.truncate();
        let mut despawned = false;
        for (enemy_entity, enemy_tf, mut enemy) in &mut enemies {
            if proj.already_hit.contains(&enemy_entity) {
                continue;
            }
            let dist = proj_pos.distance(enemy_tf.translation.truncate());
            if dist < HUSK_SIZE * 0.5 + 7.0 {
                enemy.hp -= proj.damage;
                enemy.flash = 0.08;
                let hit_at = enemy_tf.translation.truncate();
                if proj.slow {
                    enemy.slow_left = 2.5;
                }
                if enemy.hp <= 0 {
                    commands.entity(enemy_entity).despawn();
                    stats.kills += 1;
                    spawn_burst(&mut commands, hit_at, HUSK, 14, 260.0);
                    info!("husk down");
                } else {
                    spawn_burst(&mut commands, hit_at, GOLD, 4, 120.0);
                }
                if proj.pierce {
                    proj.already_hit.push(enemy_entity);
                } else {
                    commands.entity(proj_entity).despawn();
                    despawned = true;
                    break;
                }
            }
        }
        if despawned {
            continue;
        }
    }
}

fn contact_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut player: Query<(&mut Player, &Transform, &mut Sprite)>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    let Ok((mut player, player_tf, mut sprite)) = player.single_mut() else {
        return;
    };
    if player.invuln > 0.0 {
        player.invuln -= time.delta_secs();
        // Flicker while invulnerable
        sprite.color = if (player.invuln * 12.0) as i32 % 2 == 0 {
            CRIMSON
        } else {
            Color::linear_rgb(0.6, 0.08, 0.2)
        };
        return;
    }
    sprite.color = CRIMSON;

    let pos = player_tf.translation.truncate();
    for enemy_tf in &enemies {
        if pos.distance(enemy_tf.translation.truncate()) < HUSK_SIZE * 0.5 + 13.0 {
            player.hp -= 1;
            player.invuln = 1.0;
            spawn_burst(&mut commands, pos, CRIMSON, 20, 320.0);
            info!("player hit, hp={}", player.hp);
            if player.hp <= 0 {
                next_phase.set(Phase::GameOver);
            }
            break;
        }
    }
}

fn update_hud(
    armory: Res<Armory>,
    stats: Res<GameStats>,
    player: Query<&Player>,
    mut weapon_hud: Query<&mut Text2d, (With<WeaponHud>, Without<StatsHud>)>,
    mut stats_hud: Query<&mut Text2d, (With<StatsHud>, Without<WeaponHud>)>,
) {
    if let Ok(mut text) = weapon_hud.single_mut() {
        let mut line = format!("[{} {:02}]", armory.current().stats().name, armory.ammo);
        for kind in armory.queue.iter().skip(1) {
            line.push_str(&format!("  >  {}", kind.stats().name));
        }
        text.0 = line;
    }
    if let (Ok(mut text), Ok(player)) = (stats_hud.single_mut(), player.single()) {
        let hearts = "#".repeat(player.hp.max(0) as usize);
        text.0 = format!("HP {:3} | KILLS {}", hearts, stats.kills);
    }
}

fn show_game_over(mut commands: Commands, stats: Res<GameStats>) {
    commands.spawn((
        GameOverText,
        Text2d::new(format!(
            "ECLIPSED\n{} husks down in {:.0}s\npress R to rise again",
            stats.kills, stats.elapsed
        )),
        TextFont::from_font_size(42.0),
        TextColor(CRIMSON),
        Transform::from_xyz(0.0, 60.0, 6.0),
    ));
}

#[allow(clippy::too_many_arguments)]
fn restart(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut armory: ResMut<Armory>,
    mut stats: ResMut<GameStats>,
    mut spawner: ResMut<EnemySpawner>,
    mut player: Query<(&mut Player, &mut Transform)>,
    cleanup: Query<
        Entity,
        Or<(With<Enemy>, With<Projectile>, With<Particle>, With<GameOverText>)>,
    >,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    for entity in &cleanup {
        commands.entity(entity).despawn();
    }
    *armory = Armory::default();
    *stats = GameStats::default();
    spawner.0 = Timer::from_seconds(1.4, TimerMode::Repeating);
    if let Ok((mut player, mut tf)) = player.single_mut() {
        player.hp = 3;
        player.invuln = 1.0;
        tf.translation = Vec3::new(0.0, 0.0, 1.0);
    }
    next_phase.set(Phase::Playing);
}
