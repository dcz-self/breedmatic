use bevy::prelude::*;
use bevy::sprite::entity::{ SpriteBundle, SpriteSheetBundle };
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::ColliderBuilder,
        math::Vector,
        //        math::Point,
    },
};
use rand::{thread_rng, Rng};
use rand_distr::Poisson;
use std::f32;
use std::fs::File;
use super::assets;
use super::components::*;
use super::player::*;
use super::state::{ GameState, Mode, RunState, ValidStates };


use rand_distr::Distribution;
use crate::rapier::WithBody;


/// Pixel perfect.
pub const CAMERA_SCALE: f32 = 1.0;
pub const ARENA_WIDTH: f32 = 640.0;
pub const ARENA_HEIGHT: f32 = 640.0;
/// See spawn zone or not?
const MARGINS: f32 = 1.125;
pub const WINDOW_WIDTH: u32 = (MARGINS * CAMERA_SCALE * ARENA_WIDTH) as u32;
pub const WINDOW_HEIGHT: u32 = (MARGINS * CAMERA_SCALE * ARENA_HEIGHT) as u32;

pub const START_LIFE: u32 = 3;


pub enum ControlledBy {
    Player,
    AI,
}

#[derive(Debug)]
pub struct Arena {
    /// Kinda reflects how often mobs spawn.
    pub mob_virility: f32,
}

pub fn setup_arena(
    commands: &mut Commands,
    mut runstate: ResMut<RunState>,
    assets: Res<assets::Assets>,
) {
    if runstate.gamestate.entering_group_pred(GameState::is_live_arena) {
        runstate.arena = Some(Arena {
            mob_virility: 0.0,
        });
        runstate.score = Some(0);
        let mode = runstate.gamestate.entering().unwrap().arena_mode().unwrap();
        let control = match mode {
            Mode::AI => ControlledBy::AI,
            Mode::Player => ControlledBy::Player,
        };
        spawn_borg(commands, runstate, assets, control);
    }
}

fn spawn_borg(
    commands: &mut Commands,
    mut runstate: ResMut<RunState>,
    assets: Res<assets::Assets>,
    control: ControlledBy,
) {
    let body = RigidBodyBuilder::new_dynamic();
    let collider = ColliderBuilder::ball(5.0);

    commands
        .spawn(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -5.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Borg {
            rotation_speed: std::f32::consts::TAU,
            speed: 30.0,
            life: START_LIFE,
            time_alive: 0.0,
            score: 0,
        })
        .with_body(body)
        .with(collider)
        .with(ValidStates::from_func(GameState::is_live_arena))
        .with_children(|parent| {
            parent.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 100.0, 0.0),
                    scale: Vec3::splat(1.0/32.0),
                    ..Default::default()
                },
                material: assets.arrow.clone().unwrap(),
                ..Default::default()
            }).with(ValidStates::from_func(GameState::is_live_arena));
        });

    let genotype = runstate.shooter_gene_pool.spawn();
    println!("Spawned genotype {}", genotype.pretty_print().unwrap());
    match File::create("shooter.dot")
        .and_then(|mut f| genotype.to_dot(&mut f))
    {
        Err(e) => eprintln!("Filed to write shooter.dot: {:?}", e),
        Ok(_) => println!("Wrote shooter.dot"),
    };
    match control {
        ControlledBy::Player => commands.with(KeyboardWalk),
        ControlledBy::AI => commands.with(genotype),
    };
    
    let borg_entity = commands.current_entity().unwrap();

    commands
        .spawn(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::splat(1.0/8.0),
                ..Default::default()
            },
            material: assets.borg.clone().unwrap(),
            ..Default::default()
        })
        .with(Weapon {
            repeat_timer: Timer::from_seconds(0.5, false),
        })
        .with(AttachedToEntity(borg_entity))
        .with(ValidStates::from_func(GameState::is_live_arena));

    match control {
        ControlledBy::Player => {
            commands.with(LooksAt::default());
            runstate.player = Some(borg_entity);
        },
        _ => {},
    }
}


#[derive(Default)]
pub struct SpawnAsteroidState {
    event_reader: EventReader<AsteroidSpawnEvent>,
}

