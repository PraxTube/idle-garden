use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    world::{
        camera::YSort,
        collisions::{StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    GameAssets, GameState,
};

fn spawn_trees(mut commands: Commands, assets: Res<GameAssets>) {
    let mut rng = thread_rng();

    let size = 25;
    for i in -size..size {
        for j in -size..size {
            if rng.gen_range(0..100) < 98 {
                continue;
            }

            let pos = Vec2::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE);

            let root = commands
                .spawn((
                    Transform::from_translation(pos.extend(0.0)),
                    YSort(2.0 * TILE_SIZE),
                    Sprite::from_image(assets.tree.clone()),
                ))
                .id();

            commands.spawn((
                ChildOf(root),
                Transform::from_translation(Vec3::new(0.0, -2.5 * TILE_SIZE, 0.0)),
                WORLD_COLLISION_GROUPS,
                StaticCollider::new(8.0, 6.0),
            ));
        }
    }
}

pub struct MapFloraPlugin;

impl Plugin for MapFloraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_trees);
    }
}
