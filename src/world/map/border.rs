use bevy::{color::palettes::css::PINK, prelude::*};

use crate::{
    world::{
        collisions::{ColliderColor, StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    GameAssets, GameState,
};

use super::{MapData, ZLevel, MAP_SIZE};

/// Return the correct asset handle, x flip, ysort above.
fn indices_to_asset(assets: &GameAssets, x: usize, y: usize) -> (Handle<Image>, bool, bool) {
    if y == 0 && (x == 0 || x == MAP_SIZE + 1) {
        return (assets.fence_bottom_corner.clone(), x == 0, true);
    } else if y == MAP_SIZE + 1 && (x == 0 || x == MAP_SIZE + 1) {
        return (assets.fence_top_corner.clone(), x == 0, false);
    } else if x == 0 || x == MAP_SIZE + 1 {
        return (assets.fence_vertical.clone(), x == 0, false);
    } else if y == 0 || y == MAP_SIZE + 1 {
        return (assets.fence_horizontal.clone(), false, y == 0);
    }

    error!("incorrect mapping of fence! Must never happen");
    (assets.fence_horizontal.clone(), true, false)
}

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

fn spawn_fence(commands: &mut Commands, assets: &GameAssets, pos: Vec2, x: usize, y: usize) {
    let (image, flip_x, ysort_above) = indices_to_asset(assets, x, y);
    let z = if ysort_above {
        ZLevel::TopEnvironment.value()
    } else {
        ZLevel::Floor.value()
    };

    commands.spawn((
        Sprite {
            image,
            flip_x,
            ..default()
        },
        Transform::from_translation(pos.extend(z)),
    ));
}

fn spawn_fences(mut commands: Commands, assets: Res<GameAssets>, map_data: Res<MapData>) {
    let offset = Vec2::ONE * TILE_SIZE;

    for x in 0..MAP_SIZE + 2 {
        for y in [0, MAP_SIZE + 1] {
            let pos = map_data.grid_indices_to_pos(x, y) - offset;
            spawn_fence(&mut commands, &assets, pos, x, y);
        }
    }

    for x in [0, MAP_SIZE + 1] {
        for y in 0..MAP_SIZE + 2 {
            let pos = map_data.grid_indices_to_pos(x, y) - offset;
            spawn_fence(&mut commands, &assets, pos, x, y);
        }
    }
}

pub struct MapBorderPlugin;

impl Plugin for MapBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_map_border)
            .add_systems(
                Update,
                spawn_fences.run_if(
                    resource_exists::<GameAssets>.and(resource_exists::<MapData>.and(run_once)),
                ),
            );
    }
}
