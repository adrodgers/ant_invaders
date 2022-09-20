use std::f32::consts::PI;

use crate::{
    components::{Enemy, FromEnemy, Laser, Movable, SpriteSize, Velocity, Health, Damage, NumberOfHits, ParentEntity, Player},
    EnemyCount, GameTextures, WinSize, ENEMY_LASER_SIZE, ENEMY_MAX, ENEMY_SIZE, SPRITE_SCALE, BASE_SPEED, TIME_STEP, EnemyState, PlayerState,
};
use bevy::{time::FixedTimestep, ecs::schedule::ShouldRun, prelude::*, math::Vec3Swizzles};
use rand::{thread_rng, Rng};

use self::formation::{FormationMaker, Formation};

mod formation;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FormationMaker::default())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.5))
                .with_system(enemy_spawn_system),
        )
        .add_system(enemy_movement_system)
        .add_system(enemy_fire_system);
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut enemy_count: ResMut<EnemyCount>,
    mut formation_maker: ResMut<FormationMaker>,
    win_size: Res<WinSize>,
) {
    if enemy_count.0 < ENEMY_MAX {
        let formation = formation_maker.make(&win_size);
        let (x,y) = formation.start;

        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.enemy.clone(),
                transform: Transform {
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    translation: Vec3::new(x, y, 10.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy)
            .insert(EnemyState::default())
            .insert(formation)
            .insert(SpriteSize::from(ENEMY_SIZE))
            .insert(Health {hp: 2., multiplier: 0.})
            .insert(Velocity{x:0.,y:0.});
            // .insert(LastFired { time:-1., rate: 1.})
            // .insert(NumberOfHits{hits:0});
        enemy_count.0 += 1;
    }
}

// fn enemy_fire_criteria() -> ShouldRun {
//     if thread_rng().gen_bool(1. / 5.) {
//         ShouldRun::Yes
//     } else {
//         ShouldRun::No
//     }
// }

fn enemy_fire_system(
    mut commands: Commands,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    // player_state: Res<PlayerState>,
    mut enemy_query: Query<(Entity, &Transform, &mut Velocity, &mut EnemyState), With<Enemy>>,
    player_query: Query<&Transform, (Without<Enemy>, With<Player>)>,    
    win_size: Res<WinSize>
) {
    for (entity,&enemy_transform, mut velocity, mut enemy_state) in enemy_query.iter_mut() {
        if enemy_state.fire_cooldown.tick(time.delta()).finished() {
            enemy_state.fire_cooldown.reset();
            let mut player_transform = &Transform::from_translation(Vec3::new(0.,0.,0.));
            if player_query.get_single().is_ok() {
                player_transform = player_query.get_single().unwrap();
            };
            let (x, y) = (enemy_transform.translation.x, enemy_transform.translation.y);
            let direction = (player_transform.translation.truncate() - enemy_transform.translation.truncate()).normalize();
            commands
                .spawn_bundle(SpriteBundle {
                    texture: game_textures.enemy_laser.clone(),
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.),
                        rotation: enemy_transform.rotation,
                        scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Laser)
                .insert(Damage{dmg: if time.seconds_since_startup() < 10. {1.} else {10.},multiplier:1.,limit:2.})
                .insert(Movable {auto_despawn: true })
                .insert(FromEnemy)
                .insert(ParentEntity{entity})
                .insert(SpriteSize::from(ENEMY_LASER_SIZE))
                .insert(Velocity {x:direction.x,y:direction.y});
        }
    }
}

fn enemy_movement_system(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut Formation, &mut Velocity), With<Enemy>>,
    mut player_query: Query<&Transform, (With<Player>,Without<Enemy>)>,
    win_size: Res<WinSize>,
) {
    let now = time.seconds_since_startup() as f32;
    let mut rng = thread_rng();
    let w_span = win_size.w / 2. - 100.;
    let h_span = win_size.h / 2. - 100.;
    // for each enemy
    for (mut transform,mut formation, mut velocity) in enemy_query.iter_mut() {
        
        let mut player_transform = &Transform::from_translation(Vec3::new(0.,0.,0.));
        if player_query.get_single().is_ok() {
            player_transform = player_query.get_single().unwrap();
        };

        // let player_transform = player_query.get_single().unwrap_or_else(&Transform::default());
        let (x_org, y_org) = (transform.translation.x, transform.translation.y);
        let max_distance = TIME_STEP * formation.speed;
        // let dir:i32 = rng.gen_range(-1..1); // -1 ccw, 1 cw
        let dir = if formation.start.0 < 0. {-1.} else { 1.};
        let (x_pivot,y_pivot) = formation.pivot;
        let (x_radius, y_radius) = formation.radius;
        
        // compute next angle
        let angle = formation.angle + dir * formation.speed * TIME_STEP / (x_radius.min(y_radius)*PI/2.);

        // Compute target xy
        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;
        let dx = x_org - x_dst;
        let dy = y_org - y_dst;
        let distance = (dx*dx + dy*dy).sqrt();
        let distance_ratio = if distance !=0. {max_distance/distance} else {0.};

        if distance < max_distance * formation.speed / 20. {
            formation.angle = angle;
        }

        // compute final xy
        let x = x_org - dx * distance_ratio;
        let x = if dx>0. {x.max(x_dst)} else {x.min(x_dst)};
        let y = y_org - dy * distance_ratio;
        let y = if dy>0. {y.max(y_dst)} else {y.min(y_dst)};
        let translation = &mut transform.translation;
        velocity.x = -dx*TIME_STEP;
        velocity.y = dy*TIME_STEP;
        (translation.x,translation.y) = (x,y);

        // Rotate to face player
        let diff = transform.translation - player_transform.translation;
        let angle = diff.y.atan2(diff.x) - PI/2.; // Add/sub FRAC_PI here optionally
        transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);
    }
    
}
