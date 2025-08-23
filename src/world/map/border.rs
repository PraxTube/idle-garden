use bevy::{color::palettes::css::PINK, prelude::*};

use crate::{
    world::{
        collisions::{ColliderColor, StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    GameAssets, GameState,
};

use super::{MapData, ZLevel, MAP_SIZE};

fn spawn_map_border(mut commands: Commands) {
    let size = MAP_SIZE as f32 * TILE_SIZE;

    let x = size;
    let y = size;
    let thickness = TILE_SIZE / 4.0;

    let offset = Vec2::NEG_ONE * size * 0.5 - Vec2::ONE * TILE_SIZE * 0.5;

    for (translation, cuboid) in [
        (
            Vec2::new(-thickness, y / 2.0),
            Vec2::new(thickness, y / 2.0),
        ),
        (
            Vec2::new(x / 2.0, -thickness),
            Vec2::new(x / 2.0, thickness),
        ),
        (
            Vec2::new(x + thickness, y / 2.0),
            Vec2::new(thickness, y / 2.0),
        ),
        (
            Vec2::new(x / 2.0, y + thickness),
            Vec2::new(x / 2.0, thickness),
        ),
    ] {
        debug_assert!(cuboid.x > 0.0 && cuboid.y > 0.0);
        commands.spawn((
            WORLD_COLLISION_GROUPS,
            Transform::from_translation((translation + offset).extend(0.0)),
            StaticCollider::new(cuboid.x, cuboid.y),
            ColliderColor(PINK.into()),
        ));
    }
}

fn spawn_dark_area(commands: &mut Commands, pos: Vec2, scale: Vec3) {
    commands.spawn((
        Transform::from_translation(pos.extend(ZLevel::Floor.value() + 1.0)).with_scale(scale),
        Sprite {
            image: Handle::<Image>::default(),
            color: Color::BLACK.with_alpha(0.3).into(),
            ..default()
        },
    ));
}

fn spawn_dark_areas(mut commands: Commands, map_data: Res<MapData>) {
    spawn_dark_area(
        &mut commands,
        map_data.grid_indices_to_pos(0, MAP_SIZE / 2) + Vec2::new(-1.5 * TILE_SIZE, 0.0),
        Vec3::new(2.0 * TILE_SIZE, TILE_SIZE * MAP_SIZE as f32 * 1.5, 1.0),
    );

    spawn_dark_area(
        &mut commands,
        map_data.grid_indices_to_pos(MAP_SIZE, MAP_SIZE / 2) + Vec2::new(0.5 * TILE_SIZE, 0.0),
        Vec3::new(2.0 * TILE_SIZE, TILE_SIZE * MAP_SIZE as f32 * 1.5, 1.0),
    );

    spawn_dark_area(
        &mut commands,
        map_data.grid_indices_to_pos(MAP_SIZE / 2, MAP_SIZE)
            + Vec2::new(-0.5 * TILE_SIZE, 0.5 * TILE_SIZE),
        Vec3::new(TILE_SIZE * MAP_SIZE as f32, 2.0 * TILE_SIZE, 1.0),
    );

    spawn_dark_area(
        &mut commands,
        map_data.grid_indices_to_pos(MAP_SIZE / 2, 0)
            + Vec2::new(-0.5 * TILE_SIZE, -1.7 * TILE_SIZE),
        Vec3::new(TILE_SIZE * MAP_SIZE as f32, 2.0 * TILE_SIZE, 1.0),
    );
}

pub struct MapBorderPlugin;

impl Plugin for MapBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_map_border)
            .add_systems(
                Update,
                spawn_dark_areas.run_if(
                    resource_exists::<GameAssets>.and(resource_exists::<MapData>.and(run_once)),
                ),
            );
    }
}
