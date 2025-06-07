use bevy::{prelude::*, text::FontSmoothing};

use crate::world::{
    collisions::{IntersectionEvent, StaticSensorAABB, GRASS_COLLISION_GROUPS},
    ZLevel, TILE_SIZE,
};

use crate::GameAssets;

use super::{
    MapData, ProgressionSystemSet, CUT_TALL_GRASS_POINTS, MAP_SIZE, TALL_GRASS_CELL_VALUE,
};

const GRASS_GRID_SIZE: (usize, usize) = (1, 1);

#[derive(Component)]
struct TallGrass;
#[derive(Component)]
struct NumberPopUp {
    move_speed: f32,
    timer: Timer,
}

#[derive(Event)]
pub struct CutTallGrass {
    entity: Entity,
    pub pos: Vec2,
}

impl Default for NumberPopUp {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            timer: Timer::from_seconds(0.7, TimerMode::Once),
        }
    }
}

fn spawn_tall_grass(commands: &mut Commands, assets: &GameAssets, pos: Vec2) {
    commands.spawn((
        TallGrass,
        Transform::from_translation(pos.extend(ZLevel::Floor.value())),
        Sprite::from_image(assets.grass.clone()),
        StaticSensorAABB::new(8.0, 8.0),
        GRASS_COLLISION_GROUPS,
    ));
}

fn spawn_grass(mut commands: Commands, assets: Res<GameAssets>, mut map_data: ResMut<MapData>) {
    let size = (MAP_SIZE / 2) as i32;
    for i in -size..size {
        for j in -size..size {
            if i.abs() < 3 && j.abs() < 3 {
                continue;
            }

            let pos = Vec2::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE);
            map_data.set_map_data_value_at_pos(pos, GRASS_GRID_SIZE, TALL_GRASS_CELL_VALUE);
            spawn_tall_grass(&mut commands, &assets, pos);
        }
    }
}

fn despawn_tall_grass(mut commands: Commands, mut ev_cut_tall_grass: EventReader<CutTallGrass>) {
    for ev in ev_cut_tall_grass.read() {
        commands.entity(ev.entity).despawn();
    }
}

fn trigger_cut_tall_grass_event(
    q_grass: Query<&Transform, With<TallGrass>>,
    mut ev_intersection: EventReader<IntersectionEvent>,
    mut ev_cut_tall_grass: EventWriter<CutTallGrass>,
) {
    for ev in ev_intersection.read() {
        let entity = ev.aabb;
        let Ok(transform) = q_grass.get(entity) else {
            continue;
        };

        ev_cut_tall_grass.write(CutTallGrass {
            entity,
            pos: transform.translation.xy(),
        });
    }
}

fn spawn_number_pop_up(commands: &mut Commands, assets: &GameAssets, pos: Vec2) {
    commands.spawn((
        NumberPopUp::default(),
        Text2d(format!("+{}", CUT_TALL_GRASS_POINTS)),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: 8.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(Color::WHITE.with_alpha(1.0)),
        Transform::from_translation(pos.extend(ZLevel::TopUi.value())),
    ));
}

fn spawn_number_pop_ups(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_cut_tall_grass: EventReader<CutTallGrass>,
) {
    for ev in ev_cut_tall_grass.read() {
        spawn_number_pop_up(&mut commands, &assets, ev.pos);
    }
}

fn animate_number_pop_ups(
    time: Res<Time>,
    mut q_pop_ups: Query<(&mut Transform, &mut TextColor, &mut NumberPopUp)>,
) {
    for (mut transform, mut color, mut pop_up) in &mut q_pop_ups {
        pop_up.timer.tick(time.delta());
        transform.translation.y += pop_up.move_speed * time.delta_secs();
        color.set_alpha(1.0 - pop_up.timer.elapsed_secs() / pop_up.timer.duration().as_secs_f32());
    }
}

fn despawn_number_pop_ups(mut commands: Commands, q_pop_ups: Query<(Entity, &NumberPopUp)>) {
    for (entity, pop_up) in &q_pop_ups {
        if pop_up.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct MapGrassPlugin;

impl Plugin for MapGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CutTallGrass>()
            .add_systems(
                Update,
                spawn_grass.run_if(
                    resource_exists::<GameAssets>.and(resource_exists::<MapData>.and(run_once)),
                ),
            )
            .add_systems(
                Update,
                (
                    trigger_cut_tall_grass_event,
                    despawn_tall_grass,
                    spawn_number_pop_ups.run_if(resource_exists::<GameAssets>),
                    animate_number_pop_ups,
                    despawn_number_pop_ups,
                )
                    .chain()
                    .before(ProgressionSystemSet),
            );
    }
}
