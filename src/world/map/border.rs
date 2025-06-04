use bevy::{color::palettes::css::PINK, prelude::*};

use crate::{
    world::{
        collisions::{ColliderColor, StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    GameState,
};

use super::MAP_SIZE;

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

pub struct MapBorderPlugin;

impl Plugin for MapBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_map_border);
    }
}
