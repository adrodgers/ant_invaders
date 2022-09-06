#![allow(unused)]

use std::time::Duration;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::Vec3Swizzles, prelude::*, render::camera::ScalingMode, sprite::collide_aabb::collide,
    text, utils::HashSet, time::Stopwatch,
};
use components::{
    Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromPlayer, Laser, Movable, SpriteSize,
    Velocity, FromEnemy, Player, Health, ScoreText, Damage, ParentEntity
};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

mod components;
mod enemy;
mod player;

// region: --- Asset Constants
const PLAYER_SIZE: (f32, f32) = (144., 75.); //(14., 7.5)
const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);
const PLAYER_RESPAWN_DELAY: f64 = 2.;

const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (9., 54.);

const SPRITE_SCALE: f32 = 0.5;

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const ENEMY_MAX: u32 = 5;
const FORMATION_MEMBERS_MAX: u32 = 1;
// endregion: --- Asset Constants

// region:    --- Game Constants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;
// endregion: --- Game Constants

// region: --- Resources
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}
struct GameTextures {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>,
}

struct EnemyCount(u32);

#[derive(Component)]
struct LastFired {
    time: f64,
    rate: f64
}

struct PlayerState {
    on: bool,
    last_shot: f64,
    health: Health, // -1. if last hit
    last_fired: f64,
    last_spawned: f64,
    score: f64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self { on: false, last_shot: -1. , health: Health{hp: 3., multiplier: 0.}, last_fired: -1., last_spawned: -1., score: 0.}
    }
}

impl PlayerState {
    fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }
    pub fn spawned(&mut self, time:f64) {
        self.on = true;
        self.last_shot = -1.;
        self.last_spawned = time
    }
    fn fired(&mut self, time: f64) {
        self.last_fired = time;
    }
}
// endregion: --- Resources

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Ant Invaders".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup_system)
        .add_system(movable_system)
        .add_system(player_laser_hit_enemy_system)
        .add_system(explosion_to_spawn_system)
        .add_system(explosion_animation_system)
        .add_system(enemy_laser_hit_player_system)
        .add_system(text_score_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>,
    player_state: Res<PlayerState>,
) {
    // camera
    commands.spawn_bundle(Camera2dBundle::default()).insert(UiCameraConfig {
        show_ui: true,
        ..default()
    });

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            // Use the `Text::with_section` constructor
            text: Text::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                player_state.score.to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                
            ).with_alignment(TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..default()
            },),
            ..default()
        })
        .insert(ScoreText);

    // capture window size
    let window = windows.get_primary_mut().unwrap();
    let (win_w, win_h) = (window.width(), window.height());

    // position window
    // window.set_position(IVec2::new(2780, 4900));

    // add winsize
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // load explosion
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
    let explosion = texture_atlases.add(texture_atlas);

    // add game textures
    let game_textures: GameTextures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * BASE_SPEED * TIME_STEP;
        translation.y += velocity.y * BASE_SPEED * TIME_STEP;

        if movable.auto_despawn {
            const MARGIN: f32 = 200.;
            if translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    mut player_state: ResMut<PlayerState>,
    laser_query: Query<(Entity, &Transform, &SpriteSize, &Damage), (With<FromPlayer>, With<Laser>)>,
    mut enemy_query: Query<(Entity, &Transform, &SpriteSize, &mut Health), With<Enemy>>,
) {
    let mut despwaned_entities: HashSet<Entity> = HashSet::new();
    for (laser_entity, laser_tf, laser_size, laser_damage) in laser_query.iter() {
        if despwaned_entities.contains(&laser_entity) {
            continue;
        }
        let laser_scale = Vec2::from(laser_tf.scale.xy());
        for (enemy_entity, enemy_tf, enemy_size, mut enemy_health) in enemy_query.iter_mut() {
            if despwaned_entities.contains(&laser_entity)
                || despwaned_entities.contains(&enemy_entity)
            {
                continue;
            }
            let enemy_scale = Vec2::from(enemy_tf.scale.xy());
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );
            if let Some(_) = collision {
                enemy_health.hp -= laser_damage.dmg;
                if enemy_health.hp <= 0. {
                    commands.entity(enemy_entity).despawn();
                    despwaned_entities.insert(enemy_entity);
                    enemy_count.0 -= 1;

                    commands.entity(laser_entity).despawn();
                    despwaned_entities.insert(laser_entity);

                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_tf.translation.clone()));
                    player_state.score += 1.;

                } else {
                    commands.entity(laser_entity).despawn();
                    despwaned_entities.insert(laser_entity);
                }
            }
        }
    }
}

fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &SpriteSize, &Damage, &FromEnemy), (With<FromEnemy>,With<Laser>)>,
    mut player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
    if time.seconds_since_startup() - player_state.last_spawned > 2. {
        if let Ok((player_entity, player_tf, player_size,)) = player_query.get_single_mut() {
            let player_scale = Vec2::from(player_tf.scale.xy());
            for (laser_entity, laser_tf, laser_size, laser_damage, from_enemy) in laser_query.iter() {
                let laser_scale = Vec2::from(laser_tf.scale.xy());
                // Detect collision
                let collision = collide(
                    laser_tf.translation,
                    laser_size.0 * laser_scale,
                    player_tf.translation,
                    player_size.0 * player_scale
                );

                if let Some(_) = collision {
                    player_state.health.hp -= laser_damage.damage_dealt();
                    if player_state.health.hp <= 0. {
                        commands.entity(player_entity).despawn();
                        player_state.shot(time.seconds_since_startup());
                        commands.entity(laser_entity).despawn();
                        commands.spawn().insert(ExplosionToSpawn(player_tf.translation.clone()));
                    break;
                    } else {
                        commands.entity(laser_entity).despawn();
                    }
                }
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_to_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_textures.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        commands.entity(explosion_to_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1; // Move to next sprite
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn text_score_system(time: Res<Time>, player_state: Res<PlayerState>, mut query: Query<&mut Text, With<ScoreText>>) {
    for mut text in query.iter_mut() {
        // let seconds = time.seconds_since_startup() as f32;
        // // We used the `Text::with_section` helper method, but it is still just a `Text`,
        // // so to update it, we are still updating the one and only section
        // text.sections[0].style.color = Color::Rgba {
        //     red: (1.25 * seconds).sin() / 2.0 + 0.5,
        //     green: (0.75 * seconds).sin() / 2.0 + 0.5,
        //     blue: (0.50 * seconds).sin() / 2.0 + 0.5,
        //     alpha: 1.0,
        // };
        text.sections[0].value = player_state.score.to_string();
    }
}

fn enemy_enrage_system(time: Res<Time>, mut query: Query<&mut Enemy, With<Enemy>>) {

}
