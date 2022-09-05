use bevy::{prelude::*, time::FixedTimestep};

use crate::{
    components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity, Health},
    GameTextures, WinSize, BASE_SPEED, PLAYER_LASER_SIZE, PLAYER_SIZE, PLAYER_SPRITE, SPRITE_SCALE,
    TIME_STEP, PlayerState, PLAYER_RESPAWN_DELAY,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(PlayerState::default())
        .add_system_set(SystemSet::new().with_run_criteria(FixedTimestep::step(0.5)).with_system(player_spawn_system))
        .add_system(player_keyboard_event_system)
        // .add_system_set(SystemSet::new()
        // .with_run_criteria(FixedTimestep::step(0.))
        // .with_system(player_fire_system));
        .add_system(player_fire_system);
    }
}

fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_shot;

    if !player_state.on && (last_shot == -1. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        // add player
        let bottom = -win_size.h / 2.;
        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.player.clone(),
                transform: Transform {
                    translation: Vec3::new(0., bottom + PLAYER_SIZE.1 / 2. * SPRITE_SCALE + 5., 10.),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..Default::default()
                },
                // sprite: Sprite {
                //     color: Color::rgb(0.25,0.25,0.9),
                //     custom_size: Some(Vec2::new(150.,150.)),
                //     ..Default::default()
                // },
                ..Default::default()
            })
            .insert(Player)
            // .insert(Health{hp: 3.,extra:0.})
            .insert(SpriteSize::from(PLAYER_SIZE)) // 
            .insert(Movable {
                auto_despawn: false,
            })
            .insert(Velocity { x: 0., y: 0. });
        player_state.spawned(time.seconds_since_startup());
        player_state.health = Health {hp: 3., extra: 0.};
    }
    
}

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    win_size: Res<WinSize>,
    mut query: Query<(&mut Velocity, &Transform), With<Player>>,
) {
    if let Ok((mut velocity, transform)) = query.get_single_mut() {
        if kb.pressed(KeyCode::Left) { 
            // println!("{:?}{:?}",transform.translation.x,win_size.w);
            if transform.translation.x > -win_size.w/2. {
                if velocity.x > 0. {
                    velocity.x = 0.
                }
                if velocity.x > -0.5 {
                    velocity.x -= 0.5
                } else if velocity.x > -1. {
                    velocity.x -= 0.05
                }
            } else {
                velocity.x = 0.
            }
        } else if kb.pressed(KeyCode::Right) {
            // println!("{:?}{:?}",transform.translation.x,win_size.w);
            if transform.translation.x < win_size.w/2. {
                if velocity.x < 0. {
                    velocity.x = 0.
                }
                if velocity.x < 0.5 {
                    velocity.x += 0.5
                } else if velocity.x < 1. {
                    velocity.x += 0.05
                }
            } else {
                velocity.x = 0.
            }
        } else {
            velocity.x = 0.
            // if velocity.x > 0. {
            //     velocity.x -= 0.1
            // } else if velocity.x < 0. {
            //     velocity.x += 0.1
            // }
        };
        // velocity.y = if kb.pressed(KeyCode::Up) {
        //     1.
        // } else if kb.pressed(KeyCode::Down) {
        //     -1.
        // } else {
        //     0.
        // };
    }
}

fn player_fire_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    kb: Res<Input<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<(&Transform, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((player_tf, vel)) = query.get_single() {
        if kb.just_pressed(KeyCode::Space) && (time.seconds_since_startup() - player_state.last_fired) > 0.25 {
            player_state.last_fired = time.seconds_since_startup();
            // println!("{:?}",player_state.fire_cooldown.elapsed_secs());
            let (x, y) = (player_tf.translation.x, player_tf.translation.y);
            let x_offset: f32 = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

            let mut spawn_laser = |x_offset: f32| {
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: game_textures.player_laser.clone(),
                        transform: Transform {
                            translation: Vec3::new(x + x_offset, y + 15., 0.),
                            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(FromPlayer)
                    .insert(Laser)
                    .insert(SpriteSize::from(PLAYER_LASER_SIZE))
                    .insert(Movable { auto_despawn: true })
                    .insert(Velocity { x: vel.x/2., y: 2. });
            };

            spawn_laser(x_offset);
            spawn_laser(-x_offset);
            // spawn_laser(0.);
            // player_state.fire_cooldown.reset();
        }
    }
}
