mod flora;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{GameAssets, GameState};

use super::TILE_SIZE;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(flora::MapFloraPlugin)
            .add_systems(OnExit(GameState::AssetLoading), spawn_grass);
    }
}

pub enum ZLevel {
    // Background,
    Floor,
    // GroundEffect,
    // BottomEnvironment,
    // TopEnvironment,
    // TopUi,
}

impl ZLevel {
    pub fn value(&self) -> f32 {
        match self {
            // ZLevel::Background => -1e5,
            ZLevel::Floor => -3e4,
            // ZLevel::GroundEffect => -2e4,
            // ZLevel::BottomEnvironment => -1e4,
            // ZLevel::TopEnvironment => 1e4,
            // ZLevel::TopUi => 1e4 + 301.0,
        }
    }
}

fn spawn_grass(mut commands: Commands, assets: Res<GameAssets>) {
    let mut rng = thread_rng();

    let size = 25;
    for i in -size..size {
        for j in -size..size {
            if rng.gen_range(0..100) < 95 {
                continue;
            }

            let pos = Vec2::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE);
            commands.spawn((
                Transform::from_translation(pos.extend(ZLevel::Floor.value())),
                Sprite::from_image(assets.grass.clone()),
            ));
        }
    }
}