pub fn spawn_asteroid_system(
    commands: &mut Commands,
    mut local_state: Local<SpawnAsteroidState>,
    assets: Res<assets::Assets>,
    events: Res<Events<AsteroidSpawnEvent>>,
) {
    for event in local_state.event_reader.iter(&events) {
        let body = RigidBodyBuilder::new_dynamic()
            .translation(event.x, event.y);
        let collider = ColliderBuilder::ball(6.0).friction(-0.3);
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: assets.louse.clone().unwrap(),
                sprite: TextureAtlasSprite::new(0),
                transform: {
                    Transform::from_translation(Vec3::new(event.x, event.y, -5.0))
                        .mul_transform(Transform::from_scale(Vec3::splat(0.5)))
                },
                ..Default::default()
            })
            .with(Mob {
                size: event.size,
                life: 1,
                brain: event.brain.clone(),
                rotation_speed: f32::consts::TAU / 4.0,
                speed: 30.0,
            })
            .with(Damage { value: 1 })
            .with_body(body)
            .with(collider)
            .with(ValidStates::from_func(GameState::is_arena));
    }
}

pub fn arena_spawn(
    time: Res<Time>,
    mut runstate: ResMut<RunState>,
    mut asteroid_spawn_events: ResMut<Events<AsteroidSpawnEvent>>,
) {
    if let GameState::Arena(_) = runstate.gamestate.current() {
        let mut arena = runstate.arena.as_mut().unwrap();
        arena.mob_virility += time.delta_seconds();
        // Mobs per second. Double every 30sec.
        let spawn_rate = 0.5 * (2.0f32).powf(arena.mob_virility / 30.0);
        let expected_spawn_this_tick = time.delta_seconds() * spawn_rate;
        let dist = Poisson::new(expected_spawn_this_tick).unwrap();

        let mut rng = thread_rng();
        let mobcount: u64 = dist.sample(&mut rng);
        for _ in 0..mobcount {
            let x: f32 = rng.gen_range(-0.5, 0.5);
            let y: f32 = rng.gen_range(-0.5, 0.5);
            if x.abs() > 0.25 || y.abs() > 0.25 {
                asteroid_spawn_events.send(AsteroidSpawnEvent {
                    size: AsteroidSize::Small,
                    x: x * ARENA_WIDTH,
                    y: y * ARENA_HEIGHT,
                    brain: runstate.mob_gene_pool.spawn(),
                });
            }
        }
    }
}

pub fn hold_borgs(
    runstate: Res<RunState>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<(&RigidBodyHandleComponent, &Borg)>,
) {
    if !runstate.gamestate.current().is_live_arena() {
        return;
    }
    for (body_handle, _borg) in query.iter() {
        let body = bodies.get_mut(body_handle.handle()).unwrap();
        let mut x = body.position().translation.vector.x;
        let mut y = body.position().translation.vector.y;
        let mut xvel = body.linvel().x;
        let mut yvel = body.linvel().y;
        let mut updated = false;
        // Stop at screen edges
        let half_width = ARENA_WIDTH / 2.0;
        let half_height = ARENA_HEIGHT / 2.0;
        if x < -half_width && xvel < 0.0 {
            x = -half_width;
            xvel = 0.0;
            updated = true;
        } else if x > half_width && xvel > 0.0 {
            x = half_width;
            xvel = 0.0;
            updated = true;
        }
        if y < -half_height && yvel < 0.0 {
            y = -half_height;
            updated = true;
            yvel = 0.0;
        } else if y > half_height && yvel > 0.0 {
            y = half_height;
            updated = true;
            yvel = 0.0;
        }
        if updated {
            let mut new_position = body.position().clone();
            new_position.translation.vector.x = x;
            new_position.translation.vector.y = y;
            body.set_linvel(Vector::new(xvel, yvel), false);
            body.set_position(new_position, false);
        }
    }
}

pub fn end_ai_round(
    mut runstate: ResMut<RunState>,
) {
    if let GameState::ArenaOver(Mode::AI) = runstate.gamestate.current() {
        runstate.gamestate.transit_to(GameState::BetweenRounds);
    }
}

pub fn start_ai_round(
    mut runstate: ResMut<RunState>,
) {
    if runstate.gamestate.is(GameState::BetweenRounds) {
        runstate.gamestate.transit_to(GameState::Arena(Mode::AI));
    }
}

pub fn check_end(
    mut runstate: ResMut<RunState>,
    borgs: Query<&Borg>,
) {
    let state = runstate.gamestate.current();
    if !state.is_live_arena() {
        return;
    }
    if borgs.iter().next().is_none() {
        let mode = state.arena_mode().unwrap();
        runstate.gamestate.transit_to(GameState::ArenaOver(mode));
    }
}
