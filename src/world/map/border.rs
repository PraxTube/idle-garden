use bevy::{color::palettes::css::PINK, prelude::*};

use crate::{
    world::{
        collisions::{ColliderColor, StaticCollider, WORLD_COLLISION_GROUPS},
        YSort, TILE_SIZE,
    },
    GameAssets, GameState,
};

use super::{MapData, MAP_SIZE};

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

fn spawn_trees(mut commands: Commands, assets: Res<GameAssets>, map_data: Res<MapData>) {
    let min = map_data.grid_indices_to_pos(0, 0) - Vec2::ONE * TILE_SIZE;
    let max = map_data.grid_indices_to_pos(MAP_SIZE - 1, MAP_SIZE - 1) + Vec2::ONE * TILE_SIZE;

    let mut positions = Vec::new();
    for x in 0..MAP_SIZE + 2 {
        for y in [min.y, max.y] {
            let x = min.x + x as f32 * TILE_SIZE;
            debug_assert!(x <= max.x);
            positions.push(Vec2::new(x, y));
        }
    }

    for x in [min.x, max.x] {
        for y in 0..MAP_SIZE + 2 {
            let y = min.y + y as f32 * TILE_SIZE;
            debug_assert!(y <= max.y);
            positions.push(Vec2::new(x, y));
        }
    }

    for pos in positions {
        commands.spawn((
            Sprite::from_image(assets.pine_tree.clone()),
            Transform::from_translation(pos.extend(0.0)),
            YSort(0.0),
        ));
    }
}

pub struct MapBorderPlugin;

impl Plugin for MapBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_map_border)
            .add_systems(
                Update,
                spawn_trees.run_if(
                    resource_exists::<GameAssets>.and(resource_exists::<MapData>.and(run_once)),
                ),
            );
    }
}
